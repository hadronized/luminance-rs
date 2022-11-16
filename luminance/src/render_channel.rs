use crate::pixel::{
  self, Floating, Integral, Pixel, PixelFormat, PixelType, Unsigned, R32F, R32I, R32UI, RG32F,
  RG32I, RG32UI, RGB32F, RGB32I, RGB32UI, RGBA32F, RGBA32I, RGBA32UI,
};

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct RenderChannelDesc {
  pub name: &'static str,
  pub ty: RenderChannelType,
}

pub trait RenderChannel {
  type Type: PixelType;

  const CHANNEL_TY: RenderChannelType;
}

macro_rules! impl_RenderChannel {
  ($ty:ty, $var:ident, $dim:ident) => {
    impl RenderChannel for $ty {
      type Type = $var;

      const CHANNEL_TY: RenderChannelType = RenderChannelType::$var(RenderChannelDim::$dim);
    }
  };
}

impl_RenderChannel!(i32, Integral, Dim1);
impl_RenderChannel!(u32, Unsigned, Dim1);
impl_RenderChannel!(f32, Floating, Dim1);

#[cfg(feature = "mint")]
impl_RenderChannel!(mint::Vector2<i32>, Integral, Dim2);
#[cfg(feature = "mint")]
impl_RenderChannel!(mint::Vector2<u32>, Unsigned, Dim2);
#[cfg(feature = "mint")]
impl_RenderChannel!(mint::Vector2<f32>, Floating, Dim2);

#[cfg(feature = "mint")]
impl_RenderChannel!(mint::Vector3<i32>, Integral, Dim3);
#[cfg(feature = "mint")]
impl_RenderChannel!(mint::Vector3<u32>, Unsigned, Dim3);
#[cfg(feature = "mint")]
impl_RenderChannel!(mint::Vector3<f32>, Floating, Dim3);

#[cfg(feature = "mint")]
impl_RenderChannel!(mint::Vector4<i32>, Integral, Dim4);
#[cfg(feature = "mint")]
impl_RenderChannel!(mint::Vector4<u32>, Unsigned, Dim4);
#[cfg(feature = "mint")]
impl_RenderChannel!(mint::Vector4<f32>, Floating, Dim4);

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
}

impl RenderChannelType {
  pub fn to_pixel_format(&self) -> PixelFormat {
    match self {
      RenderChannelType::Integral(RenderChannelDim::Dim1) => R32I::pixel_format(),
      RenderChannelType::Integral(RenderChannelDim::Dim2) => RG32I::pixel_format(),
      RenderChannelType::Integral(RenderChannelDim::Dim3) => RGB32I::pixel_format(),
      RenderChannelType::Integral(RenderChannelDim::Dim4) => RGBA32I::pixel_format(),

      RenderChannelType::Unsigned(RenderChannelDim::Dim1) => R32UI::pixel_format(),
      RenderChannelType::Unsigned(RenderChannelDim::Dim2) => RG32UI::pixel_format(),
      RenderChannelType::Unsigned(RenderChannelDim::Dim3) => RGB32UI::pixel_format(),
      RenderChannelType::Unsigned(RenderChannelDim::Dim4) => RGBA32UI::pixel_format(),

      RenderChannelType::Floating(RenderChannelDim::Dim1) => R32F::pixel_format(),
      RenderChannelType::Floating(RenderChannelDim::Dim2) => RG32F::pixel_format(),
      RenderChannelType::Floating(RenderChannelDim::Dim3) => RGB32F::pixel_format(),
      RenderChannelType::Floating(RenderChannelDim::Dim4) => RGBA32F::pixel_format(),
    }
  }
}

pub trait DepthChannel {
  type Type: PixelType;

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

impl DepthChannelType {
  pub fn to_pixel_format(&self) -> PixelFormat {
    match self {
      DepthChannelType::Depth32F => pixel::Depth32F::pixel_format(),
      DepthChannelType::Depth24FStencil8 => pixel::Depth32FStencil8::pixel_format(),
    }
  }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct Depth32F;

impl DepthChannel for Depth32F {
  type Type = Floating;

  const CHANNEL_TY: DepthChannelType = DepthChannelType::Depth32F;
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct Depth24FStencil8;

impl DepthChannel for Depth24FStencil8 {
  type Type = Floating;

  const CHANNEL_TY: DepthChannelType = DepthChannelType::Depth24FStencil8;
}
