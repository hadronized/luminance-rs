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
//!   is the fact of drawing with the same [`Tess`] (GPU buffers) several times, only changing the
//!   instance index every time a new render is performed. This is done entirely on the GPU to
//!   prevent bandwidth exhaustion. The index of the instance, in the shader stages, is often used
//!   to pick material properties, matrices, etc. to customize each instances.
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
//! drawn. Creating [`TessView`]s is a cheap operation, and can be done in two different ways:
//!
//! - By directly using the methods from [`TessView`].
//! - By using the [`View`] trait.
//!
//! The [`View`] trait is a convenient way to create [`TessView`]. It provides the
//! [`View::view`] and [`View::inst_view`] methods, which accept Rust’s range operators
//! to create the [`TessView`]s in a more comfortable way.
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
//! > will not help you with resizing a [`Tess`], as this is not currently supported.
//!
//! [`TessGate`]: crate::tess_gate::TessGate

use std::error;
use std::fmt;
use std::marker::PhantomData;
use std::ops::{
  Deref, DerefMut, Range, RangeFrom, RangeFull, RangeInclusive, RangeTo, RangeToInclusive,
};

use crate::backend::tess::{
  IndexSlice as IndexSliceBackend, InstanceSlice as InstanceSliceBackend, Tess as TessBackend,
  VertexSlice as VertexSliceBackend,
};
use crate::buffer::BufferError;
use crate::context::GraphicsContext;
use crate::vertex::{Deinterleave, Vertex, VertexDesc};

/// Vertices can be connected via several modes.
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
  /// The CPU mapping failed due to buffer errors.
  BufferMapError(BufferError),
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
  /// The CPU mapping failed due to buffer errors.
  pub fn buffer_map_error(e: BufferError) -> Self {
    TessMapError::BufferMapError(e)
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
      TessMapError::BufferMapError(ref e) => write!(f, "cannot map tessellation buffer: {}", e),
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

impl From<BufferError> for TessMapError {
  fn from(e: BufferError) -> Self {
    TessMapError::buffer_map_error(e)
  }
}

impl error::Error for TessMapError {
  fn source(&self) -> Option<&(dyn error::Error + 'static)> {
    match self {
      TessMapError::BufferMapError(e) => Some(e),
      _ => None,
    }
  }
}

/// Possible errors that might occur when dealing with [`Tess`].
#[non_exhaustive]
#[derive(Debug)]
pub enum TessError {
  /// Cannot create a tessellation.
  CannotCreate(String),
  /// Error related to attributeless tessellation and/or render.
  AttributelessError(String),
  /// Length incoherency in vertex, index or instance buffers.
  LengthIncoherency(usize),
  /// Internal error ocurring with a buffer.
  InternalBufferError(BufferError),
  /// Forbidden primitive mode by hardware.
  ForbiddenPrimitiveMode(Mode),
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

  /// Internal error ocurring with a buffer.
  pub fn internal_buffer_error(e: BufferError) -> Self {
    TessError::InternalBufferError(e)
  }

  /// Forbidden primitive mode by hardware.
  pub fn forbidden_primitive_mode(mode: Mode) -> Self {
    TessError::ForbiddenPrimitiveMode(mode)
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
      TessError::InternalBufferError(ref e) => write!(f, "internal buffer error: {}", e),
      TessError::ForbiddenPrimitiveMode(ref e) => write!(f, "forbidden primitive mode: {}", e),
    }
  }
}

impl From<BufferError> for TessError {
  fn from(e: BufferError) -> Self {
    TessError::internal_buffer_error(e)
  }
}

impl error::Error for TessError {
  fn source(&self) -> Option<&(dyn error::Error + 'static)> {
    match self {
      TessError::InternalBufferError(e) => Some(e),
      _ => None,
    }
  }
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
    Some(self)
  }
}

/// Interleaved memory marker.
#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq)]
pub enum Interleaved {}

/// Deinterleaved memory marker.
#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq)]
pub enum Deinterleaved {}

/// Vertex input data of a [`TessBuilder`].
pub trait TessVertexData<S>: Vertex
where
  S: ?Sized,
{
  /// Vertex storage type.
  type Data;

  /// Coherent length of the vertices.
  ///
  /// Vertices length can be incohent for some implementations of [`TessVertexData::Data`],
  /// especially with deinterleaved memory.
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
/// possible _vertex storage_ situations, as well as _data encoding_, described below.
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
/// `Field1` etc. are all the ordered fieds in `T`.
///
/// That representation allows field-based operation later on [`Tess`], while it would be
/// impossible with the interleaved version (you would need to get all the fields at once, since
/// you would work on`T` directly and each of its fields).
///
/// # Data encoding
///
/// - Vectors: you can pass vectors as input data for both vertices and indices. Those will be
///   interpreted differently based on the vertex storage you chose for vertices, and the normal
///   way for indices.
/// - Buffers: you can pass [`Buffer`] objects, too. Those are more flexible than vectors as you can
///   use all of the [`Buffer`] API before sending them to the builder.
/// - Disabled: disabling means that no data will be passed to the GPU. You can disable independently
///   vertex data and/or index data.
///
/// # Parametricity
///
/// - `B` is the backend type
/// - `V` is the vertex type.
/// - `S` is the storage type.
///
/// [`Buffer`]: crate::buffer::Buffer
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
  vert_nb: usize,
  inst_nb: usize,
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
  /// Calling that function twice replace the previously set value.
  pub fn set_mode(mut self, mode: Mode) -> Self {
    self.mode = mode;
    self
  }

  /// Set the default number of vertices to render.
  ///
  /// Calling that function twice replace the previously set value.
  pub fn set_vertex_nb(mut self, vert_nb: usize) -> Self {
    self.vert_nb = vert_nb;
    self
  }

  /// Set the default number of instances to render.
  ///
  /// Calling that function twice replace the previously set value.
  pub fn set_instance_nb(mut self, inst_nb: usize) -> Self {
    self.inst_nb = inst_nb;
    self
  }

  /// Set the primitive restart index.
  ///
  /// Calling that function twice replace the previously set value.
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
      vert_nb: 0,
      inst_nb: 0,
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
      vert_nb: self.vert_nb,
      inst_nb: self.inst_nb,
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
      vert_nb: self.vert_nb,
      inst_nb: self.inst_nb,
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
      vert_nb: self.vert_nb,
      inst_nb: self.inst_nb,
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
        let mut deinterleaved = vec![DeinterleavedData::new(); V::ATTR_COUNT];
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
        let mut deinterleaved = vec![DeinterleavedData::new(); W::ATTR_COUNT];
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
  /// - Give vertex data and optionally indices, or give none of them (attributeless objects).
  /// - If you provide vertex data by submitting several sets with [`TessBuilder::set_attributes`]
  ///   and/or [`TessBuilder::set_instances`], do not forget that you must submit sets with the
  ///   same size. Otherwise, the GPU will not know what values use for missing attributes in
  ///   vertices.
  pub fn build(self) -> Result<Tess<B, V, I, W, S>, TessError> {
    // validate input data before giving it to the backend
    let vert_nb = self.guess_vertex_len()?;
    let inst_nb = self.guess_instance_len()?;

    unsafe {
      self
        .backend
        .build(
          (self.vertex_data, vert_nb),
          (self.instance_data, inst_nb),
          self.index_data,
          self.restart_index,
          self.mode,
        )
        .map(|repr| Tess {
          repr,
          _phantom: PhantomData,
        })
    }
  }

  fn guess_vertex_len(&self) -> Result<usize, TessError> {
    // if we don’t have an explicit number of vertex to render, we rely on the vertex data coherent
    // length
    if self.vert_nb == 0 {
      // if we don’t have index data, get the length from the vertex data; otherwise, get it from
      // the index data
      if self.index_data.is_empty() {
        match self.vertex_data {
          Some(ref data) => V::coherent_len(data),
          None => Ok(0),
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

            if self.vert_nb <= coherent_len {
              Ok(self.vert_nb)
            } else {
              Err(TessError::length_incoherency(self.vert_nb))
            }
          }

          None => Ok(self.vert_nb),
        }
      } else if self.vert_nb <= self.index_data.len() {
        Ok(self.vert_nb)
      } else {
        Err(TessError::length_incoherency(self.vert_nb))
      }
    }
  }

  fn guess_instance_len(&self) -> Result<usize, TessError> {
    // as with vertex length, we first check for an explicit number, and if none, we deduce it
    if self.inst_nb == 0 {
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

      if self.inst_nb <= coherent_len {
        Ok(self.inst_nb)
      } else {
        Err(TessError::length_incoherency(self.inst_nb))
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
/// [`Tess`] are built out of [`TessBuilder`] and can be _sliced_ to edit their content in-line —
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
  pub(crate) repr: B::TessRepr,
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

  /// Get the number of indices.
  pub fn inst_nb(&self) -> usize {
    unsafe { B::tess_instances_nb(&self.repr) }
  }

  /// Slice the [`Tess`] in order to read its content via usual slices.
  ///
  /// This method gives access to the underlying _index storage_.
  pub fn indices(&mut self) -> Result<Indices<B, V, I, W, S>, TessMapError>
  where
    B: IndexSliceBackend<V, I, W, S>,
  {
    unsafe { B::indices(&mut self.repr).map(|repr| Indices { repr }) }
  }

  /// Slice the [`Tess`] in order to read its content via usual slices.
  ///
  /// This method gives access to the underlying _index storage_.
  pub fn indices_mut(&mut self) -> Result<IndicesMut<B, V, I, W, S>, TessMapError>
  where
    B: IndexSliceBackend<V, I, W, S>,
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
  pub fn vertices(&mut self) -> Result<Vertices<B, V, I, W, Interleaved, V>, TessMapError>
  where
    B: VertexSliceBackend<V, I, W, Interleaved, V>,
  {
    unsafe { B::vertices(&mut self.repr).map(|repr| Vertices { repr }) }
  }

  /// Slice the [`Tess`] in order to read its content via usual slices.
  ///
  /// This method gives access to the underlying _vertex storage_.
  pub fn vertices_mut(&mut self) -> Result<VerticesMut<B, V, I, W, Interleaved, V>, TessMapError>
  where
    B: VertexSliceBackend<V, I, W, Interleaved, V>,
  {
    unsafe { B::vertices_mut(&mut self.repr).map(|repr| VerticesMut { repr }) }
  }

  /// Slice the [`Tess`] in order to read its content via usual slices.
  ///
  /// This method gives access to the underlying _instance storage_.
  pub fn instances(&mut self) -> Result<Instances<B, V, I, W, Interleaved, V>, TessMapError>
  where
    B: InstanceSliceBackend<V, I, W, Interleaved, V>,
  {
    unsafe { B::instances(&mut self.repr).map(|repr| Instances { repr }) }
  }

  /// Slice the [`Tess`] in order to read its content via usual slices.
  ///
  /// This method gives access to the underlying _instance storage_.
  pub fn instances_mut(&mut self) -> Result<InstancesMut<B, V, I, W, Interleaved, V>, TessMapError>
  where
    B: InstanceSliceBackend<V, I, W, Interleaved, V>,
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
  pub fn vertices<T>(&mut self) -> Result<Vertices<B, V, I, W, Deinterleaved, T>, TessMapError>
  where
    B: VertexSliceBackend<V, I, W, Deinterleaved, T>,
    V: Deinterleave<T>,
  {
    unsafe { B::vertices(&mut self.repr).map(|repr| Vertices { repr }) }
  }

  /// Slice the [`Tess`] in order to read its content via usual slices.
  ///
  /// This method gives access to the underlying _vertex storage_.
  pub fn vertices_mut<T>(
    &mut self,
  ) -> Result<VerticesMut<B, V, I, W, Deinterleaved, T>, TessMapError>
  where
    B: VertexSliceBackend<V, I, W, Deinterleaved, T>,
    V: Deinterleave<T>,
  {
    unsafe { B::vertices_mut(&mut self.repr).map(|repr| VerticesMut { repr }) }
  }

  /// Slice the [`Tess`] in order to read its content via usual slices.
  ///
  /// This method gives access to the underlying _instance storage_.
  pub fn instances<T>(&mut self) -> Result<Instances<B, V, I, W, Deinterleaved, T>, TessMapError>
  where
    B: InstanceSliceBackend<V, I, W, Deinterleaved, T>,
    W: Deinterleave<T>,
  {
    unsafe { B::instances(&mut self.repr).map(|repr| Instances { repr }) }
  }

  /// Slice the [`Tess`] in order to read its content via usual slices.
  ///
  /// This method gives access to the underlying _instance storage_.
  pub fn instances_mut<T>(
    &mut self,
  ) -> Result<InstancesMut<B, V, I, W, Deinterleaved, T>, TessMapError>
  where
    B: InstanceSliceBackend<V, I, W, Deinterleaved, T>,
    W: Deinterleave<T>,
  {
    unsafe { B::instances_mut(&mut self.repr).map(|repr| InstancesMut { repr }) }
  }
}

/// TODO
#[derive(Debug)]
pub struct Vertices<B, V, I, W, S, T>
where
  B: ?Sized + TessBackend<V, I, W, S> + VertexSliceBackend<V, I, W, S, T>,
  V: TessVertexData<S>,
  I: TessIndex,
  W: TessVertexData<S>,
  S: ?Sized,
{
  repr: B::VertexSliceRepr,
}

impl<B, V, I, W, S, T> Deref for Vertices<B, V, I, W, S, T>
where
  B: ?Sized + TessBackend<V, I, W, S> + VertexSliceBackend<V, I, W, S, T>,
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
pub struct VerticesMut<B, V, I, W, S, T>
where
  B: ?Sized + TessBackend<V, I, W, S> + VertexSliceBackend<V, I, W, S, T>,
  V: TessVertexData<S>,
  I: TessIndex,
  W: TessVertexData<S>,
  S: ?Sized,
{
  repr: B::VertexSliceMutRepr,
}

impl<B, V, I, W, S, T> Deref for VerticesMut<B, V, I, W, S, T>
where
  B: ?Sized + TessBackend<V, I, W, S> + VertexSliceBackend<V, I, W, S, T>,
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

impl<B, V, I, W, S, T> DerefMut for VerticesMut<B, V, I, W, S, T>
where
  B: ?Sized + TessBackend<V, I, W, S> + VertexSliceBackend<V, I, W, S, T>,
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
pub struct Indices<B, V, I, W, S>
where
  B: ?Sized + TessBackend<V, I, W, S> + IndexSliceBackend<V, I, W, S>,
  V: TessVertexData<S>,
  I: TessIndex,
  W: TessVertexData<S>,
  S: ?Sized,
{
  repr: B::IndexSliceRepr,
}

impl<B, V, I, W, S> Deref for Indices<B, V, I, W, S>
where
  B: ?Sized + TessBackend<V, I, W, S> + IndexSliceBackend<V, I, W, S>,
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
pub struct IndicesMut<B, V, I, W, S>
where
  B: ?Sized + TessBackend<V, I, W, S> + IndexSliceBackend<V, I, W, S>,
  V: TessVertexData<S>,
  I: TessIndex,
  W: TessVertexData<S>,
  S: ?Sized,
{
  repr: B::IndexSliceMutRepr,
}

impl<B, V, I, W, S> Deref for IndicesMut<B, V, I, W, S>
where
  B: ?Sized + TessBackend<V, I, W, S> + IndexSliceBackend<V, I, W, S>,
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

impl<B, V, I, W, S> DerefMut for IndicesMut<B, V, I, W, S>
where
  B: ?Sized + TessBackend<V, I, W, S> + IndexSliceBackend<V, I, W, S>,
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
pub struct Instances<B, V, I, W, S, T>
where
  B: ?Sized + TessBackend<V, I, W, S> + InstanceSliceBackend<V, I, W, S, T>,
  V: TessVertexData<S>,
  I: TessIndex,
  W: TessVertexData<S>,
  S: ?Sized,
{
  repr: B::InstanceSliceRepr,
}

impl<B, V, I, W, S, T> Deref for Instances<B, V, I, W, S, T>
where
  B: ?Sized + TessBackend<V, I, W, S> + InstanceSliceBackend<V, I, W, S, T>,
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
pub struct InstancesMut<B, V, I, W, S, T>
where
  B: ?Sized + TessBackend<V, I, W, S> + InstanceSliceBackend<V, I, W, S, T>,
  V: TessVertexData<S>,
  I: TessIndex,
  W: TessVertexData<S>,
  S: ?Sized,
{
  repr: B::InstanceSliceMutRepr,
}

impl<B, V, I, W, S, T> Deref for InstancesMut<B, V, I, W, S, T>
where
  B: ?Sized + TessBackend<V, I, W, S> + InstanceSliceBackend<V, I, W, S, T>,
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

impl<B, V, I, W, S, T> DerefMut for InstancesMut<B, V, I, W, S, T>
where
  B: ?Sized + TessBackend<V, I, W, S> + InstanceSliceBackend<V, I, W, S, T>,
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
#[derive(Debug)]
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
      TessViewError::IncorrectViewWindow{ capacity, start, nb } => {
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
      vert_nb: tess.vert_nb(),
      inst_nb: tess.inst_nb(),
    }
  }

  /// Create a view that is using the whole input [`Tess`] with `inst_nb` instances.
  pub fn inst_whole(tess: &'a Tess<B, V, I, W, S>, inst_nb: usize) -> Self {
    TessView {
      tess,
      start_index: 0,
      vert_nb: tess.vert_nb(),
      inst_nb,
    }
  }

  /// Create a view that is using only a subpart of the input [`Tess`], starting from the beginning
  /// of the vertices.
  pub fn sub(tess: &'a Tess<B, V, I, W, S>, vert_nb: usize) -> Result<Self, TessViewError> {
    let capacity = tess.vert_nb();

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
      inst_nb: tess.inst_nb(),
    })
  }

  /// Create a view that is using only a subpart of the input [`Tess`], starting from the beginning
  /// of the vertices, with `inst_nb` instances.
  pub fn inst_sub(
    tess: &'a Tess<B, V, I, W, S>,
    vert_nb: usize,
    inst_nb: usize,
  ) -> Result<Self, TessViewError> {
    let capacity = tess.vert_nb();

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
    let capacity = tess.vert_nb();

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
      inst_nb: tess.inst_nb(),
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
    let capacity = tess.vert_nb();

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
    TessView::slice(self, from.start, self.vert_nb() - from.start)
  }

  fn inst_view(
    &self,
    from: RangeFrom<usize>,
    inst_nb: usize,
  ) -> Result<TessView<B, V, I, W, S>, TessViewError> {
    TessView::inst_slice(self, from.start, self.vert_nb() - from.start, inst_nb)
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
