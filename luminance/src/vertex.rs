//! Vertex formats, associated types and functions.
//!
//! A vertex is a type representing a point. It’s common to find vertex positions, normals, colors
//! or even texture coordinates. Even though you’re free to use whichever type you want, you’re
//! limited to a range of types and dimensions. See [`VertexAttribType`] and [`VertexAttribDim`]
//! for further details.
//!
//! [`VertexAttribDim`]: crate::vertex::VertexAttribDim
//! [`VertexAttribType`]: crate::vertex::VertexAttribType

use std::fmt::Debug;

/// A type that can be used as a [`Vertex`] has to implement that trait – it must provide an
/// associated [`VertexDesc`] value via a function call. This associated value gives enough
/// information on the types being used as attributes to reify enough memory data to align and, size
/// and type buffers correctly.
///
/// In theory, you should never have to implement that trait directly. Instead, feel free to use the
/// [luminance-derive] [`Vertex`] proc-macro-derive instead.
///
/// > Note: implementing this trait is `unsafe`.
pub unsafe trait Vertex {
  /// The associated vertex format.
  fn vertex_desc() -> VertexDesc;
}

unsafe impl Vertex for () {
  fn vertex_desc() -> VertexDesc {
    Vec::new()
  }
}

/// A [`VertexDesc`] is a list of [`VertexBufferDesc`]s.
pub type VertexDesc = Vec<VertexBufferDesc>;

/// A vertex attribute descriptor in a vertex buffer.
///
/// Such a description is used to state what vertex buffers are made of and how they should be
/// aligned / etc.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct VertexBufferDesc {
  /// Internal index of the attribute.
  ///
  /// That index is used as a mapping with vertex shaders to know how to fetch vertex attributes.
  pub index: usize,
  /// The name of the attribute.
  ///
  /// Such a name is used in vertex shaders to perform mapping.
  pub name: &'static str,
  /// Whether _vertex instancing_ should be used with that vertex attribute.
  pub instancing: VertexInstancing,
  /// Vertex attribute descriptor.
  pub attrib_desc: VertexAttribDesc,
}

impl VertexBufferDesc {
  /// Create a new [`VertexBufferDesc`].
  pub fn new<S>(sem: S, instancing: VertexInstancing, attrib_desc: VertexAttribDesc) -> Self
  where
    S: Semantics,
  {
    let index = sem.index();
    let name = sem.name();
    VertexBufferDesc {
      index,
      name,
      instancing,
      attrib_desc,
    }
  }
}

/// Should vertex instancing be used for a vertex attribute?
///
/// Enabling this is done per attribute but if you enable it for a single attribute of a struct, it
/// should be enabled for all others (interleaved vertex instancing is not supported).
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum VertexInstancing {
  /// Use vertex instancing.
  On,
  /// Disable vertex instancing.
  Off,
}

/// Vertex attribute format.
///
/// Vertex attributes (such as positions, colors, texture UVs, normals, etc.) have all a specific
/// format that must be passed to the GPU. This type gathers information about a single vertex
/// attribute and is completly agnostic of the rest of the attributes used to form a vertex.
///
/// A type is associated with a single value of type [`VertexAttribDesc`] via the [`VertexAttrib`]
/// trait. If such an implementor exists for a type, it means that this type can be used as a vertex
/// attribute.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct VertexAttribDesc {
  /// Type of the attribute. See [`VertexAttribType`] for further details.
  pub ty: VertexAttribType,
  /// Dimension of the attribute. It should be in 1–4. See [`VertexAttribDim`] for further details.
  pub dim: VertexAttribDim,
  /// Size in bytes that a single element of the attribute takes. That is, if your attribute has
  /// a dimension set to 2, then the unit size should be the size of a single element (not two).
  pub unit_size: usize,
  /// Alignment of the attribute. The best advice is to respect what Rust does, so it’s highly
  /// recommended to use `::std::mem::align_of` to let it does the job for you.
  pub align: usize,
}

impl VertexAttribDesc {
  /// Normalize a vertex attribute format’s type.
  pub fn normalize(self) -> Self {
    VertexAttribDesc {
      ty: self.ty.normalize(),
      ..self
    }
  }
}

/// Possible type of vertex attributes.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum VertexAttribType {
  /// An integral type.
  ///
  /// Typically, `i32` is integral but not `u32`.
  Integral(Normalized),
  /// An unsigned integral type.
  ///
  /// Typically, `u32` is unsigned but not `i32`.
  Unsigned(Normalized),
  /// A floating point integral type.
  Floating,
  /// A boolean integral type.
  Boolean,
}

impl VertexAttribType {
  /// Normalize a vertex attribute type if it’s integral.
  ///
  /// Return the normalized integer vertex attribute type if non-normalized. Otherwise, return the
  /// vertex attribute type directly.
  pub fn normalize(self) -> Self {
    match self {
      VertexAttribType::Integral(Normalized::No) => VertexAttribType::Integral(Normalized::Yes),
      VertexAttribType::Unsigned(Normalized::No) => VertexAttribType::Unsigned(Normalized::Yes),
      _ => self,
    }
  }
}

/// Whether an integral vertex type should be normalized when fetched from a shader program.
///
/// The default implementation is not to normalize anything. You have to explicitly ask for
/// normalized integers (that will, then, be accessed as floating vertex attributes).
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Normalized {
  /// Normalize integral values and expose them as floating-point values.
  Yes,
  /// Do not perform any normalization and hence leave integral values as-is.
  No,
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
/// A vertex attribute type is always associated with a single constant of type [`VertexAttribDesc`],
/// giving GPUs hints about how to treat them.
pub unsafe trait VertexAttrib {
  /// The vertex attribute descriptor.
  const VERTEX_ATTRIB_DESC: VertexAttribDesc;
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
pub trait Semantics: Sized + Copy + Clone + Debug {
  /// Retrieve the semantics index of this semantics.
  fn index(&self) -> usize;
  /// Get the name of this semantics.
  fn name(&self) -> &'static str;
  /// Get all available semantics.
  fn semantics_set() -> Vec<SemanticsDesc>;
}

impl Semantics for () {
  fn index(&self) -> usize {
    0
  }

  fn name(&self) -> &'static str {
    ""
  }

  fn semantics_set() -> Vec<SemanticsDesc> {
    Vec::new()
  }
}

/// Semantics description.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct SemanticsDesc {
  /// Semantics index.
  pub index: usize,
  /// Name of the semantics (used in shaders).
  pub name: String,
}

/// Class of types that have an associated value which type implements [`Semantics`], defining
/// vertex legit attributes.
///
/// Vertex attribute types can be associated with only one semantics.
pub trait HasSemantics {
  /// Type of the semantics.
  ///
  /// See the [`Semantics`] trait for further information.
  type Sem: Semantics;

  /// The aforementioned vertex semantics for the attribute type.
  const SEMANTICS: Self::Sem;
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
  ($t:ty, $q:ty, $attr_ty:expr, $dim:expr) => {
    unsafe impl VertexAttrib for $t {
      const VERTEX_ATTRIB_DESC: VertexAttribDesc = VertexAttribDesc {
        ty: $attr_ty,
        dim: $dim,
        unit_size: $crate::vertex::size_of::<$q>(),
        align: $crate::vertex::align_of::<$q>(),
      };
    }
  };

  ($t:ty, $attr_ty:expr) => {
    impl_vertex_attribute!($t, $t, $attr_ty, VertexAttribDim::Dim1);
    impl_vertex_attribute!([$t; 1], $t, $attr_ty, VertexAttribDim::Dim1);
    impl_vertex_attribute!([$t; 2], $t, $attr_ty, VertexAttribDim::Dim2);
    impl_vertex_attribute!([$t; 3], $t, $attr_ty, VertexAttribDim::Dim3);
    impl_vertex_attribute!([$t; 4], $t, $attr_ty, VertexAttribDim::Dim4);
  };
}

impl_vertex_attribute!(i8, VertexAttribType::Integral(Normalized::No));
impl_vertex_attribute!(i16, VertexAttribType::Integral(Normalized::No));
impl_vertex_attribute!(i32, VertexAttribType::Integral(Normalized::No));
impl_vertex_attribute!(u8, VertexAttribType::Unsigned(Normalized::No));
impl_vertex_attribute!(u16, VertexAttribType::Unsigned(Normalized::No));
impl_vertex_attribute!(u32, VertexAttribType::Unsigned(Normalized::No));
impl_vertex_attribute!(f32, VertexAttribType::Floating);
impl_vertex_attribute!(f64, VertexAttribType::Floating);
impl_vertex_attribute!(bool, VertexAttribType::Floating);
