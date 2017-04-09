//! Tessellation features.
//!
//! # Tessellation mode
//!
//! Tessellation is geometric information. Currently, several kinds of tessellation are supported:
//!
//! - *point clouds*;
//! - *lines*;
//! - *line strips*;
//! - *triangles*;
//! - *triangle fans*;
//! - *triangle strips*.
//!
//! Those kinds of tessellation are designated by the `Mode` type.
//!
//! # Tessellation creation
//!
//! Creation is done via the `Tess::new` function. This function is polymorphing in the type of
//! vertices you send. See the `TessVertices` type for further details.
//!
//! # Tessellation vertices mapping
//!
//! It’s possible to map `Tess`’ vertices into your code. You’re provided with two types to do so:
//!
//! - `BufferSlice`, which gives you an immutable access to the vertices
//! - `BufferSliceMut`, which gives you a mutable access to the vertices
//!
//! You can retrieve those slices with the `Tess::as_slice` and `Tess::as_slice_mut` methods. 
//!
//! # Tessellation render
//!
//! In order to render a `Tess`, you have to use a `TessRender` object. You’ll be able to use that
//! object in *pipelines*. See the `pipeline` module for further details.

use gl;
use gl::types::*;
use std::ptr;

use buffer::{Buffer, BufferError, BufferSlice, BufferSliceMut, RawBuffer};
use vertex::{Dim, Type, Vertex, VertexComponentFormat, VertexFormat};

/// Vertices can be connected via several modes.
#[derive(Copy, Clone, Debug)]
pub enum Mode {
  /// A single point.
  Point,
  /// A line, defined by two points.
  Line,
  /// A strip line, defined by at least two points and zero or many other ones.
  LineStrip,
  /// A triangle, defined by three points.
  Triangle,
  /// A triangle fan, defined by at least three points and zero or many other ones.
  TriangleFan,
  /// A triangle strip, defined by at least three points and zero or many other ones.
  TriangleStrip
}

/// Error that can occur while trying to map GPU tessellation to host code.
#[derive(Debug, Eq, PartialEq)]
pub enum TessMapError {
  MismatchVertexFormat(VertexFormat, VertexFormat),
  VertexBufferMapFailed(BufferError),
  ForbiddenAttributelessMapping
}

/// Accepted vertices for building tessellations.
///
/// This type enables you to pass in a slice of vertices or ask for the GPU to only reserve enough
/// space for the number of vertices, leaving the allocated memory uninitialized.
pub enum TessVertices<'a, T> where T: 'a + Vertex {
  /// Pass in a slice of vertices.
  Fill(&'a [T]),
  /// Reserve a certain number of vertices.
  Reserve(usize)
}

impl<'a, T> From<&'a [T]> for TessVertices<'a, T> where T: 'a + Vertex {
  fn from(slice: &'a [T]) -> Self {
    TessVertices::Fill(slice)
  }
}

/// GPU tessellation.
pub struct Tess {
  mode: GLenum,
  vert_nb: usize,
  vao: GLenum,
  vbo: Option<RawBuffer>, // no vbo means attributeless render
  ibo: Option<RawBuffer>,
  vertex_format: VertexFormat,
}

impl Tess {
  /// Create a new tessellation.
  ///
  /// The `mode` argument gives the type of the primitives and how to interpret the `vertices` and
  /// `indices` slices. If `indices` is set to `None`, the tessellation will use the `vertices`
  /// as-is.
  pub fn new<'a, V, T, O>(mode: Mode, vertices: V, indices: O) -> Self where TessVertices<'a, T>: From<V>, T: 'a + Vertex, O: Into<Option<&'a[u32]>> {
    let vertices = vertices.into();

    let mut vao: GLuint = 0;
    let vert_nb = match vertices {
      TessVertices::Fill(slice) => slice.len(),
      TessVertices::Reserve(nb) => nb
    };

    unsafe {
      gl::GenVertexArrays(1, &mut vao);

      gl::BindVertexArray(vao);

      // vertex buffer
      let vertex_buffer = Buffer::new(vert_nb);

      // fill the buffer with vertices only if asked by the user
      if let TessVertices::Fill(verts) = vertices {
        vertex_buffer.fill(verts);
      }

      let raw_vbo = vertex_buffer.to_raw();

      gl::BindBuffer(gl::ARRAY_BUFFER, raw_vbo.handle());
      set_vertex_pointers(&T::vertex_format());

      // TODO: refactor this schiesse
      // in case of indexed render, create an index buffer
      if let Some(indices) = indices.into() {
        let ind_nb = indices.len();
        let index_buffer = Buffer::new(ind_nb);
        index_buffer.fill(indices);

        let raw_ibo = index_buffer.to_raw();

        gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, raw_ibo.handle());

        gl::BindVertexArray(0);

        Tess {
          mode: opengl_mode(mode),
          vert_nb: ind_nb,
          vao: vao,
          vbo: Some(raw_vbo),
          ibo: Some(raw_ibo),
          vertex_format: T::vertex_format()
        }
      } else {
        gl::BindVertexArray(0);

        Tess {
          mode: opengl_mode(mode),
          vert_nb: vert_nb,
          vao: vao,
          vbo: Some(raw_vbo),
          ibo: None,
          vertex_format: T::vertex_format()
        }
      }
    }
  }

  /// Create a tessellation that will procedurally generate its vertices (i.e. *attribute-less*).
  ///
  /// You just have to give the `Mode` to use and the number of vertices the tessellation must
  /// have. You’ll be handed back a tessellation object that doesn’t actually hold anything. You
  /// will have to generate the vertices on the fly in your shaders.
  pub fn attributeless(mode: Mode, vert_nb: usize) -> Self {
    let mut vao = 0;

    unsafe {
      gl::GenVertexArrays(1, &mut vao);

      gl::BindVertexArray(vao);
      gl::BindVertexArray(0);

      Tess {
        mode: opengl_mode(mode),
        vert_nb: vert_nb,
        vao: vao,
        vbo: None,
        ibo: None,
        vertex_format: Vec::new()
      }
    }
  }

  /// Render the tessellation by providing the number of vertices to pick from it and how many
  /// instances are wished.
  fn render(&self, vert_nb: usize, inst_nb: usize) {
    let vert_nb = vert_nb as GLsizei;
    let inst_nb = inst_nb as GLsizei;

    unsafe {
      gl::BindVertexArray(self.vao);

      if self.ibo.is_some() { // indexed render
        if inst_nb == 1 {
          gl::DrawElements(self.mode, vert_nb, gl::UNSIGNED_INT, ptr::null());
        } else if inst_nb > 1 {
          gl::DrawElementsInstanced(self.mode, vert_nb, gl::UNSIGNED_INT, ptr::null(), inst_nb);
        } else {
          panic!("cannot index-render 0 instance");
        }
      } else { // direct render
        if inst_nb == 1 {
          gl::DrawArrays(self.mode, 0, vert_nb);
        } else if inst_nb > 1 {
          gl::DrawArraysInstanced(self.mode, 0, vert_nb, inst_nb);
        } else {
          panic!("cannot render 0 instance");
        }
      }
    }
  }

  /// Get an immutable slice over the vertices stored on GPU.
  pub fn as_slice<T>(&self) -> Result<BufferSlice<T>, TessMapError> where T: Vertex {
    let live_vf = &self.vertex_format;
    let req_vf = T::vertex_format();

    if live_vf != &req_vf {
      return Err(TessMapError::MismatchVertexFormat(live_vf.clone(), req_vf));
    }

    self.vbo.as_ref()
      .ok_or(TessMapError::ForbiddenAttributelessMapping)
      .and_then(|raw| RawBuffer::as_slice(raw).map_err(TessMapError::VertexBufferMapFailed))
  }

  /// Get a mutable slice over the vertices stored on GPU.
  pub fn as_slice_mut<T>(&mut self) -> Result<BufferSliceMut<T>, TessMapError> where T: Vertex {
    let live_vf = &self.vertex_format;
    let req_vf = T::vertex_format();

    if live_vf != &req_vf {
      return Err(TessMapError::MismatchVertexFormat(live_vf.clone(), req_vf));
    }

    self.vbo.as_mut()
      .ok_or(TessMapError::ForbiddenAttributelessMapping)
      .and_then(|raw| RawBuffer::as_slice_mut(raw).map_err(TessMapError::VertexBufferMapFailed))
  }
}

impl Drop for Tess {
  fn drop(&mut self) {
    // delete the vertex array and all bound buffers
    unsafe {
      gl::DeleteVertexArrays(1, &self.vao);

      if let &Some(ref vbo) = &self.vbo {
        gl::DeleteBuffers(1, &vbo.handle());
      }

      if let &Some(ref ibo) = &self.ibo {
        gl::DeleteBuffers(1, &ibo.handle());
      }
    }
  }
}

// Give OpenGL types information on the content of the VBO by setting vertex formats and pointers
// to buffer memory.
fn set_vertex_pointers(formats: &[VertexComponentFormat]) {
  let offsets = aligned_offsets(formats);
  let vertex_weight = offset_based_vertex_weight(formats, &offsets) as GLsizei;

  for (i, (format, off)) in formats.iter().zip(offsets).enumerate() {
    set_component_format(i as u32, vertex_weight, off, format);
  }
}

// Compute offsets for all the vertex components according to the alignments provided.
fn aligned_offsets(formats: &[VertexComponentFormat]) -> Vec<usize> {
  let mut offsets = Vec::with_capacity(formats.len());
  let mut off = 0;

  // compute offsets
  for f in formats {
    off = off_align(off, f.align); // keep the current component format aligned
    offsets.push(off);
    off += component_weight(f); // increment the offset by the pratical size of the component
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
fn component_weight(f: &VertexComponentFormat) -> usize {
  dim_as_size(&f.dim) as usize * f.unit_size
}

fn dim_as_size(d: &Dim) -> GLint {
  match *d {
    Dim::Dim1 => 1,
    Dim::Dim2 => 2,
    Dim::Dim3 => 3,
    Dim::Dim4 => 4
  }
}

// Weight in bytes of a single vertex, taking into account padding so that the vertex stay correctly
// aligned.
fn offset_based_vertex_weight(formats: &[VertexComponentFormat], offsets: &[usize]) -> usize {
  if formats.is_empty() || offsets.is_empty() {
    return 0;
  }

  off_align(offsets[offsets.len() - 1] + component_weight(&formats[formats.len() - 1]), formats[0].align)
}

// Set the vertex component OpenGL pointers regarding the index of the component (i), the stride
fn set_component_format(i: u32, stride: GLsizei, off: usize, f: &VertexComponentFormat) {
  match f.comp_type {
    Type::Floating => {
      unsafe {
        gl::VertexAttribPointer(i as GLuint, dim_as_size(&f.dim), opengl_sized_type(&f), gl::FALSE, stride, ptr::null().offset(off as isize));
      }
    },
    Type::Integral | Type::Unsigned | Type::Boolean => {
      unsafe {
        gl::VertexAttribIPointer(i as GLuint, dim_as_size(&f.dim), opengl_sized_type(&f), stride, ptr::null().offset(off as isize));
      }
    }
  }

  unsafe {
    gl::EnableVertexAttribArray(i as GLuint);
  }
}

fn opengl_sized_type(f: &VertexComponentFormat) -> GLenum {
  match (f.comp_type, f.unit_size) {
    (Type::Integral, 1) => gl::BYTE,
    (Type::Integral, 2) => gl::SHORT,
    (Type::Integral, 4) => gl::INT,
    (Type::Unsigned, 1) | (Type::Boolean, 1) => gl::UNSIGNED_BYTE,
    (Type::Unsigned, 2) => gl::UNSIGNED_SHORT,
    (Type::Unsigned, 4) => gl::UNSIGNED_INT,
    (Type::Floating, 4) => gl::FLOAT,
    _ => panic!("unsupported vertex component format: {:?}", f)
  }
}

fn opengl_mode(mode: Mode) -> GLenum {
  match mode {
    Mode::Point => gl::POINTS,
    Mode::Line => gl::LINES,
    Mode::LineStrip => gl::LINE_STRIP,
    Mode::Triangle => gl::TRIANGLES,
    Mode::TriangleFan => gl::TRIANGLE_FAN,
    Mode::TriangleStrip => gl::TRIANGLE_STRIP
  }
}

/// Tessellation render.
#[derive(Clone)]
pub struct TessRender<'a> {
  /// Tessellation to render.
  tess: &'a Tess,
  /// Number of vertices to pick from the tessellation. If `None`, all of them are selected.
  vert_nb: usize,
  /// Number of instances to render.
  inst_nb: usize
}

impl<'a> TessRender<'a> {
  /// Create a tessellation render for the whole tessellation once.
  pub fn one_whole(tess: &'a Tess) -> Self {
    TessRender {
      tess: tess,
      vert_nb: tess.vert_nb,
      inst_nb: 1
    }
  }

  /// Create a tessellation render for a part of the tessellation once. The part is selected by
  /// giving the number of vertices to render. This function can then be used to use the
  /// tessellation’s vertex buffer as one see fit.
  ///
  /// # Panic
  ///
  /// Panic if the number of vertices is higher to the capacity of the tessellation’s vertex buffer.
  pub fn one_sub(tess: &'a Tess, vert_nb: usize) -> Self {
    if vert_nb > tess.vert_nb {
      panic!("cannot render {} vertices for a tessellation which vertex capacity is {}", vert_nb, tess.vert_nb);
    }

    TessRender {
      tess: tess,
      vert_nb: vert_nb,
      inst_nb: 1
    }
  }

  pub fn render(&self) {
    self.tess.render(self.vert_nb, self.inst_nb);
  }
}

impl<'a> From<&'a Tess> for TessRender<'a> {
  fn from(tess: &'a Tess) -> Self {
    TessRender::one_whole(tess)
  }
}
