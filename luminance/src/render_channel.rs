/// Render channel definition.
///
/// A render channel is a named slot in a [`RenderLayer`].
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct RenderChannel {
  /// Index of the render channel.
  pub index: usize,

  /// Name of the render channel.
  ///
  /// Used by other systems, mostly.
  pub name: &'static str,

  /// Type of the render channel.
  pub ty: RenderChannelType,

  /// Dimension of the render channel.
  pub dim: RenderChannelDim,
}

pub trait IsRenderChannelType {
  const CHANNEL_TY: RenderChannelType;
  const CHANNEL_DIM: RenderChannelDim;
}

macro_rules! impl_IsRenderChannelType {
  ($ty:ty, $var:ident, $dim:ident) => {
    impl IsRenderChannelType for $ty {
      const CHANNEL_TY: RenderChannelType = RenderChannelType::$var;
      const CHANNEL_DIM: RenderChannelDim = RenderChannelDim::$dim;
    }
  };
}

impl_IsRenderChannelType!(i32, Integral, Dim1);
impl_IsRenderChannelType!(u32, Unsigned, Dim1);
impl_IsRenderChannelType!(f32, Floating, Dim1);
impl_IsRenderChannelType!(bool, Boolean, Dim1);

#[cfg(feature = "mint")]
impl<T: IsRenderChannelType> IsRenderChannelType for mint::Vector2<T> {
  const CHANNEL_TY: RenderChannelType = T::CHANNEL_TY;
  const CHANNEL_DIM: RenderChannelDim = RenderChannelDim::Dim2;
}

#[cfg(feature = "mint")]
impl<T: IsRenderChannelType> IsRenderChannelType for mint::Vector3<T> {
  const CHANNEL_TY: RenderChannelType = T::CHANNEL_TY;
  const CHANNEL_DIM: RenderChannelDim = RenderChannelDim::Dim3;
}

#[cfg(feature = "mint")]
impl<T: IsRenderChannelType> IsRenderChannelType for mint::Vector4<T> {
  const CHANNEL_TY: RenderChannelType = T::CHANNEL_TY;
  const CHANNEL_DIM: RenderChannelDim = RenderChannelDim::Dim4;
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum RenderChannelType {
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

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum RenderChannelDim {
  Dim1,
  Dim2,
  Dim3,
  Dim4,
}
