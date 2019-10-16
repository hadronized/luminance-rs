//! # GPU geometries.
//!
//! Tessellations (i.e. [`Tess`]) represent geometric information stored on GPU. They are at the
//! heart of any render, should it be 2D, 3D or even more exotic configuration. Please familiarize
//! yourself with the tessellation abstractions before going on.
//!
//! # Tessellation primitive
//!
//! Currently, several kinds of tessellation are supported:
//!
//! - [`Mode::Point`]; _point clouds_.
//! - [`Mode::Line`]; _lines_.
//! - [`Mode::LineStrip`]; _line strips_, which are lines connected between them to create a single,
//!   long line.
//! - [`Mode::Triangle`]; _triangles_.
//! - [`Mode::TriangleFan`]; _triangle fans_, a way of connecting triangles.
//! - [`Mode::TriangleStrip`]; _triangle strips_, another way of connecting triangles.
//!
//! Those kinds of tessellation are designated by the [`Mode`] type. You will also come across the
//! name of _primitive mode_ to designate such an idea.
//!
//! # Tessellation creation
//!
//! Creation is done via the [`TessBuilder`] type, using the _builder_ pattern. Once you’re done
//! with configuring everything, you can generate the tessellation and get a [`Tess`] object.
//!
//! [`Tess`] represents data on the GPU and can be thought of as an access to the actual data, a bit
//! in the same way as a [`Vec`] is just a small data structure that represents an access to a
//! much bigger memory area.
//!
//! # Tessellation render
//!
//! In order to render a [`Tess`], you have to use a [`TessSlice`] object. You’ll be able to use
//! that object in *pipelines*. See the [pipeline] module for further details.
//!
//! [`Mode`]: crate::tess::Mode
//! [`Mode::Point`]: crate::tess::Mode::Point
//! [`Mode::Line`]: crate::tess::Mode::Line
//! [`Mode::LineStrip`]: crate::tess::Mode::LineStrip
//! [`Mode::Triangle`]: crate::tess::Mode::Triangle
//! [`Mode::TriangleFan`]: crate::tess::Mode::TriangleFan
//! [`Mode::TriangleStrip`]: crate::tess::Mode::TriangleStrip
//! [`BufferSlice`]: crate::buffer::BufferSlice
//! [`BufferSliceMut`]: crate::buffer::BufferSliceMut
//! [`Tess`]: crate::tess::Tess
//! [`Tess::as_slice`]: crate::tess::Tess::as_slice
//! [`Tess::as_slice_mut`]: crate::tess::Tess::as_slice_mut
//! [`TessBuilder`]: crate::tess::TessBuilder
//! [`TessSlice`]: crate::tess::TessSlice
//! [pipeline]: crate::pipeline

use crate::buffer::Buffer;
use crate::vertex::Vertex;

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
pub trait TessBuilder<'a, C>: Sized {
  type Tess;

  type Err;

  /// Create a new, default [`TessBuilder`].
  fn new(ctx: &'a mut C) -> Self;

  /// Add vertices to be part of the tessellation.
  ///
  /// This method can be used in several ways. First, you can decide to use interleaved memory, in
  /// which case you will call this method only once by providing an interleaved slice / borrowed
  /// buffer. Second, you can opt-in to use deinterleaved memory, in which case you will have
  /// several, smaller buffers of borrowed data and you will issue a call to this method for all of
  /// them.
  fn add_vertices<V, W>(self, vertices: W) -> Self where W: AsRef<[V]>, V: Vertex;

  /// Add instances to be part of the tessellation.
  fn add_instances<V, W>(self, instances: W) -> Self where W: AsRef<[V]>, V: Vertex;

  /// Set vertex indices in order to specify how vertices should be picked by the GPU pipeline.
  fn set_indices<T, I>(self, indices: T) -> Self where T: AsRef<[I]>, I: TessIndex;

  /// Set the primitive mode for the building [`Tess`].
  fn set_mode(self, mode: Mode) -> Self;

  /// Set the default number of vertices to be rendered.
  ///
  /// That function is not mandatory if you are not building an _attributeless_ tessellation but is
  /// if you are.
  ///
  /// When called while building a [`Tess`] owning at least one vertex buffer, it acts as a _default_
  /// number of vertices to render and is useful when you will slice the tessellation with open
  /// ranges.
  fn set_vertex_nb(self, nb: usize) -> Self;

  /// Set the default number of instances to render.
  ///
  /// `0` disables geometry instancing.
  fn set_instance_nb(self, nb: usize) -> Self;

  /// Set the primitive restart index. The initial value is `None`, implying no primitive restart.
  fn set_primitive_restart_index(self, index: Option<u32>) -> Self;

  /// Build the [`Tess`].
  fn build(self) -> Result<Self::Tess, Self::Err>;
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
pub trait Tess<C> {
  type Err;

  /// Render the tessellation.
  fn render(&self, ctx: &mut C, start_index: usize, vert_nb: usize, inst_nb: usize);
}

pub trait VertexSlice<'a, C, B, V>: Tess<C> where V: Vertex, B: Buffer<'a, C, V> {
  /// Obtain a slice over the vertex buffer.
  fn as_slice(&'a mut self) -> Result<B::Slice, Self::Err>;
}

pub trait VertexSliceMut<'a, C, B, V>: Tess<C> where V: Vertex, B: Buffer<'a, C, V> {
  /// Obtain a mutable slice over the vertex buffer.
  ///
  /// This function fails if you try to obtain a buffer from an attriteless [`Tess`] or
  /// deinterleaved memory.
  fn as_slice_mut(&'a mut self) -> Result<B::SliceMut, Self::Err>;
}

pub trait IndexSlice<'a, C, B, I>: Tess<C> where I: TessIndex, B: Buffer<'a, C, I> {
  /// Obtain a slice over the index buffer.
  ///
  /// This function fails if you try to obtain a buffer from an attriteless [`Tess`] or if no
  /// index buffer is available.
  fn as_index_slice(&'a mut self) -> Result<B::Slice, Self::Err>;
}

pub trait IndexSliceMut<'a, C, B, I>: Tess<C> where I: TessIndex, B: Buffer<'a, C, I> {
  /// Obtain a mutable slice over the index buffer.
  ///
  /// This function fails if you try to obtain a buffer from an attriteless [`Tess`] or if no
  /// index buffer is available.
  fn as_index_slice_mut(&'a mut self) -> Result<B::SliceMut, Self::Err>;
}

pub trait InstanceSlice<'a, C, B, V>: Tess<C> where V: Vertex, B: Buffer<'a, C, V> {
  /// Obtain a slice over the instance buffer.
  ///
  /// This function fails if you try to obtain a buffer from an attriteless [`Tess`] or
  /// deinterleaved memory.
  fn as_instance_slice(&'a mut self) -> Result<B::Slice, Self::Err>;
}

pub trait InstanceSliceMut<'a, C, B, V>: Tess<C> where V: Vertex, B: Buffer<'a, C, V> {
  /// Obtain a slice over the instance buffer.
  ///
  /// This function fails if you try to obtain a buffer from an attriteless [`Tess`] or
  /// deinterleaved memory.
  fn as_instance_slice_mut(&'a mut self) -> Result<B::SliceMut, Self::Err>;
}

/// Vertices can be connected via several modes.
///
/// Some modes allow for _primitive restart_. Primitive restart is a cool feature that allows to
/// _break_ the building of a primitive to _start over again_. For instance, when making a curve,
/// you can imagine gluing segments next to each other. If at some point, you want to start a new
/// line, you have two choices:
///
///   - Either you stop your draw call and make another one.
///   - Or you just use the _primitive restart_ feature to ask to create another line from scratch.
///
/// That feature is encoded with a special _vertex index_. You can setup the value of the _primitive
/// restart index_ with [`TessBuilder::set_primitive_restart_index`]. Whenever a vertex index is set
/// to the same value as the _primitive restart index_, the value is not interpreted as a vertex
/// index but just a marker / hint to start a new primitive.
#[derive(Copy, Clone, Debug)]
pub enum Mode {
  /// A single point.
  ///
  /// Points are left unconnected from each other and represent a _point cloud_. This is the typical
  /// primitive mode you want to do, for instance, particles rendering.
  Point,
  /// A line, defined by two points.
  ///
  /// Every pair of vertices are connected together to form a straight line.
  Line,
  /// A strip line, defined by at least two points and zero or many other ones.
  ///
  /// The first two vertices create a line, and every new vertex flowing in the graphics pipeline
  /// (starting from the third, then) well extend the initial line, making a curve composed of
  /// several segments.
  ///
  /// > This kind of primitive mode allows the usage of _primitive restart_.
  LineStrip,
  /// A triangle, defined by three points.
  Triangle,
  /// A triangle fan, defined by at least three points and zero or many other ones.
  ///
  /// Such a mode is easy to picture: a cooling fan is a circular shape, with blades.
  /// [`Mode::TriangleFan`] is kind of the same. The first vertex is at the center of the fan, then
  /// the second vertex creates the first edge of the first triangle. Every time you add a new
  /// vertex, a triangle is created by taking the first (center) vertex, the very previous vertex
  /// and the current vertex. By specifying vertices around the center, you actually create a
  /// fan-like shape.
  ///
  /// > This kind of primitive mode allows the usage of _primitive restart_.
  TriangleFan,
  /// A triangle strip, defined by at least three points and zero or many other ones.
  ///
  /// This mode is a bit different from [`Mode::TriangleStrip`]. The first two vertices define the
  /// first edge of the first triangle. Then, for each new vertex, a new triangle is created by
  /// taking the very previous vertex and the last to very previous vertex. What it means is that
  /// every time a triangle is created, the next vertex will share the edge that was created to
  /// spawn the previous triangle.
  ///
  /// This mode is useful to create long ribbons / strips of triangles.
  ///
  /// > This kind of primitive mode allows the usage of _primitive restart_.
  TriangleStrip,
}

/// Possible tessellation index types.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum TessIndexType {
  /// 8-bit unsigned integer.
  U8,
  /// 16-bit unsigned integer.
  U16,
  /// 32-bit unsigned integer.
  U32,
}

impl TessIndexType {
  pub fn bytes(self) -> usize {
    match self {
      TessIndexType::U8 => 1,
      TessIndexType::U16 => 2,
      TessIndexType::U32 => 4,
    }
  }
}

/// Class of tessellation indexes.
///
/// Values which types implement this trait are allowed to be used to index tessellation in *indexed
/// draw commands*.
///
/// You shouldn’t have to worry to much about that trait. Have a look at the current implementors
/// for an exhaustive list of types you can use.
///
/// > Implementing this trait is `unsafe`.
pub unsafe trait TessIndex {
  /// Type of the underlying index.
  ///
  /// You are limited in which types you can use as indexes. Feel free to have a look at the
  /// documentation of the [`TessIndexType`] trait for further information.
  const INDEX_TYPE: TessIndexType;
}

unsafe impl TessIndex for u8 {
  const INDEX_TYPE: TessIndexType = TessIndexType::U8;
}

unsafe impl TessIndex for u16 {
  const INDEX_TYPE: TessIndexType = TessIndexType::U16;
}

unsafe impl TessIndex for u32 {
  const INDEX_TYPE: TessIndexType = TessIndexType::U32;
}

// /// Tessellation slice.
// ///
// /// This type enables slicing a tessellation on the fly so that we can render patches of it.
// /// Typically, you can obtain a slice by using the [`TessSliceIndex`] trait (the
// /// [`TessSliceIndex::slice`] method) and combining it with some Rust range operators, such as
// /// [`..`] or [`..=`].
// ///
// /// [`..`]: https://doc.rust-lang.org/std/ops/struct.RangeFull.html
// /// [`..=`]: https://doc.rust-lang.org/std/ops/struct.RangeInclusive.html
// //pub trait TessSlice
