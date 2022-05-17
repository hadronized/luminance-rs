//! Vertex sets.
//!
//! [`Tess`] is a type that represents the gathering of vertices and the way to connect / link
//! them. A [`Tess`] has several intrinsic properties:
//!
//! - Its _primitive mode_ — [`Mode`]. That object tells the GPU how to connect the vertices.
//! - A default number of vertex to render. When passing the [`Tess`] to the GPU for rendering,
//!   it’s possible to specify the number of vertices to render or just let the [`Tess`] render
//!   a default number of vertices (typically, the whole [`Tess`]).
//! - A default number of _instances_, which allows for geometry instancing. Geometry instancing
//!   is the fact of drawing with the same [`Tess`] several times, only changing the
//!   instance index every time a new render is performed. This is done entirely on the backend to
//!   prevent bandwidth exhaustion. The index of the instance, in the shader stages, is often used
//!   to pick material properties, matrices, etc. to customize each instances. Instances can manually
//!   be asked when using a [`TessView`].
//! - An indexed configuration, allowing to tell the GPU how to render the vertices by referring to
//!   them via indices.
//! - For indexed configuration, an optional _primitive restart index_ can be specified. That
//!   index, when present in the indexed set, will make some primitive modes _“restart”_ and create
//!   new primitives. More on this on the documentation of [`Mode`].
//!
//! # Tessellation creation
//!
//! [`Tess`] is not created directly. Instead, you need to use a [`TessBuilder`]. Tessellation
//! builders make it easy to customize what a [`Tess`] will be made of before actually requesting
//! the GPU to create them. They support a large number of possible situations:
//!
//! - _Attributeless_: when you only specify the [`Mode`] and number of vertices to render (and
//!   optionally the number of instances). That will create a vertex set with no vertex data. Your
//!   vertex shader will be responsible for creating the vertex attributes on the fly.
//! - _Direct geometry_: when you pass vertices directly.
//! - _Indexed geometry_: when you pass vertices and reference from with indices.
//! - _Instanced geometry_: when you ask to use instances, making the graphics pipeline create
//!   several instances of your vertex set on the GPU.
//!
//! # Tessellation views
//!
//! Once you have a [`Tess`] — created from [`TessBuilder::build`], you can now render it in a
//! [`TessGate`]. In order to do so, you need a [`TessView`].
//!
//! A [`TessView`] is a temporary _view_ into a [`Tess`], describing what part of it should be
//! drawn. It is also responsible in providing the number of instances to draw.
//! Creating [`TessView`]s is a cheap operation, and can be done in two different ways:
//!
//! - By directly using the methods from [`TessView`].
//! - By using the [`View`] trait.
//!
//! The [`View`] trait is a convenient way to create [`TessView`]. It provides the
//! [`View::view`] and [`View::inst_view`] (for instanced rendering) methods, which accept Rust’s
//! range operators to create the [`TessView`]s in a more comfortable way.
//!
//! # Tessellation mapping
//!
//! Sometimes, you will want to edit tessellations in a dynamic way instead of re-creating new
//! ones. That can be useful for streaming data of for using a small part of a big [`Tess`]. The
//! [`Tess`] type has several methods to obtain subparts, allow you to map values and iterate over
//! them via standard Rust slices. See these for further details:
//!
//! - [`Tess::vertices`] [`Tess::vertices_mut`] to map tessellations’ vertices.
//! - [`Tess::indices`] [`Tess::indices_mut`] to map tessellations’ indices.
//! - [`Tess::instances`] [`Tess::instances_mut`] to map tessellations’ instances.
//!
//! > Note: because of their slice nature, mapping a tessellation (vertices, indices or instances)
//! > will not help you with resizing a [`Tess`], as this is not currently supported. Creating a large
//! > enough [`Tess`] is preferable for now.
//!
//! [`TessGate`]: crate::tess_gate::TessGate

use crate::{
  backend::tess::{
    IndexSlice as IndexSliceBackend, InstanceSlice as InstanceSliceBackend, Tess as TessBackend,
    VertexSlice as VertexSliceBackend,
  },
  context::GraphicsContext,
  vertex::{Deinterleave, Vertex, VertexDesc},
};
use std::{
  error, fmt,
  marker::PhantomData,
  ops::{Deref, DerefMut, Range, RangeFrom, RangeFull, RangeInclusive, RangeTo, RangeToInclusive},
};

/// Primitive mode.
///
/// Some modes allow for _primitive restart_. Primitive restart is a cool feature that allows to
/// _break_ the building of a primitive to _start over again_. For instance, when making a curve,
/// you can imagine gluing segments next to each other. If at some point, you want to start a new
/// curve, you have two choices:
///
///   - Either you stop your draw call and make another one.
///   - Or you just use the _primitive restart_ feature to ask to create another line from scratch.
///
/// _Primitive restart_ should be used as much as possible as it will decrease the number of GPU
/// commands you have to issue.
///
/// > Deprecation notice: the next version of luminance will not support setting the primitive restart index: you will
/// then must provide the maximum value of index type.
///
/// That feature is encoded with a special _vertex index_. You can setup the value of the _primitive
/// restart index_ with [`TessBuilder::set_primitive_restart_index`]. Whenever a vertex index is set
/// to the same value as the _primitive restart index_, the value is not interpreted as a vertex
/// index but just a marker / hint to start a new primitive.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
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
  /// This mode is a bit different from [`Mode::TriangleFan`]. The first two vertices define the
  /// first edge of the first triangle. Then, for each new vertex, a new triangle is created by
  /// taking the very previous vertex and the last to very previous vertex. What it means is that
  /// every time a triangle is created, the next vertex will share the edge that was created to
  /// spawn the previous triangle.
  ///
  /// This mode is useful to create long ribbons / strips of triangles.
  ///
  /// > This kind of primitive mode allows the usage of _primitive restart_.
  TriangleStrip,

  /// A general purpose primitive with _n_ vertices, for use in tessellation shaders.
  /// For example, `Mode::Patch(3)` represents triangle patches, so every three vertices in the
  /// buffer form a patch.
  ///
  /// If you want to employ tessellation shaders, this is the only primitive mode you can use.
  Patch(usize),
}

impl fmt::Display for Mode {
  fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
    match *self {
      Mode::Point => f.write_str("point"),
      Mode::Line => f.write_str("line"),
      Mode::LineStrip => f.write_str("line strip"),
      Mode::Triangle => f.write_str("triangle"),
      Mode::TriangleStrip => f.write_str("triangle strip"),
      Mode::TriangleFan => f.write_str("triangle fan"),
      Mode::Patch(ref n) => write!(f, "patch ({})", n),
    }
  }
}

/// Error that can occur while trying to map GPU tessellations to host code.
#[non_exhaustive]
#[derive(Debug, Eq, PartialEq)]
pub enum TessMapError {
  /// Cannot obtain a slice on the backend.
  CannotMap,
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

impl TessMapError {
  /// Cannot obtain a slice on the backend.
  pub fn cannot_map() -> Self {
    TessMapError::CannotMap
  }

  /// Vertex target type is not the same as the one stored in the buffer.
  pub fn vertex_type_mismatch(a: VertexDesc, b: VertexDesc) -> Self {
    TessMapError::VertexTypeMismatch(a, b)
  }

  /// Index target type is not the same as the one stored in the buffer.
  pub fn index_type_mismatch(a: TessIndexType, b: TessIndexType) -> Self {
    TessMapError::IndexTypeMismatch(a, b)
  }

  /// The CPU mapping failed because you cannot map an attributeless tessellation since it doesn’t
  /// have any vertex attribute.
  pub fn forbidden_attributeless_mapping() -> Self {
    TessMapError::ForbiddenAttributelessMapping
  }

  /// The CPU mapping failed because currently, mapping deinterleaved buffers is not supported via
  /// a single slice.
  pub fn forbidden_deinterleaved_mapping() -> Self {
    TessMapError::ForbiddenDeinterleavedMapping
  }
}

impl fmt::Display for TessMapError {
  fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
    match *self {
      TessMapError::CannotMap => f.write_str("cannot map on the backend"),

      TessMapError::VertexTypeMismatch(ref a, ref b) => write!(
        f,
        "cannot map tessellation: vertex type mismatch between {:?} and {:?}",
        a, b
      ),

      TessMapError::IndexTypeMismatch(ref a, ref b) => write!(
        f,
        "cannot map tessellation: index type mismatch between {:?} and {:?}",
        a, b
      ),

      TessMapError::ForbiddenAttributelessMapping => {
        f.write_str("cannot map an attributeless buffer")
      }

      TessMapError::ForbiddenDeinterleavedMapping => {
        f.write_str("cannot map a deinterleaved buffer as interleaved")
      }
    }
  }
}

impl error::Error for TessMapError {}

/// Possible errors that might occur when dealing with [`Tess`].
#[non_exhaustive]
#[derive(Debug, Eq, PartialEq)]
pub enum TessError {
  /// Cannot create a tessellation.
  CannotCreate(String),
  /// Error related to attributeless tessellation and/or render.
  AttributelessError(String),
  /// Length incoherency in vertex, index or instance buffers.
  LengthIncoherency(usize),
  /// Forbidden primitive mode by hardware.
  ForbiddenPrimitiveMode(Mode),
  /// No data provided and empty tessellation.
  NoData,
}

impl TessError {
  /// Cannot create a tessellation.
  pub fn cannot_create(e: impl Into<String>) -> Self {
    TessError::CannotCreate(e.into())
  }

  /// Error related to attributeless tessellation and/or render.
  pub fn attributeless_error(e: impl Into<String>) -> Self {
    TessError::AttributelessError(e.into())
  }

  /// Length incoherency in vertex, index or instance buffers.
  pub fn length_incoherency(len: usize) -> Self {
    TessError::LengthIncoherency(len)
  }

  /// Forbidden primitive mode by hardware.
  pub fn forbidden_primitive_mode(mode: Mode) -> Self {
    TessError::ForbiddenPrimitiveMode(mode)
  }

  /// No data or empty tessellation.
  pub fn no_data() -> Self {
    TessError::NoData
  }
}

impl fmt::Display for TessError {
  fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
    match *self {
      TessError::CannotCreate(ref s) => write!(f, "Creation error: {}", s),
      TessError::AttributelessError(ref s) => write!(f, "Attributeless error: {}", s),
      TessError::LengthIncoherency(ref s) => {
        write!(f, "Incoherent size for internal buffers: {}", s)
      }
      TessError::ForbiddenPrimitiveMode(ref e) => write!(f, "forbidden primitive mode: {}", e),
      TessError::NoData => f.write_str("no data or empty tessellation"),
    }
  }
}

impl error::Error for TessError {}

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
  /// Get the number of bytes that are needed to represent a type described by the variant.
  pub fn bytes(self) -> usize {
    match self {
      TessIndexType::U8 => 1,
      TessIndexType::U16 => 2,
      TessIndexType::U32 => 4,
    }
  }
}

/// Class of tessellation indices.
///
/// Values which types implement this trait are allowed to be used to index tessellation in *indexed
/// draw commands*.
///
/// You shouldn’t have to worry too much about that trait. Have a look at the current implementors
/// for an exhaustive list of types you can use.
///
/// > Implementing this trait is `unsafe`.
pub unsafe trait TessIndex: Copy {
  /// Type of the underlying index.
  ///
  /// You are limited in which types you can use as indexes. Feel free to have a look at the
  /// documentation of the [`TessIndexType`] trait for further information.
  ///
  /// `None` means that you disable indexing.
  const INDEX_TYPE: Option<TessIndexType>;

  /// Get and convert the index to [`u32`], if possible.
  fn try_into_u32(self) -> Option<u32>;
}

unsafe impl TessIndex for () {
  const INDEX_TYPE: Option<TessIndexType> = None;

  fn try_into_u32(self) -> Option<u32> {
    None
  }
}

/// Boop.
unsafe impl TessIndex for u8 {
  const INDEX_TYPE: Option<TessIndexType> = Some(TessIndexType::U8);

  fn try_into_u32(self) -> Option<u32> {
    Some(self.into())
  }
}

/// Boop.
unsafe impl TessIndex for u16 {
  const INDEX_TYPE: Option<TessIndexType> = Some(TessIndexType::U16);

  fn try_into_u32(self) -> Option<u32> {
    Some(self.into())
  }
}

/// Wuuuuuuha.
unsafe impl TessIndex for u32 {
  const INDEX_TYPE: Option<TessIndexType> = Some(TessIndexType::U32);

  fn try_into_u32(self) -> Option<u32> {
    Some(self.into())
  }
}

/// Interleaved memory marker.
#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq)]
pub enum Interleaved {}

/// Deinterleaved memory marker.
#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq)]
pub enum Deinterleaved {}

/// Vertex input data of a [`TessBuilder`].
///
/// This trait defines the _storage_ of vertices that a [`TessBuilder`] will use to build its internal storage on the
/// backend.
///
/// There are two implementors of this trait:
///
/// - `impl<V> TessVertexData<Interleaved> for V where V: Vertex`
/// - `impl<V> TessVertexData<Deinterleaved> for V where V: Vertex`
///
/// For the situation where `S` is [`Interleaved`], this trait associates the data (with [`TessVertexData::Data`]) to be
/// a `Vec<V>`. What it means is that the [`TessBuilder`] will build the vertices as a `Vec<V>`, where `V: Vertex`,
/// implementing an _interleaved memory layout_.
///
/// For the situation where `S` is [`Deinterleaved`], this trait associates the data to be a `Vec<DeinterleavedData>`.
/// [`DeinterleavedData`] is a special type used to store a collection of one of the attributes of a `V: Vertex`. For
/// instance, if `V: Vertex` has two attributes, vertices will end up in two [`DeinterleavedData`]: the first one for
/// the first attribute, the second one for the second attribute. The [`TessBuilder`] will handle that logic for you
/// when you will use the [`TessBuilder::set_vertices`] by tracking at the type-level which set of attributes you are setting.
///
/// # Parametricity
///
/// - `S` is the storage marker. It will be set to either [`Interleaved`] or [`Deinterleaved`].
pub trait TessVertexData<S>: Vertex
where
  S: ?Sized,
{
  /// Vertex storage type.
  type Data;

  /// Coherent length of the vertices.
  ///
  /// Vertices length can be incoherent for some implementations of [`TessVertexData::Data`],
  /// especially with deinterleaved memory. For this reason, this method can fail with [`TessError`].
  fn coherent_len(data: &Self::Data) -> Result<usize, TessError>;
}

impl<V> TessVertexData<Interleaved> for V
where
  V: Vertex,
{
  type Data = Vec<V>;

  fn coherent_len(data: &Self::Data) -> Result<usize, TessError> {
    Ok(data.len())
  }
}

impl<V> TessVertexData<Deinterleaved> for V
where
  V: Vertex,
{
  type Data = Vec<DeinterleavedData>;

  fn coherent_len(data: &Self::Data) -> Result<usize, TessError> {
    if data.is_empty() {
      Ok(0)
    } else {
      let len = data[0].len;

      if data[1..].iter().any(|a| a.len != len) {
        Err(TessError::length_incoherency(len))
      } else {
        Ok(len)
      }
    }
  }
}

/// Deinterleaved data.
///
/// [`DeinterleavedData`] represents a collection of one type of attributes of a set of vertices, for each vertex
/// implements [`Vertex`]. End-users shouldn’t need to know about this type as it’s only used internally.
#[derive(Debug, Clone)]
pub struct DeinterleavedData {
  raw: Vec<u8>,
  len: usize,
}

impl DeinterleavedData {
  fn new() -> Self {
    DeinterleavedData {
      raw: Vec::new(),
      len: 0,
    }
  }

  /// Turn the [`DeinterleavedData`] into its raw representation.
  pub fn into_vec(self) -> Vec<u8> {
    self.raw
  }
}

/// [`Tess`] builder object.
///
/// This type allows to create [`Tess`] via a _builder pattern_. You have several flavors of
/// possible _vertex storages_, as well as _data encoding_, described below.
///
/// # Vertex storage
///
/// ## Interleaved
///
/// You can pass around interleaved vertices and indices. Those are encoded in `Vec<T>`. You
/// typically want to use this when you already have the vertices and/or indices allocated somewhere,
/// as the interface will use the input vector as a source of truth for lengths.
///
/// ## Deinterleaved
///
/// This is the same as interleaved data in terms of interface, but the `T` type is interpreted
/// a bit differently. Here, the encoding is `(Vec<Field0>, Vec<Field1>, …)`, where `Field0`,
/// `Field1` etc. are all the ordered fieds in `T`. This logic is hidden behind `Vec<DeinterleavedData>`.
///
/// That representation allows field-based operations on [`Tess`], while it would be impossible
/// with the interleaved version (you would need to get all the fields at once, since
/// you would work on `T` directly and each of its fields).
///
/// # Data encoding
///
/// - Vectors: you can pass vectors as input data for both vertices and indices. Those will be
///   interpreted differently based on the vertex storage you chose for vertices / instances. For indices, there is no
///   difference.
/// - Disabled: disabling means that no data will be passed to the GPU. You can disable independently
///   vertex data and/or index data by using the unit `()` type.
///
/// # Indexed vertex sets
///
/// It is possible to _index_ the geometry via the use of indices. Indices are stored in contiguous
/// regions of memory (`Vec<T>`), where `T` satisfies [`TessIndex`]. When using an indexed tessellation,
/// the meaning of its attributes slightly changes. First, the vertices are not used as input source for
/// the vertex stream. In order to provide vertices that will go through the vertex stream, the indices
/// reference the vertex set to provide the order in which they should appear in the stream.
///
/// When rendering with a [`TessView`], the number of vertices to render must be provided or inferred
/// based on the [`Tess`] the view was made from. That number will refer to either the vertex set or
/// index set, depending on the kind of tessellation. Asking to render a [`Tess`] with 3 vertices will
/// pick 3 vertices from the vertex set for direct tessellations and 3 indices to index the vertex set
/// for indexed tessellations.
///
/// # Primitive mode
///
/// By default, a [`TessBuilder`] will build _points_. Each vertex in the vertex stream will be independently rendered
/// from the others, resulting in a _point cloud_. This logic is encoded with [`Mode::Point`]. You can change how
/// vertices are interpreted by changing the [`Mode`].
///
/// # Parametricity
///
/// - `B` is the backend type
/// - `V` is the vertex type.
/// - `I` is the index type.
/// - `W` is the vertex instance type.
/// - `S` is the storage type.
#[derive(Debug)]
pub struct TessBuilder<'a, B, V, I = (), W = (), S = Interleaved>
where
  B: ?Sized,
  V: TessVertexData<S>,
  W: TessVertexData<S>,
  S: ?Sized,
{
  backend: &'a mut B,
  vertex_data: Option<V::Data>,
  index_data: Vec<I>,
  instance_data: Option<W::Data>,
  mode: Mode,
  render_vert_nb: usize,
  render_inst_nb: usize,
  restart_index: Option<I>,
  _phantom: PhantomData<&'a mut ()>,
}

impl<'a, B, V, I, W, S> TessBuilder<'a, B, V, I, W, S>
where
  B: ?Sized,
  V: TessVertexData<S>,
  I: TessIndex,
  W: TessVertexData<S>,
  S: ?Sized,
{
  /// Set the [`Mode`] to connect vertices.
  ///
  /// Calling that function twice replaces the previously set value.
  pub fn set_mode(mut self, mode: Mode) -> Self {
    self.mode = mode;
    self
  }

  /// Set the default number of vertices to render.
  ///
  /// Calling that function twice replaces the previously set value. This method changes the number of vertices to pick:
  ///
  /// - From the vertex set for regular geometries.
  /// - From the index set, using the picked indices to reference the vertex set.
  pub fn set_render_vertex_nb(mut self, vert_nb: usize) -> Self {
    self.render_vert_nb = vert_nb;
    self
  }

  /// Set the default number of instances to render.
  ///
  /// Calling that function twice replaces the previously set value.
  pub fn set_render_instance_nb(mut self, inst_nb: usize) -> Self {
    self.render_inst_nb = inst_nb;
    self
  }

  /// Set the primitive restart index.
  ///
  /// Calling that function twice replaces the previously set value.
  pub fn set_primitive_restart_index(mut self, restart_index: I) -> Self {
    self.restart_index = Some(restart_index);
    self
  }
}

impl<'a, B, V, I, W, S> TessBuilder<'a, B, V, I, W, S>
where
  B: ?Sized,
  V: TessVertexData<S>,
  I: TessIndex,
  W: TessVertexData<S>,
  S: ?Sized,
{
  /// Create a new default [`TessBuilder`].
  ///
  /// # Notes
  ///
  /// Feel free to use the [`GraphicsContext::new_tess`] method for a simpler method.
  ///
  /// [`GraphicsContext::new_tess`]: crate::context::GraphicsContext::new_tess
  pub fn new<C>(ctx: &'a mut C) -> Self
  where
    C: GraphicsContext<Backend = B>,
  {
    TessBuilder {
      backend: ctx.backend(),
      vertex_data: None,
      index_data: Vec::new(),
      instance_data: None,
      mode: Mode::Point,
      render_vert_nb: 0,
      render_inst_nb: 0,
      restart_index: None,
      _phantom: PhantomData,
    }
  }
}

// set_indices, which works only if I = ()
impl<'a, B, V, W, S> TessBuilder<'a, B, V, (), W, S>
where
  B: ?Sized,
  V: TessVertexData<S>,
  W: TessVertexData<S>,
  S: ?Sized,
{
  /// Add indices to be bundled in the [`Tess`].
  ///
  /// Every time you call that function, the set of indices is replaced by the one you provided.
  /// The type of expected indices is ruled by the `II` type variable you chose.
  pub fn set_indices<I, X>(self, indices: X) -> TessBuilder<'a, B, V, I, W, S>
  where
    X: Into<Vec<I>>,
  {
    TessBuilder {
      backend: self.backend,
      vertex_data: self.vertex_data,
      index_data: indices.into(),
      instance_data: self.instance_data,
      mode: self.mode,
      render_vert_nb: self.render_vert_nb,
      render_inst_nb: self.render_inst_nb,
      restart_index: None,
      _phantom: PhantomData,
    }
  }
}

// set_vertices, interleaved version; works only for V = ()
impl<'a, B, I, W> TessBuilder<'a, B, (), I, W, Interleaved>
where
  B: ?Sized,
  I: TessIndex,
  W: TessVertexData<Interleaved>,
{
  /// Add vertices to be bundled in the [`Tess`].
  ///
  /// Every time you call that function, the set of vertices is replaced by the one you provided.
  pub fn set_vertices<V, X>(self, vertices: X) -> TessBuilder<'a, B, V, I, W, Interleaved>
  where
    X: Into<Vec<V>>,
    V: TessVertexData<Interleaved, Data = Vec<V>>,
  {
    TessBuilder {
      backend: self.backend,
      vertex_data: Some(vertices.into()),
      index_data: self.index_data,
      instance_data: self.instance_data,
      mode: self.mode,
      render_vert_nb: self.render_vert_nb,
      render_inst_nb: self.render_inst_nb,
      restart_index: self.restart_index,
      _phantom: PhantomData,
    }
  }
}

impl<'a, B, I, V> TessBuilder<'a, B, V, I, (), Interleaved>
where
  B: ?Sized,
  I: TessIndex,
  V: TessVertexData<Interleaved>,
{
  /// Add instances to be bundled in the [`Tess`].
  ///
  /// Every time you call that function, the set of instances is replaced by the one you provided.
  pub fn set_instances<W, X>(self, instances: X) -> TessBuilder<'a, B, V, I, W, Interleaved>
  where
    X: Into<Vec<W>>,
    W: TessVertexData<Interleaved, Data = Vec<W>>,
  {
    TessBuilder {
      backend: self.backend,
      vertex_data: self.vertex_data,
      index_data: self.index_data,
      instance_data: Some(instances.into()),
      mode: self.mode,
      render_vert_nb: self.render_vert_nb,
      render_inst_nb: self.render_inst_nb,
      restart_index: self.restart_index,
      _phantom: PhantomData,
    }
  }
}

impl<'a, B, V, I, W> TessBuilder<'a, B, V, I, W, Deinterleaved>
where
  B: ?Sized,
  V: TessVertexData<Deinterleaved, Data = Vec<DeinterleavedData>>,
  I: TessIndex,
  W: TessVertexData<Deinterleaved, Data = Vec<DeinterleavedData>>,
{
  /// Add vertices to be bundled in the [`Tess`].
  ///
  /// Every time you call that function, the set of vertices is replaced by the one you provided.
  pub fn set_attributes<A, X>(mut self, attributes: X) -> Self
  where
    X: Into<Vec<A>>,
    V: Deinterleave<A>,
  {
    let build_raw = |deinterleaved: &mut Vec<DeinterleavedData>| {
      // turn the attribute into a raw vector (Vec<u8>)
      let boxed_slice = attributes.into().into_boxed_slice();
      let len = boxed_slice.len();
      let len_bytes = len * std::mem::size_of::<A>();
      let ptr = Box::into_raw(boxed_slice);
      // please Dog pardon me
      let raw = unsafe { Vec::from_raw_parts(ptr as _, len_bytes, len_bytes) };

      deinterleaved[V::RANK] = DeinterleavedData { raw, len };
    };

    match self.vertex_data {
      Some(ref mut deinterleaved) => {
        build_raw(deinterleaved);
      }

      None => {
        let attrs = V::vertex_desc();
        let mut deinterleaved = vec![DeinterleavedData::new(); attrs.len()];
        build_raw(&mut deinterleaved);

        self.vertex_data = Some(deinterleaved);
      }
    }

    self
  }

  /// Add instances to be bundled in the [`Tess`].
  ///
  /// Every time you call that function, the set of instances is replaced by the one you provided.
  pub fn set_instance_attributes<A, X>(mut self, attributes: X) -> Self
  where
    X: Into<Vec<A>>,
    W: Deinterleave<A>,
  {
    let build_raw = |deinterleaved: &mut Vec<DeinterleavedData>| {
      // turn the attribute into a raw vector (Vec<u8>)
      let boxed_slice = attributes.into().into_boxed_slice();
      let len = boxed_slice.len();
      let len_bytes = len * std::mem::size_of::<A>();
      let ptr = Box::into_raw(boxed_slice);
      // please Dog pardon me
      let raw = unsafe { Vec::from_raw_parts(ptr as _, len_bytes, len_bytes) };

      deinterleaved[W::RANK] = DeinterleavedData { raw, len };
    };

    match self.instance_data {
      None => {
        let attrs = W::vertex_desc();
        let mut deinterleaved = vec![DeinterleavedData::new(); attrs.len()];
        build_raw(&mut deinterleaved);

        self.instance_data = Some(deinterleaved);
      }

      Some(ref mut deinterleaved) => {
        build_raw(deinterleaved);
      }
    }

    self
  }
}

impl<'a, B, V, I, W, S> TessBuilder<'a, B, V, I, W, S>
where
  B: ?Sized + TessBackend<V, I, W, S>,
  V: TessVertexData<S>,
  I: TessIndex,
  W: TessVertexData<S>,
{
  /// Build a [`Tess`] if the [`TessBuilder`] has enough data and is in a valid state. What is
  /// needed is backend-dependent but most of the time, you will want to:
  ///
  /// - Set a [`Mode`].
  /// - Give vertex data and optionally indices, or give none of them but only a number of vertices
  ///   (attributeless objects).
  /// - If you provide vertex data by submitting several sets with [`TessBuilder::set_attributes`]
  ///   and/or [`TessBuilder::set_instances`], do not forget that you must submit sets with the
  ///   same size. Otherwise, the GPU will not know what values use for missing attributes in
  ///   vertices.
  pub fn build(self) -> Result<Tess<B, V, I, W, S>, TessError> {
    // validate input data before giving it to the backend
    let render_vert_nb = self.guess_render_vertex_len()?;
    let render_inst_nb = self.guess_render_instance_len()?;

    unsafe {
      self
        .backend
        .build(
          self.vertex_data,
          self.index_data,
          self.instance_data,
          self.mode,
          self.restart_index,
        )
        .map(|repr| Tess {
          repr,
          render_vert_nb,
          render_inst_nb,
          _phantom: PhantomData,
        })
    }
  }

  /// Guess how many vertices we want to render by default.
  fn guess_render_vertex_len(&self) -> Result<usize, TessError> {
    // if we don’t have an explicit number of vertex to render, we rely on the vertex data coherent
    // length
    if self.render_vert_nb == 0 {
      // if we don’t have index data, get the length from the vertex data; otherwise, get it from
      // the index data
      if self.index_data.is_empty() {
        match self.vertex_data {
          Some(ref data) => V::coherent_len(data),
          None => Err(TessError::NoData),
        }
      } else {
        Ok(self.index_data.len())
      }
    } else {
      // ensure the length is okay regarding what we have in the index / vertex data
      if self.index_data.is_empty() {
        match self.vertex_data {
          Some(ref data) => {
            let coherent_len = V::coherent_len(data)?;

            if self.render_vert_nb <= coherent_len {
              Ok(self.render_vert_nb)
            } else {
              Err(TessError::length_incoherency(self.render_vert_nb))
            }
          }

          // attributeless render, always accept
          None => Ok(self.render_vert_nb),
        }
      } else {
        if self.render_vert_nb <= self.index_data.len() {
          Ok(self.render_vert_nb)
        } else {
          Err(TessError::length_incoherency(self.render_vert_nb))
        }
      }
    }
  }

  fn guess_render_instance_len(&self) -> Result<usize, TessError> {
    // as with vertex length, we first check for an explicit number, and if none, we deduce it
    if self.render_inst_nb == 0 {
      match self.instance_data {
        Some(ref data) => W::coherent_len(data),
        None => Ok(0),
      }
    } else {
      let coherent_len = self
        .instance_data
        .as_ref()
        .ok_or_else(|| TessError::attributeless_error("missing number of instances"))
        .and_then(W::coherent_len)?;

      if self.render_inst_nb <= coherent_len {
        Ok(self.render_inst_nb)
      } else {
        Err(TessError::length_incoherency(self.render_inst_nb))
      }
    }
  }
}

/// A GPU vertex set.
///
/// Vertex set are the only way to represent space data. The dimension you choose is up to you, but
/// people will typically want to represent objects in 2D or 3D. A _vertex_ is a point in such
/// space and it carries _properties_ — called _“vertex attributes_”. Those attributes are
/// completely free to use. They must, however, be compatible with the [`Semantics`] and [`Vertex`]
/// traits.
///
/// [`Tess`] are built with a [`TessBuilder`] and can be _sliced_ to edit their content in-line —
/// by mapping the GPU memory region and access data via slices.
///
/// [`Semantics`]: crate::vertex::Semantics
/// [`TessGate`]: crate::tess_gate::TessGate
#[derive(Debug)]
pub struct Tess<B, V, I = (), W = (), S = Interleaved>
where
  B: ?Sized + TessBackend<V, I, W, S>,
  V: TessVertexData<S>,
  I: TessIndex,
  W: TessVertexData<S>,
  S: ?Sized,
{
  // backend representation of the tessellation
  pub(crate) repr: B::TessRepr,

  // default number of vertices to render
  render_vert_nb: usize,

  // default number of instances to render
  render_inst_nb: usize,

  _phantom: PhantomData<*const S>,
}

impl<B, V, I, W, S> Tess<B, V, I, W, S>
where
  B: ?Sized + TessBackend<V, I, W, S>,
  V: TessVertexData<S>,
  I: TessIndex,
  W: TessVertexData<S>,
  S: ?Sized,
{
  /// Get the number of vertices.
  pub fn vert_nb(&self) -> usize {
    unsafe { B::tess_vertices_nb(&self.repr) }
  }

  /// Get the number of vertex indices.
  pub fn idx_nb(&self) -> usize {
    unsafe { B::tess_indices_nb(&self.repr) }
  }

  /// Get the number of instances.
  pub fn inst_nb(&self) -> usize {
    unsafe { B::tess_instances_nb(&self.repr) }
  }

  /// Default number of vertices to render.
  ///
  /// This number represents the number of vertices that will be rendered when not explicitly asked to render a given
  /// amount of vertices.
  pub fn render_vert_nb(&self) -> usize {
    self.render_vert_nb
  }

  /// Default number of vertex instances to render.
  ///
  /// This number represents the number of vertex instances that will be rendered when not explicitly asked to render a
  /// given amount of instances.
  pub fn render_inst_nb(&self) -> usize {
    self.render_inst_nb
  }

  /// Slice the [`Tess`] in order to read its content via usual slices.
  ///
  /// This method gives access to the underlying _index storage_.
  pub fn indices<'a>(&'a mut self) -> Result<Indices<'a, B, V, I, W, S>, TessMapError>
  where
    B: IndexSliceBackend<'a, V, I, W, S>,
  {
    unsafe { B::indices(&mut self.repr).map(|repr| Indices { repr }) }
  }

  /// Slice the [`Tess`] in order to read its content via usual slices.
  ///
  /// This method gives access to the underlying _index storage_.
  pub fn indices_mut<'a>(&'a mut self) -> Result<IndicesMut<'a, B, V, I, W, S>, TessMapError>
  where
    B: IndexSliceBackend<'a, V, I, W, S>,
  {
    unsafe { B::indices_mut(&mut self.repr).map(|repr| IndicesMut { repr }) }
  }
}

impl<B, V, I, W> Tess<B, V, I, W, Interleaved>
where
  B: ?Sized + TessBackend<V, I, W, Interleaved>,
  V: TessVertexData<Interleaved>,
  I: TessIndex,
  W: TessVertexData<Interleaved>,
{
  /// Slice the [`Tess`] in order to read its content via usual slices.
  ///
  /// This method gives access to the underlying _vertex storage_.
  pub fn vertices<'a>(
    &'a mut self,
  ) -> Result<Vertices<'a, B, V, I, W, Interleaved, V>, TessMapError>
  where
    B: VertexSliceBackend<'a, V, I, W, Interleaved, V>,
  {
    unsafe { B::vertices(&mut self.repr).map(|repr| Vertices { repr }) }
  }

  /// Slice the [`Tess`] in order to read its content via usual slices.
  ///
  /// This method gives access to the underlying _vertex storage_.
  pub fn vertices_mut<'a>(
    &'a mut self,
  ) -> Result<VerticesMut<'a, B, V, I, W, Interleaved, V>, TessMapError>
  where
    B: VertexSliceBackend<'a, V, I, W, Interleaved, V>,
  {
    unsafe { B::vertices_mut(&mut self.repr).map(|repr| VerticesMut { repr }) }
  }

  /// Slice the [`Tess`] in order to read its content via usual slices.
  ///
  /// This method gives access to the underlying _instance storage_.
  pub fn instances<'a>(
    &'a mut self,
  ) -> Result<Instances<'a, B, V, I, W, Interleaved, W>, TessMapError>
  where
    B: InstanceSliceBackend<'a, V, I, W, Interleaved, W>,
  {
    unsafe { B::instances(&mut self.repr).map(|repr| Instances { repr }) }
  }

  /// Slice the [`Tess`] in order to read its content via usual slices.
  ///
  /// This method gives access to the underlying _instance storage_.
  pub fn instances_mut<'a>(
    &'a mut self,
  ) -> Result<InstancesMut<'a, B, V, I, W, Interleaved, W>, TessMapError>
  where
    B: InstanceSliceBackend<'a, V, I, W, Interleaved, W>,
  {
    unsafe { B::instances_mut(&mut self.repr).map(|repr| InstancesMut { repr }) }
  }
}

impl<B, V, I, W> Tess<B, V, I, W, Deinterleaved>
where
  B: ?Sized + TessBackend<V, I, W, Deinterleaved>,
  V: TessVertexData<Deinterleaved>,
  I: TessIndex,
  W: TessVertexData<Deinterleaved>,
{
  /// Slice the [`Tess`] in order to read its content via usual slices.
  ///
  /// This method gives access to the underlying _vertex storage_.
  pub fn vertices<'a, T>(
    &'a mut self,
  ) -> Result<Vertices<'a, B, V, I, W, Deinterleaved, T>, TessMapError>
  where
    B: VertexSliceBackend<'a, V, I, W, Deinterleaved, T>,
    V: Deinterleave<T>,
  {
    unsafe { B::vertices(&mut self.repr).map(|repr| Vertices { repr }) }
  }

  /// Slice the [`Tess`] in order to read its content via usual slices.
  ///
  /// This method gives access to the underlying _vertex storage_.
  pub fn vertices_mut<'a, T>(
    &'a mut self,
  ) -> Result<VerticesMut<'a, B, V, I, W, Deinterleaved, T>, TessMapError>
  where
    B: VertexSliceBackend<'a, V, I, W, Deinterleaved, T>,
    V: Deinterleave<T>,
  {
    unsafe { B::vertices_mut(&mut self.repr).map(|repr| VerticesMut { repr }) }
  }

  /// Slice the [`Tess`] in order to read its content via usual slices.
  ///
  /// This method gives access to the underlying _instance storage_.
  pub fn instances<'a, T>(
    &'a mut self,
  ) -> Result<Instances<'a, B, V, I, W, Deinterleaved, T>, TessMapError>
  where
    B: InstanceSliceBackend<'a, V, I, W, Deinterleaved, T>,
    W: Deinterleave<T>,
  {
    unsafe { B::instances(&mut self.repr).map(|repr| Instances { repr }) }
  }

  /// Slice the [`Tess`] in order to read its content via usual slices.
  ///
  /// This method gives access to the underlying _instance storage_.
  pub fn instances_mut<'a, T>(
    &'a mut self,
  ) -> Result<InstancesMut<'a, B, V, I, W, Deinterleaved, T>, TessMapError>
  where
    B: InstanceSliceBackend<'a, V, I, W, Deinterleaved, T>,
    W: Deinterleave<T>,
  {
    unsafe { B::instances_mut(&mut self.repr).map(|repr| InstancesMut { repr }) }
  }
}

/// TODO
#[derive(Debug)]
pub struct Vertices<'a, B, V, I, W, S, T>
where
  B: ?Sized + TessBackend<V, I, W, S> + VertexSliceBackend<'a, V, I, W, S, T>,
  V: TessVertexData<S>,
  I: TessIndex,
  W: TessVertexData<S>,
  S: ?Sized,
{
  repr: B::VertexSliceRepr,
}

impl<'a, B, V, I, W, S, T> Deref for Vertices<'a, B, V, I, W, S, T>
where
  B: ?Sized + TessBackend<V, I, W, S> + VertexSliceBackend<'a, V, I, W, S, T>,
  V: TessVertexData<S>,
  I: TessIndex,
  W: TessVertexData<S>,
  S: ?Sized,
{
  type Target = [T];

  fn deref(&self) -> &Self::Target {
    self.repr.deref()
  }
}

/// TODO
#[derive(Debug)]
pub struct VerticesMut<'a, B, V, I, W, S, T>
where
  B: ?Sized + TessBackend<V, I, W, S> + VertexSliceBackend<'a, V, I, W, S, T>,
  V: TessVertexData<S>,
  I: TessIndex,
  W: TessVertexData<S>,
  S: ?Sized,
{
  repr: B::VertexSliceMutRepr,
}

impl<'a, B, V, I, W, S, T> Deref for VerticesMut<'a, B, V, I, W, S, T>
where
  B: ?Sized + TessBackend<V, I, W, S> + VertexSliceBackend<'a, V, I, W, S, T>,
  V: TessVertexData<S>,
  I: TessIndex,
  W: TessVertexData<S>,
  S: ?Sized,
{
  type Target = [T];

  fn deref(&self) -> &Self::Target {
    self.repr.deref()
  }
}

impl<'a, B, V, I, W, S, T> DerefMut for VerticesMut<'a, B, V, I, W, S, T>
where
  B: ?Sized + TessBackend<V, I, W, S> + VertexSliceBackend<'a, V, I, W, S, T>,
  V: TessVertexData<S>,
  I: TessIndex,
  W: TessVertexData<S>,
  S: ?Sized,
{
  fn deref_mut(&mut self) -> &mut Self::Target {
    self.repr.deref_mut()
  }
}

/// TODO
#[derive(Debug)]
pub struct Indices<'a, B, V, I, W, S>
where
  B: ?Sized + TessBackend<V, I, W, S> + IndexSliceBackend<'a, V, I, W, S>,
  V: TessVertexData<S>,
  I: TessIndex,
  W: TessVertexData<S>,
  S: ?Sized,
{
  repr: B::IndexSliceRepr,
}

impl<'a, B, V, I, W, S> Deref for Indices<'a, B, V, I, W, S>
where
  B: ?Sized + TessBackend<V, I, W, S> + IndexSliceBackend<'a, V, I, W, S>,
  V: TessVertexData<S>,
  I: TessIndex,
  W: TessVertexData<S>,
  S: ?Sized,
{
  type Target = [I];

  fn deref(&self) -> &Self::Target {
    self.repr.deref()
  }
}

/// TODO
#[derive(Debug)]
pub struct IndicesMut<'a, B, V, I, W, S>
where
  B: ?Sized + TessBackend<V, I, W, S> + IndexSliceBackend<'a, V, I, W, S>,
  V: TessVertexData<S>,
  I: TessIndex,
  W: TessVertexData<S>,
  S: ?Sized,
{
  repr: B::IndexSliceMutRepr,
}

impl<'a, B, V, I, W, S> Deref for IndicesMut<'a, B, V, I, W, S>
where
  B: ?Sized + TessBackend<V, I, W, S> + IndexSliceBackend<'a, V, I, W, S>,
  V: TessVertexData<S>,
  I: TessIndex,
  W: TessVertexData<S>,
  S: ?Sized,
{
  type Target = [I];

  fn deref(&self) -> &Self::Target {
    self.repr.deref()
  }
}

impl<'a, B, V, I, W, S> DerefMut for IndicesMut<'a, B, V, I, W, S>
where
  B: ?Sized + TessBackend<V, I, W, S> + IndexSliceBackend<'a, V, I, W, S>,
  V: TessVertexData<S>,
  I: TessIndex,
  W: TessVertexData<S>,
  S: ?Sized,
{
  fn deref_mut(&mut self) -> &mut Self::Target {
    self.repr.deref_mut()
  }
}

/// TODO
#[derive(Debug)]
pub struct Instances<'a, B, V, I, W, S, T>
where
  B: ?Sized + TessBackend<V, I, W, S> + InstanceSliceBackend<'a, V, I, W, S, T>,
  V: TessVertexData<S>,
  I: TessIndex,
  W: TessVertexData<S>,
  S: ?Sized,
{
  repr: B::InstanceSliceRepr,
}

impl<'a, B, V, I, W, S, T> Deref for Instances<'a, B, V, I, W, S, T>
where
  B: ?Sized + TessBackend<V, I, W, S> + InstanceSliceBackend<'a, V, I, W, S, T>,
  V: TessVertexData<S>,
  I: TessIndex,
  W: TessVertexData<S>,
  S: ?Sized,
{
  type Target = [T];

  fn deref(&self) -> &Self::Target {
    self.repr.deref()
  }
}

/// TODO
#[derive(Debug)]
pub struct InstancesMut<'a, B, V, I, W, S, T>
where
  B: ?Sized + TessBackend<V, I, W, S> + InstanceSliceBackend<'a, V, I, W, S, T>,
  V: TessVertexData<S>,
  I: TessIndex,
  W: TessVertexData<S>,
  S: ?Sized,
{
  repr: B::InstanceSliceMutRepr,
}

impl<'a, B, V, I, W, S, T> Deref for InstancesMut<'a, B, V, I, W, S, T>
where
  B: ?Sized + TessBackend<V, I, W, S> + InstanceSliceBackend<'a, V, I, W, S, T>,
  S: ?Sized,
  V: TessVertexData<S>,
  I: TessIndex,
  W: TessVertexData<S>,
{
  type Target = [T];

  fn deref(&self) -> &Self::Target {
    self.repr.deref()
  }
}

impl<'a, B, V, I, W, S, T> DerefMut for InstancesMut<'a, B, V, I, W, S, T>
where
  B: ?Sized + TessBackend<V, I, W, S> + InstanceSliceBackend<'a, V, I, W, S, T>,
  V: TessVertexData<S>,
  I: TessIndex,
  W: TessVertexData<S>,
  S: ?Sized,
{
  fn deref_mut(&mut self) -> &mut Self::Target {
    self.repr.deref_mut()
  }
}

/// Possible error that might occur while dealing with [`TessView`] objects.
#[non_exhaustive]
#[derive(Debug, Eq, PartialEq)]
pub enum TessViewError {
  /// The view has incorrect size.
  ///
  /// data.
  IncorrectViewWindow {
    /// Capacity of data in the [`Tess`].
    capacity: usize,
    /// Requested start.
    start: usize,
    /// Requested number.
    nb: usize,
  },
}

impl fmt::Display for TessViewError {
  fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
    match self {
      TessViewError::IncorrectViewWindow {
        capacity,
        start,
        nb,
      } => {
        write!(f, "TessView incorrect window error: requested slice size {} starting at {}, but capacity is only {}",
          nb, start, capacity)
      }
    }
  }
}

impl error::Error for TessViewError {}

/// A _view_ into a GPU tessellation.
#[derive(Clone)]
pub struct TessView<'a, B, V, I, W, S>
where
  B: ?Sized + TessBackend<V, I, W, S>,
  V: TessVertexData<S>,
  I: TessIndex,
  W: TessVertexData<S>,
  S: ?Sized,
{
  /// Tessellation to render.
  pub(crate) tess: &'a Tess<B, V, I, W, S>,
  /// Start index (vertex) in the tessellation.
  pub(crate) start_index: usize,
  /// Number of vertices to pick from the tessellation.
  pub(crate) vert_nb: usize,
  /// Number of instances to render.
  pub(crate) inst_nb: usize,
}

impl<'a, B, V, I, W, S> TessView<'a, B, V, I, W, S>
where
  B: ?Sized + TessBackend<V, I, W, S>,
  V: TessVertexData<S>,
  I: TessIndex,
  W: TessVertexData<S>,
  S: ?Sized,
{
  /// Create a view that is using the whole input [`Tess`].
  pub fn whole(tess: &'a Tess<B, V, I, W, S>) -> Self {
    TessView {
      tess,
      start_index: 0,
      vert_nb: tess.render_vert_nb(),
      inst_nb: tess.render_inst_nb(),
    }
  }

  /// Create a view that is using the whole input [`Tess`] with `inst_nb` instances.
  pub fn inst_whole(tess: &'a Tess<B, V, I, W, S>, inst_nb: usize) -> Self {
    TessView {
      tess,
      start_index: 0,
      vert_nb: tess.render_vert_nb(),
      inst_nb,
    }
  }

  /// Create a view that is using only a subpart of the input [`Tess`], starting from the beginning
  /// of the vertices.
  pub fn sub(tess: &'a Tess<B, V, I, W, S>, vert_nb: usize) -> Result<Self, TessViewError> {
    let capacity = tess.render_vert_nb();

    if vert_nb > capacity {
      return Err(TessViewError::IncorrectViewWindow {
        capacity,
        start: 0,
        nb: vert_nb,
      });
    }

    Ok(TessView {
      tess,
      start_index: 0,
      vert_nb,
      inst_nb: tess.render_inst_nb(),
    })
  }

  /// Create a view that is using only a subpart of the input [`Tess`], starting from the beginning
  /// of the vertices, with `inst_nb` instances.
  pub fn inst_sub(
    tess: &'a Tess<B, V, I, W, S>,
    vert_nb: usize,
    inst_nb: usize,
  ) -> Result<Self, TessViewError> {
    let capacity = tess.render_vert_nb();

    if vert_nb > capacity {
      return Err(TessViewError::IncorrectViewWindow {
        capacity,
        start: 0,
        nb: vert_nb,
      });
    }

    Ok(TessView {
      tess,
      start_index: 0,
      vert_nb,
      inst_nb,
    })
  }

  /// Create a view that is using only a subpart of the input [`Tess`], starting from `start`, with
  /// `nb` vertices.
  pub fn slice(
    tess: &'a Tess<B, V, I, W, S>,
    start: usize,
    nb: usize,
  ) -> Result<Self, TessViewError> {
    let capacity = tess.render_vert_nb();

    if start > capacity || nb + start > capacity {
      return Err(TessViewError::IncorrectViewWindow {
        capacity,
        start,
        nb,
      });
    }

    Ok(TessView {
      tess,
      start_index: start,
      vert_nb: nb,
      inst_nb: tess.render_inst_nb(),
    })
  }

  /// Create a view that is using only a subpart of the input [`Tess`], starting from `start`, with
  /// `nb` vertices and `inst_nb` instances.
  pub fn inst_slice(
    tess: &'a Tess<B, V, I, W, S>,
    start: usize,
    nb: usize,
    inst_nb: usize,
  ) -> Result<Self, TessViewError> {
    let capacity = tess.render_vert_nb();

    if start > capacity || nb + start > capacity {
      return Err(TessViewError::IncorrectViewWindow {
        capacity,
        start,
        nb,
      });
    }

    Ok(TessView {
      tess,
      start_index: start,
      vert_nb: nb,
      inst_nb,
    })
  }
}

impl<'a, B, V, I, W, S> From<&'a Tess<B, V, I, W, S>> for TessView<'a, B, V, I, W, S>
where
  B: ?Sized + TessBackend<V, I, W, S>,
  V: TessVertexData<S>,
  I: TessIndex,
  W: TessVertexData<S>,
  S: ?Sized,
{
  fn from(tess: &'a Tess<B, V, I, W, S>) -> Self {
    TessView::whole(tess)
  }
}

/// [`TessView`] helper trait.
///
/// This trait helps to create [`TessView`] by allowing using the Rust range operators, such as
///
/// - [`..`](https://doc.rust-lang.org/std/ops/struct.RangeFull.html); the full range operator.
/// - [`a .. b`](https://doc.rust-lang.org/std/ops/struct.Range.html); the range operator.
/// - [`a ..`](https://doc.rust-lang.org/std/ops/struct.RangeFrom.html); the range-from operator.
/// - [`.. b`](https://doc.rust-lang.org/std/ops/struct.RangeTo.html); the range-to operator.
/// - [`..= b`](https://doc.rust-lang.org/std/ops/struct.RangeToInclusive.html); the inclusive range-to operator.
pub trait View<B, V, I, W, S, Idx>
where
  B: ?Sized + TessBackend<V, I, W, S>,
  V: TessVertexData<S>,
  I: TessIndex,
  W: TessVertexData<S>,
  S: ?Sized,
{
  /// Slice a tessellation object and yields a [`TessView`] according to the index range.
  fn view(&self, idx: Idx) -> Result<TessView<B, V, I, W, S>, TessViewError>;

  /// Slice a tesselation object and yields a [`TessView`] according to the index range with as
  /// many instances as specified.
  fn inst_view(&self, idx: Idx, inst_nb: usize) -> Result<TessView<B, V, I, W, S>, TessViewError>;
}

impl<B, V, I, W, S> View<B, V, I, W, S, RangeFull> for Tess<B, V, I, W, S>
where
  B: ?Sized + TessBackend<V, I, W, S>,
  V: TessVertexData<S>,
  I: TessIndex,
  W: TessVertexData<S>,
  S: ?Sized,
{
  fn view(&self, _: RangeFull) -> Result<TessView<B, V, I, W, S>, TessViewError> {
    Ok(TessView::whole(self))
  }

  fn inst_view(
    &self,
    _: RangeFull,
    inst_nb: usize,
  ) -> Result<TessView<B, V, I, W, S>, TessViewError> {
    Ok(TessView::inst_whole(self, inst_nb))
  }
}

impl<B, V, I, W, S> View<B, V, I, W, S, RangeTo<usize>> for Tess<B, V, I, W, S>
where
  B: ?Sized + TessBackend<V, I, W, S>,
  V: TessVertexData<S>,
  I: TessIndex,
  W: TessVertexData<S>,
  S: ?Sized,
{
  fn view(&self, to: RangeTo<usize>) -> Result<TessView<B, V, I, W, S>, TessViewError> {
    TessView::sub(self, to.end)
  }

  fn inst_view(
    &self,
    to: RangeTo<usize>,
    inst_nb: usize,
  ) -> Result<TessView<B, V, I, W, S>, TessViewError> {
    TessView::inst_sub(self, to.end, inst_nb)
  }
}

impl<B, V, I, W, S> View<B, V, I, W, S, RangeFrom<usize>> for Tess<B, V, I, W, S>
where
  B: ?Sized + TessBackend<V, I, W, S>,
  V: TessVertexData<S>,
  I: TessIndex,
  W: TessVertexData<S>,
  S: ?Sized,
{
  fn view(&self, from: RangeFrom<usize>) -> Result<TessView<B, V, I, W, S>, TessViewError> {
    TessView::slice(self, from.start, self.render_vert_nb() - from.start)
  }

  fn inst_view(
    &self,
    from: RangeFrom<usize>,
    inst_nb: usize,
  ) -> Result<TessView<B, V, I, W, S>, TessViewError> {
    TessView::inst_slice(
      self,
      from.start,
      self.render_vert_nb() - from.start,
      inst_nb,
    )
  }
}

impl<B, V, I, W, S> View<B, V, I, W, S, Range<usize>> for Tess<B, V, I, W, S>
where
  B: ?Sized + TessBackend<V, I, W, S>,
  V: TessVertexData<S>,
  I: TessIndex,
  W: TessVertexData<S>,
  S: ?Sized,
{
  fn view(&self, range: Range<usize>) -> Result<TessView<B, V, I, W, S>, TessViewError> {
    TessView::slice(self, range.start, range.end - range.start)
  }

  fn inst_view(
    &self,
    range: Range<usize>,
    inst_nb: usize,
  ) -> Result<TessView<B, V, I, W, S>, TessViewError> {
    TessView::inst_slice(self, range.start, range.end - range.start, inst_nb)
  }
}

impl<B, V, I, W, S> View<B, V, I, W, S, RangeInclusive<usize>> for Tess<B, V, I, W, S>
where
  B: ?Sized + TessBackend<V, I, W, S>,
  V: TessVertexData<S>,
  I: TessIndex,
  W: TessVertexData<S>,
  S: ?Sized,
{
  fn view(&self, range: RangeInclusive<usize>) -> Result<TessView<B, V, I, W, S>, TessViewError> {
    let start = *range.start();
    let end = *range.end();
    TessView::slice(self, start, end - start + 1)
  }

  fn inst_view(
    &self,
    range: RangeInclusive<usize>,
    inst_nb: usize,
  ) -> Result<TessView<B, V, I, W, S>, TessViewError> {
    let start = *range.start();
    let end = *range.end();
    TessView::inst_slice(self, start, end - start + 1, inst_nb)
  }
}

impl<B, V, I, W, S> View<B, V, I, W, S, RangeToInclusive<usize>> for Tess<B, V, I, W, S>
where
  B: ?Sized + TessBackend<V, I, W, S>,
  V: TessVertexData<S>,
  I: TessIndex,
  W: TessVertexData<S>,
  S: ?Sized,
{
  fn view(&self, to: RangeToInclusive<usize>) -> Result<TessView<B, V, I, W, S>, TessViewError> {
    TessView::sub(self, to.end + 1)
  }

  fn inst_view(
    &self,
    to: RangeToInclusive<usize>,
    inst_nb: usize,
  ) -> Result<TessView<B, V, I, W, S>, TessViewError> {
    TessView::inst_sub(self, to.end + 1, inst_nb)
  }
}
