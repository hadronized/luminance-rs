//! Vertex formats, associated types and functions.
//!
//! A vertex is a type representing a point. It’s common to find vertex positions, normals, colors
//! or even texture coordinates. Even though you’re free to use whichever type you want, you’re
//! limited to a range of types and dimensions. See [`VertexAttribType`] and [`VertexAttribDim`]
//! for further details.
//!
//! [`VertexAttribDim`]: crate::vertex::VertexAttribDim
//! [`VertexAttribType`]: crate::vertex::VertexAttribType

use crate::has_field::HasField;
use std::fmt::Debug;

/// A type that can be used as a [`Vertex`] has to implement that trait – it must provide an
/// associated list of [`VertexBufferDesc`] value via a function call. This associated value gives enough
/// information on the types being used as attributes to reify enough memory data to align and, size
/// and type buffers correctly.
///
/// In theory, you should never have to implement that trait directly. Instead, feel free to use the
/// [luminance-derive] [`Vertex`] proc-macro-derive instead.
///
/// > Note: implementing this trait is `unsafe`.
pub unsafe trait Vertex: Copy {
  /// The associated vertex format.
  fn vertex_desc() -> Vec<VertexBufferDesc>;

  fn components_count() -> usize {
    Self::vertex_desc().len()
  }
}

unsafe impl Vertex for () {
  fn vertex_desc() -> Vec<VertexBufferDesc> {
    Vec::new()
  }

  fn components_count() -> usize {
    0
  }
}

pub trait CompatibleVertex<V> {}

/// A vertex with no attribute is compatible with any kind of vertex.
impl<V> CompatibleVertex<V> for () where V: Vertex {}

pub trait Deinterleave<const NAME: &'static str>: HasField<NAME> {
  /// Rank of the field.
  const RANK: usize;
}

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

  /// Vertex attribute descriptor.
  pub attrib_desc: VertexAttribDesc,
}

impl VertexBufferDesc {
  /// Create a new [`VertexBufferDesc`].
  pub fn new(index: usize, name: &'static str, attrib_desc: VertexAttribDesc) -> Self {
    VertexBufferDesc {
      index,
      name,
      attrib_desc,
    }
  }
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

impl VertexAttribDim {
  pub fn size(&self) -> usize {
    *self as usize + 1
  }
}

/// Class of vertex attributes.
///
/// A vertex attribute type is always associated with a single constant of type [`VertexAttribDesc`],
/// giving GPUs hints about how to treat them.
pub unsafe trait VertexAttrib {
  /// The vertex attribute descriptor.
  const VERTEX_ATTRIB_DESC: VertexAttribDesc;
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

    impl_vertex_attribute!([$t; 2], $t, $attr_ty, VertexAttribDim::Dim2);
    impl_vertex_attribute!([$t; 3], $t, $attr_ty, VertexAttribDim::Dim3);
    impl_vertex_attribute!([$t; 4], $t, $attr_ty, VertexAttribDim::Dim4);

    #[cfg(feature = "mint")]
    impl_vertex_attribute!(mint::Vector2<$t>, $t, $attr_ty, VertexAttribDim::Dim2);
    #[cfg(feature = "mint")]
    impl_vertex_attribute!(mint::Vector3<$t>, $t, $attr_ty, VertexAttribDim::Dim3);
    #[cfg(feature = "mint")]
    impl_vertex_attribute!(mint::Vector4<$t>, $t, $attr_ty, VertexAttribDim::Dim4);
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
impl_vertex_attribute!(bool, VertexAttribType::Boolean);
