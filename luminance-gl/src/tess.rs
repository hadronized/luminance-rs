use gl;
use gl::types::*;
use luminance::buffer::Buffer as BufferBackend;
use luminance::context::GraphicsContext;
use luminance::tess::{Mode, Tess as TessBackend, TessBuilder as TessBuilderBackend, TessIndex, IndexSlice as IndexSliceBackend, IndexSliceMut as IndexSliceMutBackend, TessIndexType, InstanceSlice as InstanceSliceBackend, InstanceSliceMut as InstanceSliceMutBackend, VertexSlice as VertexSliceBackend, VertexSliceMut as VertexSliceMutBackend};
use luminance::vertex::{Normalized, Vertex, VertexAttribDesc, VertexAttribDim, VertexAttribType, VertexBufferDesc, VertexDesc, VertexInstancing};
use luminance::vertex_restart::VertexRestart;
use std::cell::RefCell;
use std::fmt;
use std::ops::{Range, RangeFrom, RangeFull, RangeInclusive, RangeTo, RangeToInclusive};
use std::os::raw::c_void;
use std::ptr;
use std::rc::Rc;

use crate::buffer::{Buffer, BufferError, BufferSlice, BufferSliceMut, RawBuffer};
use crate::state::{Bind, GraphicsState};

/// Error that can occur while trying to map GPU tessellation to host code.
#[derive(Debug, Eq, PartialEq)]
pub enum TessMapError {
  /// The CPU mapping failed due to buffer errors.
  VertexBufferMapFailed(BufferError),
  /// The CPU mapping failed due to buffer errors.
  IndexBufferMapFailed(BufferError),
  /// Vertex target type is not the same as the one stored in the buffer.
  VertexTypeMismatch(VertexDesc, VertexDesc),
  /// Index target type is not the same as the one stored in the buffer.
  IndexTypeMismatch(TessIndexType, TessIndexType),
  /// The CPU mapping failed because you cannot map an attributeless tessellation since it doesn’t
  /// have any vertex attribute.
  ForbiddenAttributelessMapping,
  /// The CPU mapping failed because currently, mapping deinterleaved buffers is not supported via
  /// a single slice.
  ForbiddenDeinterleavedMapping,
}

impl fmt::Display for TessMapError {
  fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
    match *self {
      TessMapError::VertexBufferMapFailed(ref e) =>
        write!(f, "cannot map tessellation vertex buffer: {}", e),
      TessMapError::IndexBufferMapFailed(ref e) =>
        write!(f, "cannot map tessellation index buffer: {}", e),
      TessMapError::VertexTypeMismatch(ref a, ref b) =>
       write!(f, "cannot map tessellation: vertex type mismatch between {:?} and {:?}", a, b),
      TessMapError::IndexTypeMismatch(ref a, ref b) =>
       write!(f, "cannot map tessellation: index type mismatch between {:?} and {:?}", a, b),
      TessMapError::ForbiddenAttributelessMapping => f.write_str("cannot map an attributeless buffer"),
      TessMapError::ForbiddenDeinterleavedMapping =>
        f.write_str("cannot map a deinterleaved buffer as interleaved")
    }
  }
}

struct VertexBuffer {
  /// Indexed format of the buffer.
  fmt: VertexDesc,
  /// Internal buffer.
  buf: RawBuffer,
}

/// Build tessellations the easy way.
///
/// This type allows you to create [`Tess`] by specifying piece-by-piece what the tessellation is
/// made of. Several situations and configurations are supported.
///
/// # Specifying vertices
///
/// If you want to create a [`Tess`] holding vertices without anything else, you want to use the
/// [`TessBuilder::add_vertices`]. Every time that function is called, a _vertex buffer_ is
/// virtually allocated for your tessellation, which gives you three possibilities:
///
/// ## 1. Attributeless [`Tess`]
///
/// If you don’t call that function, you end up with an _attributeless_ tessellation. Such a
/// tessellation has zero memory allocated to vertices. Instead, when invoking a _vertex shader_,
/// the vertices must be created on the fly _inside_ the vertex shader directly.
///
/// ## 2. Interleaved [`Tess`]
///
/// If you call that function once, you have a single _vertex buffer_ allocated, which either
/// gives you a 1-attribute tessellation, or an interleaved tessellation. Interleaved tessellation
/// allows you to use a Rust `struct` (if it implements the [`Vertex`] trait) as vertex type and
/// easily fetch them from a vertex shader.
///
/// ## 3. Deinterleaved [`Tess`]
///
/// If you call that function several times, the [`TessBuilder`] assumes you want _deinterleaved_
/// memory, which means that each patch of vertices you add is supposed to contain one type of
/// deinterleaved vertex attributes. A coherency check is done by the [`TessBuilder`] to ensure
/// the vertex data is correct.
///
/// # Specifying indices
///
/// By default, vertices are picked in the order you specify them in the vertex buffer(s). If you
/// want more control on the order, you can add _indices_.
///
/// As soon as you provide indices, the [`TessBuilder`] will change the way [`Tess`] will fetch
/// vertices. Instead of fetching the first vertex, then second, then third, etc., it will first
/// fetch the first index, then the second, then third, and respectively use the value of those
/// indices to fetch the actual vertices.
///
/// For instance, if instead of fetching vertices `[1, 2, 3`] (which is the default) you want to
/// fetch `[12, 35, 2]`, you can add the `[12, 35, 2]` indices in the [`TessBuilder`]. When
/// rendering, the [`Tess`] will fetch the first index and get `12`; it will then make the first
/// vertex to be fetched the 12th; then fetch the second index; get `35` and fetch the 35th vertex.
/// Finally, as you might have guessed, it will fetch the third index, get `2` and then the third
/// vertex to be fetched will be the second one.
///
/// That feature is really important as it allows you to _factorize_ vertices: instead of
/// duplicating them, you can just reuse their indices.
///
/// You can have only one set of indices. See the [`TessBuilder::set_indices`] function.
///
/// # Specifying vertex instancing
///
/// It’s also possible to provide instancing information. Those are special vertex attributes that
/// are picked on an _instance_-based information instead of _vertex number_ one. It works very
/// similarly to how vertices data work, but on a per-instance bases.
///
/// See the [`TessBuilder::add_instances`] function for further details.
pub struct TessBuilder<'a, C> {
  ctx: &'a mut C,
  vertex_buffers: Vec<VertexBuffer>,
  index_buffer: Option<(RawBuffer, TessIndexType)>,
  restart_index: Option<u32>,
  mode: Mode,
  vert_nb: usize,
  instance_buffers: Vec<VertexBuffer>,
  inst_nb: usize,
}

impl<'a, C> TessBuilder<'a, C> {
  /// Create a new, default [`TessBuilder`].
  ///
  /// By default, the _primitive mode_ of the building [`Tess`] is [`Mode::Point`].
  pub fn new(ctx: &'a mut C) -> Self {
    TessBuilder {
      ctx,
      vertex_buffers: Vec::new(),
      index_buffer: None,
      restart_index: None,
      mode: Mode::Point,
      vert_nb: 0,
      instance_buffers: Vec::new(),
      inst_nb: 0,
    }
  }
}

impl<'a, C> TessBuilder<'a, C> where C: GraphicsContext<State = GraphicsState> {
  /// Add vertices to be part of the tessellation.
  ///
  /// This method can be used in several ways. First, you can decide to use interleaved memory, in
  /// which case you will call this method only once by providing an interleaved slice / borrowed
  /// buffer. Second, you can opt-in to use deinterleaved memory, in which case you will have
  /// several, smaller buffers of borrowed data and you will issue a call to this method for all of
  /// them.
  pub fn add_vertices<V, W>(mut self, vertices: W) -> Self where W: AsRef<[V]>, V: Vertex {
    let vertices = vertices.as_ref();

    let vb = VertexBuffer {
      fmt: V::vertex_desc(),
      buf: Buffer::from_slice(self.ctx, vertices).into_raw(),
    };

    self.vertex_buffers.push(vb);

    self
  }

  /// Add instances to be part of the tessellation.
  pub fn add_instances<V, W>(mut self, instances: W) -> Self where W: AsRef<[V]>, V: Vertex {
    let instances = instances.as_ref();

    let vb = VertexBuffer {
      fmt: V::vertex_desc(),
      buf: Buffer::from_slice(self.ctx, instances).into_raw(),
    };

    self.instance_buffers.push(vb);

    self
  }

  /// Set vertex indices in order to specify how vertices should be picked by the GPU pipeline.
  pub fn set_indices<T, I>(mut self, indices: T) -> Self where T: AsRef<[I]>, I: TessIndex  {
    let indices = indices.as_ref();

    // create a new raw buffer containing the indices and turn it into a vertex buffer
    let buf = Buffer::from_slice(self.ctx, indices).into_raw();

    self.index_buffer = Some((buf, I::INDEX_TYPE));

    self
  }

  /// Set the primitive mode for the building [`Tess`].
  pub fn set_mode(mut self, mode: Mode) -> Self {
    self.mode = mode;
    self
  }

  /// Set the default number of vertices to be rendered.
  ///
  /// That function is not mandatory if you are not building an _attributeless_ tessellation but is
  /// if you are.
  ///
  /// When called while building a [`Tess`] owning at least one vertex buffer, it acts as a _default_
  /// number of vertices to render and is useful when you will slice the tessellation with open
  /// ranges.
  pub fn set_vertex_nb(mut self, nb: usize) -> Self {
    self.vert_nb = nb;
    self
  }

  /// Set the default number of instances to render.
  ///
  /// `0` disables geometry instancing.
  pub fn set_instance_nb(mut self, nb: usize) -> Self {
    self.inst_nb = nb;
    self
  }

  /// Set the primitive restart index. The initial value is `None`, implying no primitive restart.
  pub fn set_primitive_restart_index(mut self, index: Option<u32>) -> Self {
    self.restart_index = index;
    self
  }

  /// Build the [`Tess`].
  pub fn build(self) -> Result<Tess, TessError> {
    // try to deduce the number of vertices to render if it’s not specified
    let vert_nb = self.guess_vert_nb_or_fail()?;
    let inst_nb = self.guess_inst_nb_or_fail()?;
    self.build_tess(vert_nb, inst_nb)
  }

  /// Build a tessellation based on a given number of vertices to render by default.
  fn build_tess(self, vert_nb: usize, inst_nb: usize) -> Result<Tess, TessError> {
    let mut vao: GLuint = 0;

    unsafe {
      let mut gfx_st = self.ctx.state().borrow_mut();

      gl::GenVertexArrays(1, &mut vao);

      // force binding the vertex array so that previously bound vertex arrays (possibly the same
      // handle) don’t prevent us from binding here
      gfx_st.bind_vertex_array(vao, Bind::Forced);

      // add the vertex buffers into the vao
      for vb in &self.vertex_buffers {
        // force binding as it’s meaningful when a vao is bound
        gfx_st.bind_array_buffer(vb.buf.handle(), Bind::Forced);
        set_vertex_pointers(&vb.fmt)
      }

      // in case of indexed render, create an index buffer
      if let Some(ref index_buffer) = self.index_buffer {
        // force binding as it’s meaningful when a vao is bound
        gfx_st.bind_element_array_buffer(index_buffer.0.handle(), Bind::Forced);
      }

      // add any instance buffers, if any
      for vb in &self.instance_buffers {
        // force binding as it’s meaningful when a vao is bound
        gfx_st.bind_array_buffer(vb.buf.handle(), Bind::Forced);
        set_vertex_pointers(&vb.fmt);
      }

      let restart_index = self.restart_index;
      let index_state = self.index_buffer.map(move |(buffer, index_type)| {
        IndexedDrawState {
          restart_index,
          _buffer: buffer,
          index_type,
        }
      });

      // convert to OpenGL-friendly internals and return
      Ok(Tess {
        mode: opengl_mode(self.mode),
        vert_nb,
        inst_nb,
        vao,
        vertex_buffers: self.vertex_buffers,
        instance_buffers: self.instance_buffers,
        index_state,
        state: self.ctx.state().clone(),
      })
    }
  }

  /// Guess how many vertices there are to render based on the current configuration or fail if
  /// incorrectly configured.
  fn guess_vert_nb_or_fail(&self) -> Result<usize, TessError> {
    if self.vert_nb == 0 {
      // we don’t have an explicit vertex number to render; go and guess!
      if let Some(ref index_buffer) = self.index_buffer {
        // we have an index buffer: just use its size
        Ok(index_buffer.0.len())
      } else {
        // deduce the number of vertices based on the vertex buffers; they all
        // must be of the same length, otherwise it’s an error
        match self.vertex_buffers.len() {
          0 => {
            Err(TessError::AttributelessError("attributeless render with no vertex number".to_owned()))
          }

          1 => {
            Ok(self.vertex_buffers[0].buf.len())
          }

          _ => {
            let vert_nb = self.vertex_buffers[0].buf.len();
            let incoherent = Self::check_incoherent_buffers(self.vertex_buffers.iter(), vert_nb);

            if incoherent {
              Err(TessError::LengthIncoherency(vert_nb))
            } else {
              Ok(vert_nb)
            }
          }
        }
      }
    } else {
      // we have an explicit number of vertices to render, but we’re gonna check that number actually
      // makes sense
      if let Some(ref index_buffer) = self.index_buffer {
        // we have indices (indirect draw); so we’ll compare to them
        if index_buffer.0.len() < self.vert_nb {
          return Err(TessError::Overflow(index_buffer.0.len(), self.vert_nb));
        }
      } else {
        let incoherent = Self::check_incoherent_buffers(self.vertex_buffers.iter(), self.vert_nb);

        if incoherent {
          return Err(TessError::LengthIncoherency(self.vert_nb));
        } else if !self.vertex_buffers.is_empty() && self.vertex_buffers[0].buf.len() < self.vert_nb {
          return Err(TessError::Overflow(self.vertex_buffers[0].buf.len(), self.vert_nb));
        }
      }

      Ok(self.vert_nb)
    }
  }

  /// Check whether any vertex buffer is incoherent in its length according to the input length.
  fn check_incoherent_buffers<'b, B>(mut buffers: B, len: usize) -> bool
  where B: Iterator<Item = &'b VertexBuffer> {
    !buffers.all(|vb| vb.buf.len() == len)
  }

  /// Guess how many instances there are to render based on the current configuration or fail if
  /// incorrectly configured.
  fn guess_inst_nb_or_fail(&self) -> Result<usize, TessError> {
    if self.inst_nb == 0 {
      // we don’t have an explicit instance number to render; go and guess!
      // deduce the number of instances based on the instance buffers; they all must be of the same
      // length, otherwise it’s an error
      match self.instance_buffers.len() {
        0 => {
          // no instance buffer; we we’re not using instance rendering
          Ok(0)
        }

        1 => {
          Ok(self.instance_buffers[0].buf.len())
        }

        _ => {
          let inst_nb = self.instance_buffers[0].buf.len();
          let incoherent = Self::check_incoherent_buffers(self.instance_buffers.iter(), inst_nb);

          if incoherent {
            Err(TessError::LengthIncoherency(inst_nb))
          } else {
            Ok(inst_nb)
          }
        }
      }
    } else {
      // we have an explicit number of instances to render, but we’re gonna check that number
      // actually makes sense
      let incoherent = Self::check_incoherent_buffers(self.instance_buffers.iter(), self.vert_nb);

      if incoherent {
        return Err(TessError::LengthIncoherency(self.inst_nb));
      } else if !self.instance_buffers.is_empty() && self.instance_buffers[0].buf.len() < self.inst_nb {
        return Err(TessError::Overflow(self.instance_buffers[0].buf.len(), self.inst_nb));
      }

      Ok(self.inst_nb)
    }
  }
}

/// Possible errors that might occur when dealing with [`Tess`].
#[derive(Debug)]
pub enum TessError {
  /// Error related to attributeless tessellation and/or render.
  AttributelessError(String),
  /// Length incoherency in vertex, index or instance buffers.
  LengthIncoherency(usize),
  /// Overflow when accessing underlying buffers.
  Overflow(usize, usize)
}

fn tess_index_type_to_glenum(ty: TessIndexType) -> GLenum {
  match ty {
    TessIndexType::U8 => gl::UNSIGNED_BYTE,
    TessIndexType::U16 => gl::UNSIGNED_SHORT,
    TessIndexType::U32 => gl::UNSIGNED_INT,
  }
}

/// All the extra data required when doing indexed drawing.
struct IndexedDrawState {
  _buffer: RawBuffer,
  restart_index: Option<u32>,
  index_type: TessIndexType,
}

/// GPU tessellation.
///
/// GPU tessellations gather several pieces of information:
///
///   - _Vertices_, which define points in space associated with _vertex attributes_, giving them
///     meaningful data. Those data are then processed by a _vertex shader_ to produce more
///     interesting data down the graphics pipeline.
///   - _Indices_, which are used to change the order the _vertices_ are fetched to form
///     _primitives_ (lines, triangles, etc.).
///   - _Primitive mode_, the way vertices should be linked together. See [`Mode`] for further
///     details.
///   - And other information used to determine how to render such tessellations.
///
/// A [`Tess`] doesn’t directly state how to render an object, it just describes its topology and
/// inner construction (i.e. mesh).
///
/// Constructing a [`Tess`] is not doable directly: you need to use a [`TessBuilder`] first.
pub struct Tess {
  mode: GLenum,
  vert_nb: usize,
  inst_nb: usize,
  vao: GLenum,
  vertex_buffers: Vec<VertexBuffer>,
  instance_buffers: Vec<VertexBuffer>,
  index_state: Option<IndexedDrawState>,
  state: Rc<RefCell<GraphicsState>>,
}

impl Tess {
  fn render<C>(&self, ctx: &mut C, start_index: usize, vert_nb: usize, inst_nb: usize)
  where C: ?Sized + GraphicsContext<State = GraphicsState> {
    let vert_nb = vert_nb as GLsizei;
    let inst_nb = inst_nb as GLsizei;

    unsafe {
      let mut gfx_st = ctx.state().borrow_mut();
      gfx_st.bind_vertex_array(self.vao, Bind::Cached);

      if let Some(index_state) = self.index_state.as_ref() {
        // indexed render
        let first = (index_state.index_type.bytes() * start_index) as *const c_void;

        if let Some(restart_index) = index_state.restart_index {
          gfx_st.set_vertex_restart(VertexRestart::On);
          gl::PrimitiveRestartIndex(restart_index);
        } else {
          gfx_st.set_vertex_restart(VertexRestart::Off);
        }

        if inst_nb <= 1 {
          gl::DrawElements(self.mode, vert_nb, tess_index_type_to_glenum(index_state.index_type), first);
        } else {
          gl::DrawElementsInstanced(
            self.mode,
            vert_nb,
            tess_index_type_to_glenum(index_state.index_type),
            first,
            inst_nb,
          );
        }
      } else {
        // direct render
        let first = start_index as GLint;

        if inst_nb <= 1 {
          gl::DrawArrays(self.mode, first, vert_nb);
        } else {
          gl::DrawArraysInstanced(self.mode, first, vert_nb, inst_nb);
        }
      }
    }
  }

  /// Obtain a slice over the vertex buffer.
  ///
  /// This function fails if you try to obtain a buffer from an attritebuteless [`Tess`] or
  /// deinterleaved memory.
  pub fn as_slice<V>(&mut self) -> Result<BufferSlice<V>, TessMapError> where V: Vertex {
    match self.vertex_buffers.len() {
      0 => Err(TessMapError::ForbiddenAttributelessMapping),

      1 => {
        let vb = &mut self.vertex_buffers[0];
        let target_fmt = V::vertex_desc(); // costs a bit

        if vb.fmt != target_fmt {
          Err(TessMapError::VertexTypeMismatch(vb.fmt.clone(), target_fmt))
        } else {
          vb.buf.as_slice().map_err(TessMapError::VertexBufferMapFailed)
        }
      }

      _ => Err(TessMapError::ForbiddenDeinterleavedMapping),
    }
  }

  /// Obtain a mutable slice over the vertex buffer.
  ///
  /// This function fails if you try to obtain a buffer from an attriteless [`Tess`] or
  /// deinterleaved memory.
  pub fn as_slice_mut<V>(&mut self) -> Result<BufferSliceMut<V>, TessMapError> where V: Vertex {
    match self.vertex_buffers.len() {
      0 => Err(TessMapError::ForbiddenAttributelessMapping),

      1 => {
        let vb = &mut self.vertex_buffers[0];
        let target_fmt = V::vertex_desc(); // costs a bit

        if vb.fmt != target_fmt {
          Err(TessMapError::VertexTypeMismatch(vb.fmt.clone(), target_fmt))
        } else {
          vb.buf.as_slice_mut().map_err(TessMapError::VertexBufferMapFailed)
        }
      }

      _ => Err(TessMapError::ForbiddenDeinterleavedMapping),
    }
  }

  /// Obtain a slice over the index buffer.
  ///
  /// This function fails if you try to obtain a buffer from an attriteless [`Tess`] or if no
  /// index buffer is available.
  pub fn as_index_slice<I>(&mut self) -> Result<BufferSlice<I>, TessMapError> where I: TessIndex {
    match self.index_state {
      Some(IndexedDrawState { ref mut _buffer, ref index_type, .. }) => {
        let target_fmt = I::INDEX_TYPE;

        if *index_type != target_fmt {
          Err(TessMapError::IndexTypeMismatch(*index_type, target_fmt))
        } else {
          _buffer.as_slice().map_err(TessMapError::IndexBufferMapFailed)
        }
      }

      None => Err(TessMapError::ForbiddenAttributelessMapping)
    }
  }

  /// Obtain a mutable slice over the index buffer.
  ///
  /// This function fails if you try to obtain a buffer from an attriteless [`Tess`] or if no
  /// index buffer is available.
  pub fn as_index_slice_mut<I>(
    &mut self
  ) -> Result<BufferSliceMut<I>, TessMapError>
  where I: TessIndex {
    match self.index_state {
      Some(IndexedDrawState { ref mut _buffer, ref index_type, .. }) => {
        let target_fmt = I::INDEX_TYPE;

        if *index_type != target_fmt {
          Err(TessMapError::IndexTypeMismatch(*index_type, target_fmt))
        } else {
          _buffer.as_slice_mut().map_err(TessMapError::IndexBufferMapFailed)
        }
      }

      None => Err(TessMapError::ForbiddenAttributelessMapping)
    }
  }

  /// Obtain a slice over the instance buffer.
  ///
  /// This function fails if you try to obtain a buffer from an attriteless [`Tess`] or
  /// deinterleaved memory.
  pub fn as_inst_slice<V>(&mut self) -> Result<BufferSlice<V>, TessMapError> where V: Vertex {
    match self.instance_buffers.len() {
      0 => Err(TessMapError::ForbiddenAttributelessMapping),

      1 => {
        let vb = &mut self.instance_buffers[0];
        let target_fmt = V::vertex_desc(); // costs a bit

        if vb.fmt != target_fmt {
          Err(TessMapError::VertexTypeMismatch(vb.fmt.clone(), target_fmt))
        } else {
          vb.buf.as_slice().map_err(TessMapError::VertexBufferMapFailed)
        }
      }

      _ => Err(TessMapError::ForbiddenDeinterleavedMapping),
    }
  }

  /// Obtain a mutable slice over the instance buffer.
  ///
  /// This function fails if you try to obtain a buffer from an attriteless [`Tess`] or
  /// deinterleaved memory.
  pub fn as_inst_slice_mut<V>(&mut self) -> Result<BufferSliceMut<V>, TessMapError> where V: Vertex {
    match self.instance_buffers.len() {
      0 => Err(TessMapError::ForbiddenAttributelessMapping),

      1 => {
        let vb = &mut self.instance_buffers[0];
        let target_fmt = V::vertex_desc(); // costs a bit

        if vb.fmt != target_fmt {
          Err(TessMapError::VertexTypeMismatch(vb.fmt.clone(), target_fmt))
        } else {
          vb.buf.as_slice_mut().map_err(TessMapError::VertexBufferMapFailed)
        }
      }

      _ => Err(TessMapError::ForbiddenDeinterleavedMapping),
    }
  }
}

impl Drop for Tess {
  fn drop(&mut self) {
    unsafe {
      self.state.borrow_mut().unbind_vertex_array();
      gl::DeleteVertexArrays(1, &self.vao);
    }
  }
}

// Give OpenGL types information on the content of the VBO by setting vertex descriptors and pointers
// to buffer memory.
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

// Compute offsets for all the vertex components according to the alignments provided.
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

// Align an offset.
#[inline]
fn off_align(off: usize, align: usize) -> usize {
  let a = align - 1;
  (off + a) & !a
}

// Weight in bytes of a vertex component.
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

// Weight in bytes of a single vertex, taking into account padding so that the vertex stay correctly
// aligned.
fn offset_based_vertex_weight(descriptors: &[VertexBufferDesc], offsets: &[usize]) -> usize {
  if descriptors.is_empty() || offsets.is_empty() {
    return 0;
  }

  off_align(
    offsets[offsets.len() - 1] + component_weight(&descriptors[descriptors.len() - 1].attrib_desc),
    descriptors[0].attrib_desc.align,
  )
}

// Set the vertex component OpenGL pointers regarding the index of the component (i), the stride
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

      VertexAttribType::Integral(Normalized::No) | VertexAttribType::Unsigned(Normalized::No) |
      VertexAttribType::Boolean => {
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
      VertexInstancing::Off => 0
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
  }
}

/// Tessellation slice.
///
/// This type enables slicing a tessellation on the fly so that we can render patches of it.
/// Typically, you can obtain a slice by using the [`TessSliceIndex`] trait (the
/// [`TessSliceIndex::slice`] method) and combining it with some Rust range operators, such as
/// [`..`] or [`..=`].
///
/// [`..`]: https://doc.rust-lang.org/std/ops/struct.RangeFull.html
/// [`..=`]: https://doc.rust-lang.org/std/ops/struct.RangeInclusive.html
#[derive(Clone)]
pub struct TessSlice<'a> {
  /// Tessellation to render.
  tess: &'a Tess,
  /// Start index (vertex) in the tessellation.
  start_index: usize,
  /// Number of vertices to pick from the tessellation.
  vert_nb: usize,
  /// Number of instances to render.
  inst_nb: usize,
}

impl<'a> TessSlice<'a> {
  /// Create a tessellation render that will render the whole input tessellation with only one
  /// instance.
  pub fn one_whole(tess: &'a Tess) -> Self {
    TessSlice {
      tess,
      start_index: 0,
      vert_nb: tess.vert_nb,
      inst_nb: tess.inst_nb,
    }
  }

  /// Create a tessellation render that will render the whole input tessellation with as many
  /// instances as specified.
  pub fn inst_whole(tess: &'a Tess, inst_nb: usize) -> Self {
    TessSlice {
      tess,
      start_index: 0,
      vert_nb: tess.vert_nb,
      inst_nb,
    }
  }

  /// Create a tessellation render for a part of the tessellation starting at the beginning of its
  /// buffer with only one instance.
  ///
  /// The part is selected by giving the number of vertices to render.
  ///
  /// > Note: if you also need to use an arbitrary part of your tessellation (not starting at the
  /// > first vertex in its buffer), have a look at `TessSlice::one_slice`.
  ///
  /// # Panic
  ///
  /// Panic if the number of vertices is higher to the capacity of the tessellation’s vertex buffer.
  pub fn one_sub(tess: &'a Tess, vert_nb: usize) -> Self {
    if vert_nb > tess.vert_nb {
      panic!(
        "cannot render {} vertices for a tessellation which vertex capacity is {}",
        vert_nb, tess.vert_nb
      );
    }

    TessSlice {
      tess,
      start_index: 0,
      vert_nb,
      inst_nb: 1,
    }
  }

  /// Create a tessellation render for a part of the tessellation starting at the beginning of its
  /// buffer with as many instances as specified.
  ///
  /// The part is selected by giving the number of vertices to render.
  ///
  /// > Note: if you also need to use an arbitrary part of your tessellation (not starting at the
  /// > first vertex in its buffer), have a look at `TessSlice::one_slice`.
  ///
  /// # Panic
  ///
  /// Panic if the number of vertices is higher to the capacity of the tessellation’s vertex buffer.
  pub fn inst_sub(tess: &'a Tess, vert_nb: usize, inst_nb: usize) -> Self {
    if vert_nb > tess.vert_nb {
      panic!(
        "cannot render {} vertices for a tessellation which vertex capacity is {}",
        vert_nb, tess.vert_nb
      );
    }

    TessSlice {
      tess,
      start_index: 0,
      vert_nb,
      inst_nb,
    }
  }

  /// Create a tessellation render for a slice of the tessellation starting anywhere in its buffer
  /// with only one instance.
  ///
  /// The part is selected by giving the start vertex and the number of vertices to render.
  ///
  /// # Panic
  ///
  /// Panic if the start vertex is higher to the capacity of the tessellation’s vertex buffer.
  ///
  /// Panic if the number of vertices is higher to the capacity of the tessellation’s vertex buffer.
  pub fn one_slice(tess: &'a Tess, start: usize, nb: usize) -> Self {
    if start > tess.vert_nb {
      panic!(
        "cannot render {} vertices starting at vertex {} for a tessellation which vertex capacity is {}",
        nb, start, tess.vert_nb
      );
    }

    if nb > tess.vert_nb {
      panic!(
        "cannot render {} vertices for a tessellation which vertex capacity is {}",
        nb, tess.vert_nb
      );
    }

    TessSlice {
      tess,
      start_index: start,
      vert_nb: nb,
      inst_nb: 1,
    }
  }

  /// Create a tessellation render for a slice of the tessellation starting anywhere in its buffer
  /// with as many instances as specified.
  ///
  /// The part is selected by giving the start vertex and the number of vertices to render.
  ///
  /// # Panic
  ///
  /// Panic if the start vertex is higher to the capacity of the tessellation’s vertex buffer.
  ///
  /// Panic if the number of vertices is higher to the capacity of the tessellation’s vertex buffer.
  pub fn inst_slice(tess: &'a Tess, start: usize, nb: usize, inst_nb: usize) -> Self {
    if start > tess.vert_nb {
      panic!(
        "cannot render {} vertices starting at vertex {} for a tessellation which vertex capacity is {}",
        nb, start, tess.vert_nb
      );
    }

    if nb > tess.vert_nb {
      panic!(
        "cannot render {} vertices for a tessellation which vertex capacity is {}",
        nb, tess.vert_nb
      );
    }

    TessSlice {
      tess,
      start_index: start,
      vert_nb: nb,
      inst_nb,
    }
  }

  /// Render a tessellation.
  pub fn render<C>(&self, ctx: &mut C) where C: ?Sized + GraphicsContext<State = GraphicsState> {
    self
      .tess
      .render(ctx, self.start_index, self.vert_nb, self.inst_nb);
  }
}

impl<'a> From<&'a Tess> for TessSlice<'a> {
  fn from(tess: &'a Tess) -> Self {
    TessSlice::one_whole(tess)
  }
}

/// The [`Tess`] slice index feature.
///
/// That trait allows to use the syntax `tess.slice(_)` where `_` is one of Rust range operators:
///
///   - [`..`](https://doc.rust-lang.org/std/ops/struct.RangeFull.html) for the whole range.
///   - [`a .. b`](https://doc.rust-lang.org/std/ops/struct.Range.html) for a sub-range, excluding
///     the right part.
///   - [`a ..= b`](https://doc.rust-lang.org/std/ops/struct.RangeInclusive.html) for a sub-range,
///     including the right part.
///   - [`a ..`](https://doc.rust-lang.org/std/ops/struct.RangeFrom.html) for a sub-range open
///     on the right part.
///   - [`.. b`](https://doc.rust-lang.org/std/ops/struct.RangeTo.html) for a sub-range open on the
///     left part and excluding the right part.
///   - [`..= b](https://doc.rust-lang.org/std/ops/struct.RangeToInclusive.html) for a sub-range
///     open on the left part and including the right part.
///
/// It’s technically possible to add any kind of index type, even though not really useful so far.
///
/// Additionally, you can use the `tess.inst_slice(range, inst_nb)` construct to also specify
/// the render should be performed with `inst_nb` instances.
pub trait TessSliceIndex<Idx> {
  /// Slice a tesselation object and yields a [`TessSlice`] according to the index range.
  fn slice(&self, idx: Idx) -> TessSlice;

  /// Slice a tesselation object and yields a [`TessSlice`] according to the index range with as
  /// many instances as specified.
  fn inst_slice(&self, idx: Idx, inst_nb: usize) -> TessSlice;
}

impl TessSliceIndex<RangeFull> for Tess {
  fn slice(&self, _: RangeFull) -> TessSlice {
    TessSlice::one_whole(self)
  }

  fn inst_slice(&self, _: RangeFull, inst_nb: usize) -> TessSlice {
    TessSlice::inst_whole(self, inst_nb)
  }
}

impl TessSliceIndex<RangeTo<usize>> for Tess {
  fn slice(&self, to: RangeTo<usize>) -> TessSlice {
    TessSlice::one_sub(self, to.end)
  }

  fn inst_slice(&self, to: RangeTo<usize>, inst_nb: usize) -> TessSlice {
    TessSlice::inst_sub(self, to.end, inst_nb)
  }
}

impl TessSliceIndex<RangeFrom<usize>> for Tess {
  fn slice(&self, from: RangeFrom<usize>) -> TessSlice {
    TessSlice::one_slice(self, from.start, self.vert_nb - from.start)
  }

  fn inst_slice(&self, from: RangeFrom<usize>, inst_nb: usize) -> TessSlice {
    TessSlice::inst_slice(self, from.start, self.vert_nb - from.start, inst_nb)
  }
}

impl TessSliceIndex<Range<usize>> for Tess {
  fn slice(&self, range: Range<usize>) -> TessSlice {
    TessSlice::one_slice(self, range.start, range.end - range.start)
  }

  fn inst_slice(&self, range: Range<usize>, inst_nb: usize) -> TessSlice {
    TessSlice::inst_slice(self, range.start, range.end - range.start, inst_nb)
  }
}

impl TessSliceIndex<RangeInclusive<usize>> for Tess {
  fn slice(&self, range: RangeInclusive<usize>) -> TessSlice {
    let start = *range.start();
    let end = *range.end();
    TessSlice::one_slice(self, start, end - start + 1)
  }

  fn inst_slice(&self, range: RangeInclusive<usize>, inst_nb: usize) -> TessSlice {
    let start = *range.start();
    let end = *range.end();
    TessSlice::inst_slice(self, start, end - start + 1, inst_nb)
  }
}

impl TessSliceIndex<RangeToInclusive<usize>> for Tess {
  fn slice(&self, to: RangeToInclusive<usize>) -> TessSlice {
    TessSlice::one_sub(self, to.end + 1)
  }

  fn inst_slice(&self, to: RangeToInclusive<usize>, inst_nb: usize) -> TessSlice {
    TessSlice::inst_sub(self, to.end + 1, inst_nb)
  }
}

impl<'a, C> TessBuilderBackend<'a, C> for TessBuilder<'a, C>
where C: GraphicsContext<State = GraphicsState> {
  type Tess = Tess;

  type Err = TessError;

  fn new(ctx: &'a mut C) -> Self {
    TessBuilder::new(ctx)
  }

  fn add_vertices<V, W>(self, vertices: W) -> Self where W: AsRef<[V]>, V: Vertex {
    TessBuilder::add_vertices(self, vertices)
  }

  fn add_instances<V, W>(self, instances: W) -> Self where W: AsRef<[V]>, V: Vertex {
    TessBuilder::add_instances(self, instances)
  }

  fn set_indices<T, I>(self, indices: T) -> Self where T: AsRef<[I]>, I: TessIndex {
    TessBuilder::set_indices(self, indices)
  }

  fn set_mode(self, mode: Mode) -> Self {
    TessBuilder::set_mode(self, mode)
  }

  fn set_vertex_nb(self, nb: usize) -> Self {
    TessBuilder::set_vertex_nb(self, nb)
  }

  fn set_instance_nb(self, nb: usize) -> Self {
    TessBuilder::set_instance_nb(self, nb)
  }

  fn set_primitive_restart_index(self, index: Option<u32>) -> Self {
    TessBuilder::set_primitive_restart_index(self, index)
  }

  fn build(self) -> Result<Self::Tess, Self::Err> {
    TessBuilder::build(self)
  }
}

impl<C> TessBackend<C> for Tess where C: GraphicsContext<State = GraphicsState> {
  type Err = TessMapError;

  fn render(&self, ctx: &mut C, start_index: usize, vert_nb: usize, inst_nb: usize) {
    Tess::render(self, ctx, start_index, vert_nb, inst_nb)
  }
}

impl<'a, C, B, V> VertexSliceBackend<'a, C, B, V> for Tess
where C: GraphicsContext<State = GraphicsState>,
      B: BufferBackend<'a, C, V, Slice = BufferSlice<'a, V>>,
      V: 'a + Vertex {
  fn as_slice(&'a mut self) -> Result<B::Slice, Self::Err> {
    Tess::as_slice(self)
  }
}

impl<'a, C, B, V> VertexSliceMutBackend<'a, C, B, V> for Tess
where C: GraphicsContext<State = GraphicsState>,
      B: BufferBackend<'a, C, V, SliceMut = BufferSliceMut<'a, V>>,
      V: 'a + Vertex {
  fn as_slice_mut(&'a mut self) -> Result<B::SliceMut, Self::Err> {
    Tess::as_slice_mut(self)
  }
}


impl<'a, C, B, I> IndexSliceBackend<'a, C, B, I> for Tess
where C: GraphicsContext<State = GraphicsState>,
      B: BufferBackend<'a, C, I, Slice = BufferSlice<'a, I>>,
      I: 'a + TessIndex {
  fn as_index_slice(&'a mut self) -> Result<B::Slice, Self::Err> {
    Tess::as_index_slice(self)
  }
}

impl<'a, C, B, I> IndexSliceMutBackend<'a, C, B, I> for Tess
where C: GraphicsContext<State = GraphicsState>,
      B: BufferBackend<'a, C, I, SliceMut = BufferSliceMut<'a, I>>,
      I: 'a + TessIndex {
  fn as_index_slice_mut(&'a mut self) -> Result<B::SliceMut, Self::Err> {
    Tess::as_index_slice_mut(self)
  }
}

impl<'a, C, B, V> InstanceSliceBackend<'a, C, B, V> for Tess
where C: GraphicsContext<State = GraphicsState>,
      B: BufferBackend<'a, C, V, Slice = BufferSlice<'a, V>>,
      V: 'a + Vertex {
  fn as_instance_slice(&'a mut self) -> Result<B::Slice, Self::Err> {
    Tess::as_inst_slice(self)
  }
}

impl<'a, C, B, V> InstanceSliceMutBackend<'a, C, B, V> for Tess
where C: GraphicsContext<State = GraphicsState>,
      B: BufferBackend<'a, C, V, SliceMut = BufferSliceMut<'a, V>>,
      V: 'a + Vertex {
  fn as_instance_slice_mut(&'a mut self) -> Result<B::SliceMut, Self::Err> {
    Tess::as_inst_slice_mut(self)
  }
}
