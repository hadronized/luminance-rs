pub trait RenderChannel {
  const CHANNEL_TY: RenderChannelType;
}

macro_rules! impl_RenderChannel {
  ($ty:ty, $var:ident, $dim:ident) => {
    impl RenderChannel for $ty {
      const CHANNEL_TY: RenderChannelType = RenderChannelType::$var(RenderChannelDim::$dim);
    }
  };
}

impl_RenderChannel!(i32, Integral, Dim1);
impl_RenderChannel!(u32, Unsigned, Dim1);
impl_RenderChannel!(f32, Floating, Dim1);
impl_RenderChannel!(bool, Boolean, Dim1);

#[cfg(feature = "mint")]
impl_RenderChannel!(mint::Vector2<i32>, Integral, Dim2);
#[cfg(feature = "mint")]
impl_RenderChannel!(mint::Vector2<u32>, Unsigned, Dim2);
#[cfg(feature = "mint")]
impl_RenderChannel!(mint::Vector2<f32>, Floating, Dim2);
#[cfg(feature = "mint")]
impl_RenderChannel!(mint::Vector2<bool>, Boolean, Dim2);

#[cfg(feature = "mint")]
impl_RenderChannel!(mint::Vector3<i32>, Integral, Dim3);
#[cfg(feature = "mint")]
impl_RenderChannel!(mint::Vector3<u32>, Unsigned, Dim3);
#[cfg(feature = "mint")]
impl_RenderChannel!(mint::Vector3<f32>, Floating, Dim3);
#[cfg(feature = "mint")]
impl_RenderChannel!(mint::Vector3<bool>, Boolean, Dim3);

#[cfg(feature = "mint")]
impl_RenderChannel!(mint::Vector4<i32>, Integral, Dim4);
#[cfg(feature = "mint")]
impl_RenderChannel!(mint::Vector4<u32>, Unsigned, Dim4);
#[cfg(feature = "mint")]
impl_RenderChannel!(mint::Vector4<f32>, Floating, Dim4);
#[cfg(feature = "mint")]
impl_RenderChannel!(mint::Vector4<bool>, Boolean, Dim4);

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum RenderChannelType {
  /// An integral type.
  ///
  /// Typically, `i32` is integral but not `u32`.
  Integral(RenderChannelDim),

  /// An unsigned integral type.
  ///
  /// Typically, `u32` is unsigned but not `i32`.
  Unsigned(RenderChannelDim),

  /// A floating point integral type.
  Floating(RenderChannelDim),

  /// A boolean integral type.
  Boolean(RenderChannelDim),
}

pub trait DepthChannel {
  const CHANNEL_TY: DepthChannelType;
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum RenderChannelDim {
  Dim1,
  Dim2,
  Dim3,
  Dim4,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum DepthChannelType {
  Depth32F,
  Depth24FStencil8,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct Depth32F;

impl DepthChannel for Depth32F {
  const CHANNEL_TY: DepthChannelType = DepthChannelType::Depth32F;
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct Depth24FStencil8;

impl DepthChannel for Depth24FStencil8 {
  const CHANNEL_TY: DepthChannelType = DepthChannelType::Depth24FStencil8;
}
