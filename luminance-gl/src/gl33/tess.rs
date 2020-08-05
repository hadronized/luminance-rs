use gl::types::*;
use std::cell::RefCell;
use std::marker::PhantomData;
use std::os::raw::c_void;
use std::ptr;
use std::rc::Rc;

use crate::gl33::buffer::{Buffer, BufferSlice, BufferSliceMut};
use crate::gl33::state::{Bind, GLState};
use crate::gl33::vertex_restart::VertexRestart;
use crate::gl33::GL33;
use luminance::backend::buffer::{Buffer as _, BufferSlice as _};
use luminance::backend::tess::{
  IndexSlice as IndexSliceBackend, InstanceSlice as InstanceSliceBackend, Tess as TessBackend,
  VertexSlice as VertexSliceBackend,
};
use luminance::tess::{
  Deinterleaved, DeinterleavedData, Interleaved, Mode, TessError, TessIndex, TessIndexType,
  TessMapError, TessVertexData,
};
use luminance::vertex::{
  Deinterleave, Normalized, Vertex, VertexAttribDesc, VertexAttribDim, VertexAttribType,
  VertexBufferDesc, VertexInstancing,
};

/// All the extra data required when doing indexed drawing.
struct IndexedDrawState<I>
where
  I: TessIndex,
{
  buffer: Buffer<I>,
  restart_index: Option<I>,
}

struct TessRaw<I>
where
  I: TessIndex,
{
  vao: GLenum,
  mode: GLenum,
  vert_nb: usize,
  inst_nb: usize,
  patch_vert_nb: usize,
  index_state: Option<IndexedDrawState<I>>,
  state: Rc<RefCell<GLState>>,
}

impl<I> TessRaw<I>
where
  I: TessIndex,
{
  unsafe fn render(
    &self,
    start_index: usize,
    vert_nb: usize,
    inst_nb: usize,
  ) -> Result<(), TessError> {
    let vert_nb = vert_nb as GLsizei;
    let inst_nb = inst_nb as GLsizei;

    let mut gfx_st = self.state.borrow_mut();
    gfx_st.bind_vertex_array(self.vao, Bind::Cached);

    if self.mode == gl::PATCHES {
      gfx_st.set_patch_vertex_nb(self.patch_vert_nb);
    }

    match (I::INDEX_TYPE, self.index_state.as_ref()) {
      (Some(index_ty), Some(index_state)) => {
        // indexed render
        let first = (index_ty.bytes() * start_index) as *const c_void;

        if let Some(restart_index) = index_state.restart_index {
          gfx_st.set_vertex_restart(VertexRestart::On);
          gl::PrimitiveRestartIndex(restart_index.try_into_u32().unwrap_or(0));
        } else {
          gfx_st.set_vertex_restart(VertexRestart::Off);
        }

        if inst_nb <= 1 {
          gl::DrawElements(self.mode, vert_nb, index_type_to_glenum(index_ty), first);
        } else {
          gl::DrawElementsInstanced(
            self.mode,
            vert_nb,
            index_type_to_glenum(index_ty),
            first,
            inst_nb,
          );
        }
      }

      _ => {
        // direct render
        let first = start_index as GLint;

        if inst_nb <= 1 {
          gl::DrawArrays(self.mode, first, vert_nb);
        } else {
          gl::DrawArraysInstanced(self.mode, first, vert_nb, inst_nb);
        }
      }
    }

    Ok(())
  }
}

impl<I> Drop for TessRaw<I>
where
  I: TessIndex,
{
  fn drop(&mut self) {
    unsafe {
      self.state.borrow_mut().unbind_vertex_array();
      gl::DeleteVertexArrays(1, &self.vao);
    }
  }
}

pub struct InterleavedTess<V, I, W>
where
  V: Vertex,
  I: TessIndex,
  W: Vertex,
{
  raw: TessRaw<I>,
  vertex_buffer: Option<Buffer<V>>,
  instance_buffer: Option<Buffer<W>>,
}

unsafe impl<V, I, W> TessBackend<V, I, W, Interleaved> for GL33
where
  V: TessVertexData<Interleaved, Data = Vec<V>>,
  I: TessIndex,
  W: TessVertexData<Interleaved, Data = Vec<W>>,
{
  type TessRepr = InterleavedTess<V, I, W>;

  unsafe fn build(
    &mut self,
    (vertex_data, vert_nb): (Option<V::Data>, usize),
    (instance_data, inst_nb): (Option<W::Data>, usize),
    index_data: Vec<I>,
    restart_index: Option<I>,
    mode: Mode,
  ) -> Result<Self::TessRepr, TessError> {
    let mut vao: GLuint = 0;

    let patch_vert_nb = match mode {
      Mode::Patch(nb) => nb,
      _ => 0,
    };

    gl::GenVertexArrays(1, &mut vao);

    // force binding the vertex array so that previously bound vertex arrays (possibly the same
    // handle) don’t prevent us from binding here
    self.state.borrow_mut().bind_vertex_array(vao, Bind::Forced);

    let vertex_buffer = build_interleaved_vertex_buffer(self, vertex_data)?;

    // in case of indexed render, create an index buffer
    let index_state = build_index_buffer(self, index_data, restart_index)?;

    let instance_buffer = build_interleaved_vertex_buffer(self, instance_data)?;

    let mode = opengl_mode(mode);
    let state = self.state.clone();
    let raw = TessRaw {
      vao,
      mode,
      vert_nb,
      inst_nb,
      patch_vert_nb,
      index_state,
      state,
    };

    // convert to OpenGL-friendly internals and return
    Ok(InterleavedTess {
      raw,
      vertex_buffer,
      instance_buffer,
    })
  }

  unsafe fn tess_vertices_nb(tess: &Self::TessRepr) -> usize {
    tess.raw.vert_nb
  }

  unsafe fn tess_instances_nb(tess: &Self::TessRepr) -> usize {
    tess.raw.inst_nb
  }

  unsafe fn render(
    tess: &Self::TessRepr,
    start_index: usize,
    vert_nb: usize,
    inst_nb: usize,
  ) -> Result<(), TessError> {
    tess.raw.render(start_index, vert_nb, inst_nb)
  }
}

unsafe impl<V, I, W> VertexSliceBackend<V, I, W, Interleaved, V> for GL33
where
  V: TessVertexData<Interleaved, Data = Vec<V>>,
  I: TessIndex,
  W: TessVertexData<Interleaved, Data = Vec<W>>,
{
  type VertexSliceRepr = BufferSlice<V>;
  type VertexSliceMutRepr = BufferSliceMut<V>;

  unsafe fn vertices(tess: &mut Self::TessRepr) -> Result<Self::VertexSliceRepr, TessMapError> {
    match tess.vertex_buffer {
      Some(ref vb) => Ok(GL33::slice_buffer(vb)?),
      None => Err(TessMapError::forbidden_attributeless_mapping()),
    }
  }

  unsafe fn vertices_mut(
    tess: &mut Self::TessRepr,
  ) -> Result<Self::VertexSliceMutRepr, TessMapError> {
    match tess.vertex_buffer {
      Some(ref mut vb) => Ok(GL33::slice_buffer_mut(vb)?),
      None => Err(TessMapError::forbidden_attributeless_mapping()),
    }
  }
}

unsafe impl<V, I, W> IndexSliceBackend<V, I, W, Interleaved> for GL33
where
  V: TessVertexData<Interleaved, Data = Vec<V>>,
  I: TessIndex,
  W: TessVertexData<Interleaved, Data = Vec<W>>,
{
  type IndexSliceRepr = BufferSlice<I>;
  type IndexSliceMutRepr = BufferSliceMut<I>;

  unsafe fn indices(tess: &mut Self::TessRepr) -> Result<Self::IndexSliceRepr, TessMapError> {
    match tess.raw.index_state {
      Some(ref state) => Ok(GL33::slice_buffer(&state.buffer)?),
      None => Err(TessMapError::forbidden_attributeless_mapping()),
    }
  }

  unsafe fn indices_mut(
    tess: &mut Self::TessRepr,
  ) -> Result<Self::IndexSliceMutRepr, TessMapError> {
    match tess.raw.index_state {
      Some(ref mut state) => Ok(GL33::slice_buffer_mut(&mut state.buffer)?),
      None => Err(TessMapError::forbidden_attributeless_mapping()),
    }
  }
}

unsafe impl<V, I, W> InstanceSliceBackend<V, I, W, Interleaved, W> for GL33
where
  V: TessVertexData<Interleaved, Data = Vec<V>>,
  I: TessIndex,
  W: TessVertexData<Interleaved, Data = Vec<W>>,
{
  type InstanceSliceRepr = BufferSlice<W>;
  type InstanceSliceMutRepr = BufferSliceMut<W>;

  unsafe fn instances(tess: &mut Self::TessRepr) -> Result<Self::InstanceSliceRepr, TessMapError> {
    match tess.instance_buffer {
      Some(ref vb) => Ok(GL33::slice_buffer(vb)?),
      None => Err(TessMapError::forbidden_attributeless_mapping()),
    }
  }

  unsafe fn instances_mut(
    tess: &mut Self::TessRepr,
  ) -> Result<Self::InstanceSliceMutRepr, TessMapError> {
    match tess.instance_buffer {
      Some(ref mut vb) => Ok(GL33::slice_buffer_mut(vb)?),
      None => Err(TessMapError::forbidden_attributeless_mapping()),
    }
  }
}

pub struct DeinterleavedTess<V, I, W>
where
  V: Vertex,
  I: TessIndex,
  W: Vertex,
{
  raw: TessRaw<I>,
  vertex_buffers: Vec<Buffer<u8>>,
  instance_buffers: Vec<Buffer<u8>>,
  _phantom: PhantomData<*const (V, W)>,
}

unsafe impl<V, I, W> TessBackend<V, I, W, Deinterleaved> for GL33
where
  V: TessVertexData<Deinterleaved, Data = Vec<DeinterleavedData>>,
  I: TessIndex,
  W: TessVertexData<Deinterleaved, Data = Vec<DeinterleavedData>>,
{
  type TessRepr = DeinterleavedTess<V, I, W>;

  unsafe fn build(
    &mut self,
    (vertex_data, vert_nb): (Option<V::Data>, usize),
    (instance_data, inst_nb): (Option<W::Data>, usize),
    index_data: Vec<I>,
    restart_index: Option<I>,
    mode: Mode,
  ) -> Result<Self::TessRepr, TessError> {
    let mut vao: GLuint = 0;

    let patch_vert_nb = match mode {
      Mode::Patch(nb) => nb,
      _ => 0,
    };

    gl::GenVertexArrays(1, &mut vao);

    // force binding the vertex array so that previously bound vertex arrays (possibly the same
    // handle) don’t prevent us from binding here
    self.state.borrow_mut().bind_vertex_array(vao, Bind::Forced);

    let vertex_buffers = build_deinterleaved_vertex_buffers::<V>(self, vertex_data)?;

    // in case of indexed render, create an index buffer
    let index_state = build_index_buffer(self, index_data, restart_index)?;

    let instance_buffers = build_deinterleaved_vertex_buffers::<W>(self, instance_data)?;

    let mode = opengl_mode(mode);
    let state = self.state.clone();
    let raw = TessRaw {
      vao,
      mode,
      vert_nb,
      inst_nb,
      patch_vert_nb,
      index_state,
      state,
    };

    // convert to OpenGL-friendly internals and return
    Ok(DeinterleavedTess {
      raw,
      vertex_buffers,
      instance_buffers,
      _phantom: PhantomData,
    })
  }

  unsafe fn tess_vertices_nb(tess: &Self::TessRepr) -> usize {
    tess.raw.vert_nb
  }

  unsafe fn tess_instances_nb(tess: &Self::TessRepr) -> usize {
    tess.raw.inst_nb
  }

  unsafe fn render(
    tess: &Self::TessRepr,
    start_index: usize,
    vert_nb: usize,
    inst_nb: usize,
  ) -> Result<(), TessError> {
    tess.raw.render(start_index, vert_nb, inst_nb)
  }
}

unsafe impl<V, I, W, T> VertexSliceBackend<V, I, W, Deinterleaved, T> for GL33
where
  V: TessVertexData<Deinterleaved, Data = Vec<DeinterleavedData>> + Deinterleave<T>,
  I: TessIndex,
  W: TessVertexData<Deinterleaved, Data = Vec<DeinterleavedData>>,
{
  type VertexSliceRepr = BufferSlice<T>;
  type VertexSliceMutRepr = BufferSliceMut<T>;

  unsafe fn vertices(tess: &mut Self::TessRepr) -> Result<Self::VertexSliceRepr, TessMapError> {
    if tess.vertex_buffers.is_empty() {
      Err(TessMapError::forbidden_attributeless_mapping())
    } else {
      let buffer = &tess.vertex_buffers[V::RANK];
      let slice = GL33::slice_buffer(buffer)?.transmute();
      Ok(slice)
    }
  }

  unsafe fn vertices_mut(
    tess: &mut Self::TessRepr,
  ) -> Result<Self::VertexSliceMutRepr, TessMapError> {
    if tess.vertex_buffers.is_empty() {
      Err(TessMapError::forbidden_attributeless_mapping())
    } else {
      let buffer = &mut tess.vertex_buffers[V::RANK];
      let slice = GL33::slice_buffer_mut(buffer)?.transmute();
      Ok(slice)
    }
  }
}

unsafe impl<V, I, W> IndexSliceBackend<V, I, W, Deinterleaved> for GL33
where
  V: TessVertexData<Deinterleaved, Data = Vec<DeinterleavedData>>,
  I: TessIndex,
  W: TessVertexData<Deinterleaved, Data = Vec<DeinterleavedData>>,
{
  type IndexSliceRepr = BufferSlice<I>;
  type IndexSliceMutRepr = BufferSliceMut<I>;

  unsafe fn indices(tess: &mut Self::TessRepr) -> Result<Self::IndexSliceRepr, TessMapError> {
    match tess.raw.index_state {
      Some(ref state) => Ok(GL33::slice_buffer(&state.buffer)?),
      None => Err(TessMapError::forbidden_attributeless_mapping()),
    }
  }

  unsafe fn indices_mut(
    tess: &mut Self::TessRepr,
  ) -> Result<Self::IndexSliceMutRepr, TessMapError> {
    match tess.raw.index_state {
      Some(ref mut state) => Ok(GL33::slice_buffer_mut(&mut state.buffer)?),
      None => Err(TessMapError::forbidden_attributeless_mapping()),
    }
  }
}

unsafe impl<V, I, W, T> InstanceSliceBackend<V, I, W, Deinterleaved, T> for GL33
where
  V: TessVertexData<Deinterleaved, Data = Vec<DeinterleavedData>>,
  I: TessIndex,
  W: TessVertexData<Deinterleaved, Data = Vec<DeinterleavedData>> + Deinterleave<T>,
{
  type InstanceSliceRepr = BufferSlice<T>;
  type InstanceSliceMutRepr = BufferSliceMut<T>;

  unsafe fn instances(tess: &mut Self::TessRepr) -> Result<Self::InstanceSliceRepr, TessMapError> {
    if tess.instance_buffers.is_empty() {
      Err(TessMapError::forbidden_attributeless_mapping())
    } else {
      let buffer = &tess.instance_buffers[W::RANK];
      let slice = GL33::slice_buffer(buffer)?.transmute();
      Ok(slice)
    }
  }

  unsafe fn instances_mut(
    tess: &mut Self::TessRepr,
  ) -> Result<Self::InstanceSliceMutRepr, TessMapError> {
    if tess.instance_buffers.is_empty() {
      Err(TessMapError::forbidden_attributeless_mapping())
    } else {
      let buffer = &mut tess.instance_buffers[W::RANK];
      let slice = GL33::slice_buffer_mut(buffer)?.transmute();
      Ok(slice)
    }
  }
}

fn build_interleaved_vertex_buffer<V>(
  gl33: &mut GL33,
  vertices: Option<Vec<V>>,
) -> Result<Option<Buffer<V>>, TessError>
where
  V: Vertex,
{
  match vertices {
    Some(vertices) => {
      let fmt = V::vertex_desc();

      let vb = if vertices.is_empty() {
        None
      } else {
        let vb = unsafe { gl33.from_vec(vertices)? };

        // force binding as it’s meaningful when a vao is bound
        unsafe {
          gl33
            .state
            .borrow_mut()
            .bind_array_buffer(vb.handle(), Bind::Forced)
        };
        set_vertex_pointers(&fmt);

        Some(vb)
      };

      Ok(vb)
    }

    None => Ok(None),
  }
}

fn build_deinterleaved_vertex_buffers<V>(
  gl33: &mut GL33,
  vertices: Option<Vec<DeinterleavedData>>,
) -> Result<Vec<Buffer<u8>>, TessError>
where
  V: Vertex,
{
  match vertices {
    Some(attributes) => {
      attributes
        .into_iter()
        .zip(V::vertex_desc())
        .map(|(attribute, fmt)| {
          let vb = unsafe { gl33.from_vec(attribute.into_vec())? };

          // force binding as it’s meaningful when a vao is bound
          unsafe {
            gl33
              .state
              .borrow_mut()
              .bind_array_buffer(vb.handle(), Bind::Forced);
            set_vertex_pointers(&[fmt]);
          }

          Ok(vb.into_raw())
        })
        .collect::<Result<Vec<_>, _>>()
    }

    None => Ok(Vec::new()),
  }
}

/// Turn a [`Vec`] of indices to an [`IndexedDrawState`].
fn build_index_buffer<I>(
  gl33: &mut GL33,
  data: Vec<I>,
  restart_index: Option<I>,
) -> Result<Option<IndexedDrawState<I>>, TessError>
where
  I: TessIndex,
{
  let ids = if !data.is_empty() {
    let ib = IndexedDrawState {
      buffer: unsafe { gl33.from_vec(data)? },
      restart_index,
    };

    // force binding as it’s meaningful when a vao is bound
    unsafe {
      gl33
        .state
        .borrow_mut()
        .bind_element_array_buffer(ib.buffer.handle(), Bind::Forced);
    }

    Some(ib)
  } else {
    None
  };

  Ok(ids)
}

/// Give OpenGL types information on the content of the VBO by setting vertex descriptors and pointers
/// to buffer memory.
fn set_vertex_pointers(descriptors: &[VertexBufferDesc]) {
  // this function sets the vertex attribute pointer for the input list by computing:
  //   - The vertex attribute ID: this is the “rank” of the attribute in the input list (order
  //     matters, for short).
  //   - The stride: this is easily computed, since it’s the size (bytes) of a single vertex.
  //   - The offsets: each attribute has a given offset in the buffer. This is computed by
  //     accumulating the size of all previously set attributes.
  let offsets = aligned_offsets(descriptors);
  let vertex_weight = offset_based_vertex_weight(descriptors, &offsets) as GLsizei;

  for (desc, off) in descriptors.iter().zip(offsets) {
    set_component_format(vertex_weight, off, desc);
  }
}

/// Compute offsets for all the vertex components according to the alignments provided.
fn aligned_offsets(descriptor: &[VertexBufferDesc]) -> Vec<usize> {
  let mut offsets = Vec::with_capacity(descriptor.len());
  let mut off = 0;

  // compute offsets
  for desc in descriptor {
    let desc = &desc.attrib_desc;
    off = off_align(off, desc.align); // keep the current component descriptor aligned
    offsets.push(off);
    off += component_weight(desc); // increment the offset by the pratical size of the component
  }

  offsets
}

/// Align an offset.
#[inline]
fn off_align(off: usize, align: usize) -> usize {
  let a = align - 1;
  (off + a) & !a
}

/// Weight in bytes of a vertex component.
fn component_weight(f: &VertexAttribDesc) -> usize {
  dim_as_size(f.dim) as usize * f.unit_size
}

fn dim_as_size(d: VertexAttribDim) -> GLint {
  match d {
    VertexAttribDim::Dim1 => 1,
    VertexAttribDim::Dim2 => 2,
    VertexAttribDim::Dim3 => 3,
    VertexAttribDim::Dim4 => 4,
  }
}

/// Weight in bytes of a single vertex, taking into account padding so that the vertex stay correctly
/// aligned.
fn offset_based_vertex_weight(descriptors: &[VertexBufferDesc], offsets: &[usize]) -> usize {
  if descriptors.is_empty() || offsets.is_empty() {
    return 0;
  }

  off_align(
    offsets[offsets.len() - 1] + component_weight(&descriptors[descriptors.len() - 1].attrib_desc),
    descriptors[0].attrib_desc.align,
  )
}

/// Set the vertex component OpenGL pointers regarding the index of the component and the vertex
/// stride.
fn set_component_format(stride: GLsizei, off: usize, desc: &VertexBufferDesc) {
  let attrib_desc = &desc.attrib_desc;
  let index = desc.index as GLuint;

  unsafe {
    match attrib_desc.ty {
      VertexAttribType::Floating => {
        gl::VertexAttribPointer(
          index,
          dim_as_size(attrib_desc.dim),
          opengl_sized_type(&attrib_desc),
          gl::FALSE,
          stride,
          ptr::null::<c_void>().add(off),
        );
      }

      VertexAttribType::Integral(Normalized::No)
      | VertexAttribType::Unsigned(Normalized::No)
      | VertexAttribType::Boolean => {
        // non-normalized integrals / booleans
        gl::VertexAttribIPointer(
          index,
          dim_as_size(attrib_desc.dim),
          opengl_sized_type(&attrib_desc),
          stride,
          ptr::null::<c_void>().add(off),
        );
      }

      _ => {
        // normalized integrals
        gl::VertexAttribPointer(
          index,
          dim_as_size(attrib_desc.dim),
          opengl_sized_type(&attrib_desc),
          gl::TRUE,
          stride,
          ptr::null::<c_void>().add(off),
        );
      }
    }

    // set vertex attribute divisor based on the vertex instancing configuration
    let divisor = match desc.instancing {
      VertexInstancing::On => 1,
      VertexInstancing::Off => 0,
    };
    gl::VertexAttribDivisor(index, divisor);

    gl::EnableVertexAttribArray(index);
  }
}

fn opengl_sized_type(f: &VertexAttribDesc) -> GLenum {
  match (f.ty, f.unit_size) {
    (VertexAttribType::Integral(_), 1) => gl::BYTE,
    (VertexAttribType::Integral(_), 2) => gl::SHORT,
    (VertexAttribType::Integral(_), 4) => gl::INT,
    (VertexAttribType::Unsigned(_), 1) | (VertexAttribType::Boolean, 1) => gl::UNSIGNED_BYTE,
    (VertexAttribType::Unsigned(_), 2) => gl::UNSIGNED_SHORT,
    (VertexAttribType::Unsigned(_), 4) => gl::UNSIGNED_INT,
    (VertexAttribType::Floating, 4) => gl::FLOAT,
    _ => panic!("unsupported vertex component format: {:?}", f),
  }
}

fn opengl_mode(mode: Mode) -> GLenum {
  match mode {
    Mode::Point => gl::POINTS,
    Mode::Line => gl::LINES,
    Mode::LineStrip => gl::LINE_STRIP,
    Mode::Triangle => gl::TRIANGLES,
    Mode::TriangleFan => gl::TRIANGLE_FAN,
    Mode::TriangleStrip => gl::TRIANGLE_STRIP,
    Mode::Patch(_) => gl::PATCHES,
  }
}

fn index_type_to_glenum(ty: TessIndexType) -> GLenum {
  match ty {
    TessIndexType::U8 => gl::UNSIGNED_BYTE,
    TessIndexType::U16 => gl::UNSIGNED_SHORT,
    TessIndexType::U32 => gl::UNSIGNED_INT,
  }
}
