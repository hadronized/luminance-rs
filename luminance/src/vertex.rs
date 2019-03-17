//! Vertex formats, associated types and functions.
//!
//! A vertex is a type representing a point. It’s common to find vertex positions, normals, colors
//! or even texture coordinates. Even though you’re free to use whichever type you want, you’re
//! limited to a range of types and dimensions. See [`VertexAttribType`] and [`VertexAttribDim`]
//! for further details.

/// A type that can be used as a [`Vertex`] has to implement that trait – it must provide an
/// associated [`VertexFmt`] value via a function call.
///
/// In theory, you should never have to implement that trait directly. Instead, feel free to use the
/// [luminance-derive] `Vertex` proc-macro-derive instead.
///
/// > Note: implementing this trait is `unsafe`.
pub unsafe trait Vertex {
  /// The associated vertex format.
  fn vertex_fmt() -> VertexFmt;
}

unsafe impl Vertex for () {
  fn vertex_fmt() -> VertexFmt {
    Vec::new()
  }
}

/// A [`VertexFmt`] is a list of [`VertexAttribFmt`]s.
pub type VertexFmt = Vec<IndexedVertexAttribFmt>;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct IndexedVertexAttribFmt {
  pub index: usize,
  pub name: &'static str,
  pub instancing: VertexInstancing,
  pub attrib_fmt: VertexAttribFmt
}

impl IndexedVertexAttribFmt {
  pub fn new<S>(
    sem: S,
    instancing: VertexInstancing,
    attrib_fmt: VertexAttribFmt
  ) -> Self where S: VertexAttribSem {
    let index = sem.index();
    let name = sem.name();
    IndexedVertexAttribFmt { index, name, instancing, attrib_fmt }
  }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum VertexInstancing {
  On,
  Off,
}

/// Vertex attribute format.
///
/// Vertex attributes (such as positions, colors, texture UVs, normals, etc.) have each a specific
/// format that must be passed to the GPU. This type gathers information about a single vertex
/// attribute.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct VertexAttribFmt {
  /// Type of the attribute. See [`VertexAttribType`] for further details.
  pub comp_type: VertexAttribType,
  /// Dimension of the attribute. It should be in 1–4. See [`VertexAttribDim`] for further details.
  pub dim: VertexAttribDim,
  /// Size in bytes that a single element of the attribute takes. That is, if your attribute has
  /// a dimension set to 2, then the unit size should be the size of a single element (not two).
  pub unit_size: usize,
  /// Alignment of the attribute. The best advice is to respect what Rust does, so it’s highly
  /// recommended to use `::std::mem::align_of` to let it does the job for you.
  pub align: usize,
}

/// Possible type of vertex attributes.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum VertexAttribType {
  /// An integral type.
  ///
  /// Typically, `i32` is integral but not `u32`.
  Integral,
  /// An unsigned integral type.
  ///
  /// Typically, `u32` is unsigned but not `i32`.
  Unsigned,
  /// A floating point integral type.
  Floating,
  /// A boolean integral type.
  Boolean,
}

/// Possible dimension of vertex attributes.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum VertexAttribDim {
  /// 1D.
  Dim1,
  /// 2D.
  Dim2,
  /// 3D.
  Dim3,
  /// 4D.
  Dim4,
}

/// Class of vertex attributes.
///
/// A vertex attribute type is always associated with a single constant of type [`VertexAttribFmt`],
/// giving GPUs hints about how to treat them.
pub unsafe trait VertexAttrib {
  const VERTEX_ATTRIB_FMT: VertexAttribFmt;
}

/// Vertex attribute semantics.
///
/// Vertex attribute semantics are a mean to make shaders and vertex buffers talk to each other
/// correctly. This is important for several reasons:
///
///   - The memory layout of your vertex buffers might be very different from an ideal case or even
///     the common case. Shaders don’t have any way to know where to pick vertex attributes from, so
///     a mapping is needed.
///   - Sometimes, a shader just need a few information from the vertex attributes. You then want to
///     be able to authorize _“gaps”_ in the semantics so that shaders can be used for several
///     varieties of vertex formats.
///
/// Vertex attribute semantics are any type that can implement this trait. The idea is that
/// semantics must be unique. The vertex position should have an index that is never used anywhere
/// else in the vertex buffer. Because of the second point above, it’s also highly recommended
/// (even though valid not to) to stick to the same index for a given semantics when you have
/// several tessellations – that allows better composition with shaders. Basically, the best advice
/// to follow: define your semantics once, and keep to them.
///
/// > Note: feel free to use the [luminance-derive] crate to automatically derive this trait from
/// > an `enum`.
pub trait VertexAttribSem: Sized {
  /// Retrieve the semantics index of this semantics.
  fn index(&self) -> usize;
  /// Get the name of this semantics.
  fn name(&self) -> &'static str;
  /// Convert from a semantics name to a semantics.
  fn parse(name: &str) -> Option<Self>;
}

/// Class of types that have an associated value which type implements [`VertexAttribSem`], defining
/// vertex legit attributes.
///
/// Vertex attribute types can be associated with only one semantics.
pub trait HasSemantics {
  type Sem: VertexAttribSem;

  const VERTEX_ATTRIB_SEM: Self::Sem;
}

/// A local version of size_of that depends on the state of the std feature.
#[inline(always)]
const fn size_of<T>() -> usize {
  #[cfg(feature = "std")]
  {
    ::std::mem::size_of::<T>()
  }

  #[cfg(not(feature = "std"))]
  {
    ::core::mem::size_of::<T>()
  }
}

/// A local version of align_of that depends on the state of the std feature.
#[inline(always)]
const fn align_of<T>() -> usize {
  #[cfg(feature = "std")]
  {
    ::std::mem::align_of::<T>()
  }

  #[cfg(not(feature = "std"))]
  {
    ::core::mem::align_of::<T>()
  }
}

// Macro to quickly implement VertexAttrib for a given type.
macro_rules! impl_vertex_attribute {
  ($t:ty, $q:ty, $comp_type:ident, $dim:ident) => {
    unsafe impl VertexAttrib for $t {
      const VERTEX_ATTRIB_FMT: VertexAttribFmt = VertexAttribFmt {
        comp_type: VertexAttribType::$comp_type,
        dim: VertexAttribDim::$dim,
        unit_size: $crate::vertex::size_of::<$q>(),
        align: $crate::vertex::align_of::<$q>(),
      };
    }
  };

  ($t:ty, $comp_type:ident) => {
    impl_vertex_attribute!($t, $t, $comp_type, Dim1);
    impl_vertex_attribute!([$t; 1], $t, $comp_type, Dim1);
    impl_vertex_attribute!([$t; 2], $t, $comp_type, Dim2);
    impl_vertex_attribute!([$t; 3], $t, $comp_type, Dim3);
    impl_vertex_attribute!([$t; 4], $t, $comp_type, Dim4);
  };
}

impl_vertex_attribute!(i8, Integral);
impl_vertex_attribute!(i16, Integral);
impl_vertex_attribute!(i32, Integral);
impl_vertex_attribute!(u8, Unsigned);
impl_vertex_attribute!(u16, Unsigned);
impl_vertex_attribute!(u32, Unsigned);
impl_vertex_attribute!(f32, Floating);
impl_vertex_attribute!(f64, Floating);
impl_vertex_attribute!(bool, Floating);
