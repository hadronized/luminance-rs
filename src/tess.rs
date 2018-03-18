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
use std::error::Error;
use std::fmt;
use std::marker::PhantomData;
use std::mem::size_of;
use std::os::raw::c_void;
use std::ptr;

use buffer::{Buffer, BufferError, BufferSlice, BufferSliceMut, RawBuffer};
use vertex::{Dim, Type, Vertex, VertexComponentFormat};

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
  VertexBufferMapFailed(BufferError),
  ForbiddenAttributelessMapping
}

impl fmt::Display for TessMapError {
  fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
    f.write_str(self.description())
  }
}

impl Error for TessMapError {
  fn description(&self) -> &str {
    match *self {
      TessMapError::VertexBufferMapFailed(_) => "vertex buffer map failed",
      TessMapError::ForbiddenAttributelessMapping => "cannot map an attributeless buffer"
    }
  }

  fn cause(&self) -> Option<&Error> {
    match *self {
      TessMapError::VertexBufferMapFailed(ref e) => Some(e),
      _ => None
    }
  }
}

/// Accepted vertices for building tessellations.
///
/// This type enables you to pass in a slice of vertices or ask for the GPU to only reserve enough
/// space for the number of vertices, leaving the allocated memory uninitialized.
#[derive(Debug, Eq, PartialEq)]
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

/// GPU typed tessellation.
///
/// The tessellation is typed with the vertex type.
#[derive(Debug, Eq, PartialEq)]
pub struct Tess<V> {
  mode: GLenum,
  vert_nb: usize,
  vao: GLenum,
  vbo: Option<RawBuffer>, // no vbo means attributeless render
  ibo: Option<RawBuffer>,
  _v: PhantomData<V>
}

impl<V> Tess<V> where V: Vertex {
  /// Create a new tessellation.
  ///
  /// The `mode` argument gives the type of the primitives and how to interpret the `vertices` and
  /// `indices` slices. If `indices` is set to `None`, the tessellation will use the `vertices`
  /// as-is.
  pub fn new<'a, W, I>(mode: Mode, vertices: W, indices: I) -> Self
      where TessVertices<'a, V>: From<W>,
            V: 'a + Vertex,
            I: Into<Option<&'a[u32]>> {
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
      set_vertex_pointers(&V::vertex_format());

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
          _v: PhantomData
        }
      } else {
        gl::BindVertexArray(0);

        Tess {
          mode: opengl_mode(mode),
          vert_nb: vert_nb,
          vao: vao,
          vbo: Some(raw_vbo),
          ibo: None,
          _v: PhantomData
        }
      }
    }
  }

  /// Render the tessellation by providing the number of vertices to pick from it and how many
  /// instances are wished.
  fn render(&self, start_index: usize, vert_nb: usize, inst_nb: usize) {
    let vert_nb = vert_nb as GLsizei;
    let inst_nb = inst_nb as GLsizei;

    unsafe {
      gl::BindVertexArray(self.vao);

      if self.ibo.is_some() { // indexed render
        let first = (size_of::<u32>() * start_index) as *const c_void;

        if inst_nb == 1 {
          gl::DrawElements(self.mode, vert_nb, gl::UNSIGNED_INT, first);
        } else if inst_nb > 1 {
          gl::DrawElementsInstanced(self.mode, vert_nb, gl::UNSIGNED_INT, first, inst_nb);
        } else {
          panic!("cannot index-render 0 instance");
        }
      } else { // direct render
        let first = start_index as GLint;

        if inst_nb == 1 {
          gl::DrawArrays(self.mode, first, vert_nb);
        } else if inst_nb > 1 {
          gl::DrawArraysInstanced(self.mode, first, vert_nb, inst_nb);
        } else {
          panic!("cannot render 0 instance");
        }
      }
    }
  }

  /// Get an immutable slice over the vertices stored on GPU.
  pub fn as_slice(&self) -> Result<BufferSlice<V>, TessMapError> {
    self.vbo.as_ref()
      .ok_or(TessMapError::ForbiddenAttributelessMapping)
      .and_then(|raw| RawBuffer::as_slice(raw).map_err(TessMapError::VertexBufferMapFailed))
  }

  /// Get a mutable slice over the vertices stored on GPU.
  pub fn as_slice_mut(&mut self) -> Result<BufferSliceMut<V>, TessMapError> {
    self.vbo.as_mut()
      .ok_or(TessMapError::ForbiddenAttributelessMapping)
      .and_then(|raw| RawBuffer::as_slice_mut(raw).map_err(TessMapError::VertexBufferMapFailed))
  }
}

impl Tess<()> {
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
        _v: PhantomData
      }
    }
  }
}

impl<V> Drop for Tess<V> {
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
        gl::VertexAttribPointer(i as GLuint, dim_as_size(&f.dim), opengl_sized_type(&f), gl::FALSE, stride, ptr::null::<c_void>().offset(off as isize));
      }
    },
    Type::Integral | Type::Unsigned | Type::Boolean => {
      unsafe {
        gl::VertexAttribIPointer(i as GLuint, dim_as_size(&f.dim), opengl_sized_type(&f), stride, ptr::null::<c_void>().offset(off as isize));
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
pub struct TessRender<'a, V> where V: 'a {
  /// Tessellation to render.
  tess: &'a Tess<V>,
  /// Start index (vertex) in the tessellation.
  start_index: usize,
  /// Number of vertices to pick from the tessellation. If `None`, all of them are selected.
  vert_nb: usize,
  /// Number of instances to render.
  inst_nb: usize
}

impl<'a, V> TessRender<'a, V> {
  /// Create a tessellation render that will render the whole input tessellation with only one
  /// instance.
  pub fn one_whole(tess: &'a Tess<V>) -> Self {
    TessRender {
      tess: tess,
      start_index: 0,
      vert_nb: tess.vert_nb,
      inst_nb: 1
    }
  }

  /// Create a tessellation render for a part of the tessellation starting at the beginning of its
  /// buffer with only one instance.
  ///
  /// The part is selected by giving the number of vertices to render.
  ///
  /// > Note: if you also need to use an arbitrary part of your tessellation (not starting at the
  /// > first vertex in its buffer), have a look at `TessRender::one_slice`.
  ///
  /// # Panic
  ///
  /// Panic if the number of vertices is higher to the capacity of the tessellation’s vertex buffer.
  pub fn one_sub(tess: &'a Tess<V>, vert_nb: usize) -> Self {
    if vert_nb > tess.vert_nb {
      panic!("cannot render {} vertices for a tessellation which vertex capacity is {}", vert_nb, tess.vert_nb);
    }

    TessRender {
      tess: tess,
      start_index: 0,
      vert_nb: vert_nb,
      inst_nb: 1
    }
  }

  /// Create a tessellation render for a slice of the tessellation starting anywhere in its buffer
  /// with only one instance.
  ///
  /// The part is selected by giving the start vertex and the number of vertices to render. This
  ///
  /// # Panic
  ///
  /// Panic if the start vertex is higher to the capacity of the tessellation’s vertex buffer.
  ///
  /// Panic if the number of vertices is higher to the capacity of the tessellation’s vertex buffer.
  pub fn one_slice(tess: &'a Tess<V>, start: usize, nb: usize) -> Self {
    if start > tess.vert_nb {
      panic!("cannot render {} vertices starting at vertex {} for a tessellation which vertex capacity is {}", nb, start, tess.vert_nb);
    }

    if nb > tess.vert_nb {
      panic!("cannot render {} vertices for a tessellation which vertex capacity is {}", nb, tess.vert_nb);
    }

    TessRender {
      tess: tess,
      start_index: start,
      vert_nb: nb,
      inst_nb: 1
    }
  }

  /// Render a tessellation.
  pub fn render(&self) where V: Vertex {
    self.tess.render(self.start_index, self.vert_nb, self.inst_nb);
  }
}

impl<'a, V> From<&'a Tess<V>> for TessRender<'a, V> {
  fn from(tess: &'a Tess<V>) -> Self {
    TessRender::one_whole(tess)
  }
}
