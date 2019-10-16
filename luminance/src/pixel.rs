//! Pixel formats types and function manipulation.
//!
//! The `Pixel` trait is used to reify a pixel type at runtime via `PixelFormat`.
//!

/// Reify a static pixel format at runtime.
pub unsafe trait Pixel {
  /// Encoding of a single pixel. It should match the [`PixelFormat`] mapping.
  type Encoding;

  /// Raw encoding of a single pixel; i.e. that is, encoding of underlying values in contiguous
  /// texture memory, without taking into account channels. It should match the [`PixelFormat`]
  /// mapping.
  type RawEncoding;

  /// The type of sampler required to access this pixel format.
  type SamplerType: SamplerType;

  /// Reify to `PixelFormat`.
  fn pixel_format() -> PixelFormat;
}

/// Constraint on `Pixel` for color ones.
pub unsafe trait ColorPixel: Pixel {}

/// Constraint on `Pixel` for depth ones.
pub unsafe trait DepthPixel: Pixel {}

/// Constaint on `Pixel` for renderable ones.
pub unsafe trait RenderablePixel: Pixel {}

/// Reify a static sample type at runtime.
///
/// That trait is used to allow sampling with different types than the actual encoding of the
/// texture as long as the [`Type`] remains the same.
pub unsafe trait SamplerType {
  /// Underlying type of the sampler.
  fn sample_type() -> Type;
}

/// A `PixelFormat` gathers a `Type` along with a `Format`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PixelFormat {
  /// Encoding type of the pixel format.
  pub encoding: Type,
  /// Format of the pixel format.
  pub format: Format,
}

impl PixelFormat {
  /// Does a [`PixelFormat`] represent a color?
  pub fn is_color_pixel(self) -> bool {
    match self.format {
      Format::Depth(_) => false,
      _ => true,
    }
  }

  /// Does a [`PixelFormat`] represent depth information?
  pub fn is_depth_pixel(self) -> bool {
    !self.is_color_pixel()
  }

  /// Return the number of canals.
  pub fn canals_len(self) -> usize {
    match self.format {
      Format::R(_) => 1,
      Format::RG(_, _) => 2,
      Format::RGB(_, _, _) => 3,
      Format::RGBA(_, _, _, _) => 4,
      Format::Depth(_) => 1,
    }
  }
}

/// Pixel type.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Type {
  /// Normalized signed integral pixel type.
  NormIntegral,
  /// Normalized unsigned integral pixel type.
  NormUnsigned,
  /// Signed integral pixel type.
  Integral,
  /// Unsigned integral pixel type.
  Unsigned,
  /// Floating-point pixel type.
  Floating,
}

/// Format of a pixel.
///
/// Whichever the constructor you choose, the carried `Size`s represents how many bits are used to
/// represent each channel.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Format {
  /// Holds a red-only channel.
  R(Size),
  /// Holds red and green channels.
  RG(Size, Size),
  /// Holds red, green and blue channels.
  RGB(Size, Size, Size),
  /// Holds red, green, blue and alpha channels.
  RGBA(Size, Size, Size, Size),
  /// Holds a depth channel.
  Depth(Size),
}

impl Format {
  /// Size (in bytes) of a pixel that a format represents.
  pub fn size(self) -> usize {
    let bits = match self {
      Format::R(r) => r.bits(),
      Format::RG(r, g) => r.bits() + g.bits(),
      Format::RGB(r, g, b) => r.bits() + g.bits() + b.bits(),
      Format::RGBA(r, g, b, a) => r.bits() + g.bits() + b.bits() + a.bits(),
      Format::Depth(d) => d.bits(),
    };

    bits / 8
  }
}

/// Size in bits a pixel channel can be.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Size {
  /// 8-bit.
  Eight,
  /// 10-bit.
  Ten,
  /// 11-bit.
  Eleven,
  /// 16-bit.
  Sixteen,
  /// 32-bit.
  ThirtyTwo,
}

impl Size {
  /// Size (in bits).
  pub fn bits(self) -> usize {
    match self {
      Size::Eight => 8,
      Size::Ten => 10,
      Size::Eleven => 11,
      Size::Sixteen => 16,
      Size::ThirtyTwo => 32,
    }
  }
}

/// The normalized (signed) integral sample type.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct NormIntegral;

unsafe impl SamplerType for NormIntegral {
  fn sample_type() -> Type {
    Type::NormIntegral
  }
}

/// The normalized unsigned integral sample type.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct NormUnsigned;

unsafe impl SamplerType for NormUnsigned {
  fn sample_type() -> Type {
    Type::NormUnsigned
  }
}

/// The (signed) integral sample type.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Integral;

unsafe impl SamplerType for Integral {
  fn sample_type() -> Type {
    Type::Integral
  }
}

/// The unsigned integral sample type.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Unsigned;

unsafe impl SamplerType for Unsigned {
  fn sample_type() -> Type {
    Type::Unsigned
  }
}

/// The floating sample type.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Floating;

unsafe impl SamplerType for Floating {
  fn sample_type() -> Type {
    Type::Floating
  }
}

macro_rules! impl_Pixel {
  ($t:ty, $encoding:ty, $raw_encoding:ty, $encoding_ty:ident, $format:expr) => {
    unsafe impl Pixel for $t {
      type Encoding = $encoding;
      type RawEncoding = $raw_encoding;
      type SamplerType = $encoding_ty;

      fn pixel_format() -> PixelFormat {
        PixelFormat {
          encoding: Type::$encoding_ty,
          format: $format,
        }
      }
    }
  };
}

macro_rules! impl_ColorPixel {
  ($t:ty) => {
    unsafe impl ColorPixel for $t {}
  };
}

macro_rules! impl_DepthPixel {
  ($t:ty) => {
    unsafe impl DepthPixel for $t {}
  };
}

macro_rules! impl_RenderablePixel {
  ($t:ty) => {
    unsafe impl RenderablePixel for $t {}
  };
}

/// A red 8-bit signed integral pixel format.
#[derive(Clone, Copy, Debug)]
pub struct R8I;

impl_Pixel!(R8I, i8, i8, Integral, Format::R(Size::Eight));
impl_ColorPixel!(R8I);
impl_RenderablePixel!(R8I);

/// A red 8-bit signed integral pixel format, accessed as normalized floating pixels.
#[derive(Clone, Copy, Debug)]
pub struct NormR8I;

impl_Pixel!(NormR8I, i8, i8, NormIntegral, Format::R(Size::Eight));
impl_ColorPixel!(NormR8I);
impl_RenderablePixel!(NormR8I);

/// A red 8-bit unsigned integral pixel format.
#[derive(Clone, Copy, Debug)]
pub struct R8UI;

impl_Pixel!(R8UI, u8, u8, Unsigned, Format::R(Size::Eight));
impl_ColorPixel!(R8UI);
impl_RenderablePixel!(R8UI);

/// A red 8-bit unsigned integral pixel format, accessed as normalized floating pixels.
#[derive(Clone, Copy, Debug)]
pub struct NormR8UI;

impl_Pixel!(NormR8UI, u8, u8, NormUnsigned, Format::R(Size::Eight));
impl_ColorPixel!(NormR8UI);
impl_RenderablePixel!(NormR8UI);

/// A red 16-bit signed integral pixel format.
#[derive(Clone, Copy, Debug)]
pub struct R16I;

impl_Pixel!(R16I, i16, i16, Integral, Format::R(Size::Sixteen));
impl_ColorPixel!(R16I);
impl_RenderablePixel!(R16I);

/// A red 16-bit signed integral pixel format, accessed as normalized floating pixels.
#[derive(Clone, Copy, Debug)]
pub struct NormR16I;

impl_Pixel!(NormR16I, i16, i16, NormIntegral, Format::R(Size::Sixteen));
impl_ColorPixel!(NormR16I);
impl_RenderablePixel!(NormR16I);

/// A red 16-bit unsigned integral pixel format.
#[derive(Clone, Copy, Debug)]
pub struct R16UI;

impl_Pixel!(R16UI, u16, u16, Unsigned, Format::R(Size::Sixteen));
impl_ColorPixel!(R16UI);
impl_RenderablePixel!(R16UI);

/// A red 16-bit unsigned integral pixel format, accessed as normalized floating pixels.
#[derive(Clone, Copy, Debug)]
pub struct NormR16UI;

impl_Pixel!(NormR16UI, u16, u16, NormUnsigned, Format::R(Size::Sixteen));
impl_ColorPixel!(NormR16UI);
impl_RenderablePixel!(NormR16UI);

/// A red 32-bit signed integral pixel format.
#[derive(Clone, Copy, Debug)]
pub struct R32I;

impl_Pixel!(R32I, i32, i32, Integral, Format::R(Size::ThirtyTwo));
impl_ColorPixel!(R32I);
impl_RenderablePixel!(R32I);

/// A red 32-bit signed integral pixel format, accessed as normalized floating pixels.
#[derive(Clone, Copy, Debug)]
pub struct NormR32I;

impl_Pixel!(NormR32I, i32, i32, NormIntegral, Format::R(Size::ThirtyTwo));
impl_ColorPixel!(NormR32I);
impl_RenderablePixel!(NormR32I);

/// A red 32-bit unsigned integral pixel format.
#[derive(Clone, Copy, Debug)]
pub struct R32UI;

impl_Pixel!(R32UI, u32, u32, Unsigned, Format::R(Size::ThirtyTwo));
impl_ColorPixel!(R32UI);
impl_RenderablePixel!(R32UI);

/// A red 32-bit unsigned integral pixel format, accessed as normalized floating pixels.
#[derive(Clone, Copy, Debug)]
pub struct NormR32UI;

impl_Pixel!(NormR32UI, u32, u32, NormUnsigned, Format::R(Size::ThirtyTwo));
impl_ColorPixel!(NormR32UI);
impl_RenderablePixel!(NormR32UI);

/// A red 32-bit floating pixel format.
#[derive(Clone, Copy, Debug)]
pub struct R32F;

impl_Pixel!(R32F, f32, f32, Floating, Format::R(Size::ThirtyTwo));
impl_ColorPixel!(R32F);
impl_RenderablePixel!(R32F);

/// A red and green 8-bit signed integral pixel format.
#[derive(Clone, Copy, Debug)]
pub struct RG8I;

impl_Pixel!(RG8I, (i8, i8), i8, Integral, Format::RG(Size::Eight, Size::Eight));
impl_ColorPixel!(RG8I);
impl_RenderablePixel!(RG8I);

/// A red and green 8-bit integral pixel format, accessed as normalized floating pixels.
#[derive(Clone, Copy, Debug)]
pub struct NormRG8I;

impl_Pixel!(NormRG8I, (i8, i8), i8, NormIntegral, Format::RG(Size::Eight, Size::Eight));
impl_ColorPixel!(NormRG8I);
impl_RenderablePixel!(NormRG8I);

/// A red and green 8-bit unsigned integral pixel format.
#[derive(Clone, Copy, Debug)]
pub struct RG8UI;

impl_Pixel!(
  RG8UI,
  (u8, u8),
  u8,
  Unsigned,
  Format::RG(Size::Eight, Size::Eight)
);
impl_ColorPixel!(RG8UI);
impl_RenderablePixel!(RG8UI);

/// A red and green 8-bit unsigned integral pixel format, accessed as normalized floating pixels.
#[derive(Clone, Copy, Debug)]
pub struct NormRG8UI;

impl_Pixel!(
  NormRG8UI,
  (u8, u8),
  u8,
  NormUnsigned,
  Format::RG(Size::Eight, Size::Eight)
);
impl_ColorPixel!(NormRG8UI);
impl_RenderablePixel!(NormRG8UI);

/// A red and green 16-bit signed integral pixel format.
#[derive(Clone, Copy, Debug)]
pub struct RG16I;

impl_Pixel!(
  RG16I,
  (i16, i16),
  i16,
  Integral,
  Format::RG(Size::Sixteen, Size::Sixteen)
);
impl_ColorPixel!(RG16I);
impl_RenderablePixel!(RG16I);

/// A red and green 16-bit integral pixel format, accessed as normalized floating pixels.
#[derive(Clone, Copy, Debug)]
pub struct NormRG16I;

impl_Pixel!(NormRG16I, (i16, i16), i16, NormIntegral, Format::RG(Size::Sixteen, Size::Sixteen));
impl_ColorPixel!(NormRG16I);
impl_RenderablePixel!(NormRG16I);

/// A red and green 16-bit unsigned integral pixel format.
#[derive(Clone, Copy, Debug)]
pub struct RG16UI;

impl_Pixel!(
  RG16UI,
  (u16, u16),
  u16,
  Unsigned,
  Format::RG(Size::Sixteen, Size::Sixteen)
);
impl_ColorPixel!(RG16UI);
impl_RenderablePixel!(RG16UI);

/// A red and green 16-bit unsigned integral pixel format, accessed as normalized floating pixels.
#[derive(Clone, Copy, Debug)]
pub struct NormRG16UI;

impl_Pixel!(NormRG16UI, (u16, u16), u16, NormUnsigned, Format::RG(Size::Sixteen, Size::Sixteen));
impl_ColorPixel!(NormRG16UI);
impl_RenderablePixel!(NormRG16UI);

/// A red and green 32-bit signed integral pixel format.
#[derive(Clone, Copy, Debug)]
pub struct RG32I;

impl_Pixel!(
  RG32I,
  (i32, i32),
  i32,
  Integral,
  Format::RG(Size::ThirtyTwo, Size::ThirtyTwo)
);
impl_ColorPixel!(RG32I);
impl_RenderablePixel!(RG32I);

/// A red and green 32-bit signed integral pixel format, accessed as normalized floating pixels.
#[derive(Clone, Copy, Debug)]
pub struct NormRG32I;

impl_Pixel!(NormRG32I, (i32, i32), i32, NormIntegral, Format::RG(Size::ThirtyTwo, Size::ThirtyTwo));
impl_ColorPixel!(NormRG32I);
impl_RenderablePixel!(NormRG32I);

/// A red and green 32-bit unsigned integral pixel format.
#[derive(Clone, Copy, Debug)]
pub struct RG32UI;

impl_Pixel!(
  RG32UI,
  (u32, u32),
  u32,
  Unsigned,
  Format::RG(Size::ThirtyTwo, Size::ThirtyTwo)
);
impl_ColorPixel!(RG32UI);
impl_RenderablePixel!(RG32UI);

/// A red and green 32-bit unsigned integral pixel format, accessed as normalized floating pixels.
#[derive(Clone, Copy, Debug)]
pub struct NormRG32UI;

impl_Pixel!(NormRG32UI, (u32, u32), u32, NormUnsigned, Format::RG(Size::ThirtyTwo, Size::ThirtyTwo));
impl_ColorPixel!(NormRG32UI);
impl_RenderablePixel!(NormRG32UI);

/// A red and green 32-bit floating pixel format.
#[derive(Clone, Copy, Debug)]
pub struct RG32F;

impl_Pixel!(
  RG32F,
  (f32, f32),
  f32,
  Floating,
  Format::RG(Size::ThirtyTwo, Size::ThirtyTwo)
);
impl_ColorPixel!(RG32F);
impl_RenderablePixel!(RG32F);

/// A red, green and blue 8-bit signed integral pixel format.
#[derive(Clone, Copy, Debug)]
pub struct RGB8I;

impl_Pixel!(
  RGB8I,
  (i8, i8, i8),
  i8,
  Integral,
  Format::RGB(Size::Eight, Size::Eight, Size::Eight)
);
impl_ColorPixel!(RGB8I);
impl_RenderablePixel!(RGB8I);

/// A red, green and blue 8-bit signed integral pixel format, accessed as normalized floating
/// pixels.
#[derive(Clone, Copy, Debug)]
pub struct NormRGB8I;

impl_Pixel!(
  NormRGB8I,
  (i8, i8, i8),
  i8,
  NormIntegral,
  Format::RGB(Size::Eight, Size::Eight, Size::Eight)
);
impl_ColorPixel!(NormRGB8I);
impl_RenderablePixel!(NormRGB8I);

/// A red, green and blue 8-bit unsigned integral pixel format.
#[derive(Clone, Copy, Debug)]
pub struct RGB8UI;

impl_Pixel!(
  RGB8UI,
  (u8, u8, u8),
  u8,
  Unsigned,
  Format::RGB(Size::Eight, Size::Eight, Size::Eight)
);
impl_ColorPixel!(RGB8UI);
impl_RenderablePixel!(RGB8UI);

/// A red, green and blue 8-bit unsigned integral pixel format, accessed as normalized floating
/// pixels.
#[derive(Clone, Copy, Debug)]
pub struct NormRGB8UI;

impl_Pixel!(
  NormRGB8UI,
  (u8, u8, u8),
  u8,
  NormUnsigned,
  Format::RGB(Size::Eight, Size::Eight, Size::Eight)
);
impl_ColorPixel!(NormRGB8UI);
impl_RenderablePixel!(NormRGB8UI);

/// A red, green and blue 16-bit signed integral pixel format.
#[derive(Clone, Copy, Debug)]
pub struct RGB16I;

impl_Pixel!(
  RGB16I,
  (i16, i16, i16),
  i16,
  Integral,
  Format::RGB(Size::Sixteen, Size::Sixteen, Size::Sixteen)
);
impl_ColorPixel!(RGB16I);
impl_RenderablePixel!(RGB16I);

/// A red, green and blue 16-bit signed integral pixel format, accessed as normalized floating
/// pixels.
#[derive(Clone, Copy, Debug)]
pub struct NormRGB16I;

impl_Pixel!(
  NormRGB16I,
  (i16, i16, i16),
  i16,
  NormIntegral,
  Format::RGB(Size::Sixteen, Size::Sixteen, Size::Sixteen)
);
impl_ColorPixel!(NormRGB16I);
impl_RenderablePixel!(NormRGB16I);

/// A red, green and blue 16-bit unsigned integral pixel format.
#[derive(Clone, Copy, Debug)]
pub struct RGB16UI;

impl_Pixel!(
  RGB16UI,
  (u16, u16, u16),
  u16,
  Unsigned,
  Format::RGB(Size::Sixteen, Size::Sixteen, Size::Sixteen)
);
impl_ColorPixel!(RGB16UI);
impl_RenderablePixel!(RGB16UI);

/// A red, green and blue 16-bit unsigned integral pixel format, accessed as normalized floating
/// pixels.
#[derive(Clone, Copy, Debug)]
pub struct NormRGB16UI;

impl_Pixel!(
  NormRGB16UI,
  (u16, u16, u16),
  u16,
  NormUnsigned,
  Format::RGB(Size::Sixteen, Size::Sixteen, Size::Sixteen)
);
impl_ColorPixel!(NormRGB16UI);
impl_RenderablePixel!(NormRGB16UI);

/// A red, green and blue 32-bit signed integral pixel format.
#[derive(Clone, Copy, Debug)]
pub struct RGB32I;

impl_Pixel!(
  RGB32I,
  (i32, i32, i32),
  i32,
  Integral,
  Format::RGB(Size::ThirtyTwo, Size::ThirtyTwo, Size::ThirtyTwo)
);
impl_ColorPixel!(RGB32I);
impl_RenderablePixel!(RGB32I);

/// A red, green and blue 32-bit signed integral pixel format, accessed as normalized floating
/// pixels.
#[derive(Clone, Copy, Debug)]
pub struct NormRGB32I;

impl_Pixel!(
  NormRGB32I,
  (i32, i32, i32),
  i32,
  NormIntegral,
  Format::RGB(Size::ThirtyTwo, Size::ThirtyTwo, Size::ThirtyTwo)
);
impl_ColorPixel!(NormRGB32I);
impl_RenderablePixel!(NormRGB32I);

/// A red, green and blue 32-bit unsigned integral pixel format.
#[derive(Clone, Copy, Debug)]
pub struct RGB32UI;

impl_Pixel!(
  RGB32UI,
  (u32, u32, u32),
  u32,
  Unsigned,
  Format::RGB(Size::ThirtyTwo, Size::ThirtyTwo, Size::ThirtyTwo)
);
impl_ColorPixel!(RGB32UI);
impl_RenderablePixel!(RGB32UI);

/// A red, green and blue 32-bit unsigned integral pixel format, accessed as normalized floating
/// pixels.
#[derive(Clone, Copy, Debug)]
pub struct NormRGB32UI;

impl_Pixel!(
  NormRGB32UI,
  (u32, u32, u32),
  u32,
  NormUnsigned,
  Format::RGB(Size::ThirtyTwo, Size::ThirtyTwo, Size::ThirtyTwo)
);
impl_ColorPixel!(NormRGB32UI);
impl_RenderablePixel!(NormRGB32UI);

/// A red, green and blue 32-bit floating pixel format.
#[derive(Clone, Copy, Debug)]
pub struct RGB32F;

impl_Pixel!(
  RGB32F,
  (f32, f32, f32),
  f32,
  Floating,
  Format::RGB(Size::ThirtyTwo, Size::ThirtyTwo, Size::ThirtyTwo)
);
impl_ColorPixel!(RGB32F);
impl_RenderablePixel!(RGB32F);

/// A red, green, blue and alpha 8-bit signed integral pixel format.
#[derive(Clone, Copy, Debug)]
pub struct RGBA8I;

impl_Pixel!(
  RGBA8I,
  (i8, i8, i8, i8),
  i8,
  Integral,
  Format::RGBA(Size::Eight, Size::Eight, Size::Eight, Size::Eight)
);
impl_ColorPixel!(RGBA8I);
impl_RenderablePixel!(RGBA8I);

/// A red, green, blue and alpha 8-bit signed integral pixel format, accessed as normalized floating
/// pixels.
#[derive(Clone, Copy, Debug)]
pub struct NormRGBA8I;

impl_Pixel!(
  NormRGBA8I,
  (i8, i8, i8, i8),
  i8,
  NormIntegral,
  Format::RGBA(Size::Eight, Size::Eight, Size::Eight, Size::Eight)
);
impl_ColorPixel!(NormRGBA8I);
impl_RenderablePixel!(NormRGBA8I);

/// A red, green, blue and alpha 8-bit unsigned integral pixel format.
#[derive(Clone, Copy, Debug)]
pub struct RGBA8UI;

impl_Pixel!(
  RGBA8UI,
  (u8, u8, u8, u8),
  u8,
  Unsigned,
  Format::RGBA(Size::Eight, Size::Eight, Size::Eight, Size::Eight)
);
impl_ColorPixel!(RGBA8UI);
impl_RenderablePixel!(RGBA8UI);

/// A red, green, blue and alpha 8-bit unsigned integral pixel format, accessed as normalized
/// floating pixels.
#[derive(Clone, Copy, Debug)]
pub struct NormRGBA8UI;

impl_Pixel!(
  NormRGBA8UI,
  (u8, u8, u8, u8),
  u8,
  NormUnsigned,
  Format::RGBA(Size::Eight, Size::Eight, Size::Eight, Size::Eight)
);
impl_ColorPixel!(NormRGBA8UI);
impl_RenderablePixel!(NormRGBA8UI);

/// A red, green, blue and alpha 16-bit signed integral pixel format.
#[derive(Clone, Copy, Debug)]
pub struct RGBA16I;

impl_Pixel!(
  RGBA16I,
  (i16, i16, i16, i16),
  i16,
  Integral,
  Format::RGBA(Size::Sixteen, Size::Sixteen, Size::Sixteen, Size::Sixteen)
);
impl_ColorPixel!(RGBA16I);
impl_RenderablePixel!(RGBA16I);

/// A red, green, blue and alpha 16-bit signed integral pixel format, accessed as normalized
/// floating pixels.
#[derive(Clone, Copy, Debug)]
pub struct NormRGBA16I;

impl_Pixel!(
  NormRGBA16I,
  (i16, i16, i16, i16),
  i16,
  NormIntegral,
  Format::RGBA(Size::Sixteen, Size::Sixteen, Size::Sixteen, Size::Sixteen)
);
impl_ColorPixel!(NormRGBA16I);
impl_RenderablePixel!(NormRGBA16I);

/// A red, green, blue and alpha 16-bit unsigned integral pixel format.
#[derive(Clone, Copy, Debug)]
pub struct RGBA16UI;

impl_Pixel!(
  RGBA16UI,
  (u16, u16, u16, u16),
  u16,
  Unsigned,
  Format::RGBA(Size::Sixteen, Size::Sixteen, Size::Sixteen, Size::Sixteen)
);
impl_ColorPixel!(RGBA16UI);
impl_RenderablePixel!(RGBA16UI);

/// A red, green, blue and alpha 16-bit unsigned integral pixel format, accessed as normalized
/// floating pixels.
#[derive(Clone, Copy, Debug)]
pub struct NormRGBA16UI;

impl_Pixel!(
  NormRGBA16UI,
  (u16, u16, u16, u16),
  u16,
  NormUnsigned,
  Format::RGBA(Size::Sixteen, Size::Sixteen, Size::Sixteen, Size::Sixteen)
);
impl_ColorPixel!(NormRGBA16UI);
impl_RenderablePixel!(NormRGBA16UI);

/// A red, green, blue and alpha 32-bit signed integral pixel format.
#[derive(Clone, Copy, Debug)]
pub struct RGBA32I;

impl_Pixel!(
  RGBA32I,
  (i32, i32, i32, i32),
  i32,
  Integral,
  Format::RGBA(Size::ThirtyTwo, Size::ThirtyTwo, Size::ThirtyTwo, Size::ThirtyTwo)
);
impl_ColorPixel!(RGBA32I);
impl_RenderablePixel!(RGBA32I);

/// A red, green, blue and alpha 32-bit signed integral pixel format, accessed as normalized
/// floating pixels.
#[derive(Clone, Copy, Debug)]
pub struct NormRGBA32I;

impl_Pixel!(
  NormRGBA32I,
  (i32, i32, i32, i32),
  i32,
  NormIntegral,
  Format::RGBA(Size::ThirtyTwo, Size::ThirtyTwo, Size::ThirtyTwo, Size::ThirtyTwo)
);
impl_ColorPixel!(NormRGBA32I);
impl_RenderablePixel!(NormRGBA32I);

/// A red, green, blue and alpha 32-bit unsigned integral pixel format.
#[derive(Clone, Copy, Debug)]
pub struct RGBA32UI;

impl_Pixel!(
  RGBA32UI,
  (u32, u32, u32, u32),
  u32,
  Unsigned,
  Format::RGBA(Size::ThirtyTwo, Size::ThirtyTwo, Size::ThirtyTwo, Size::ThirtyTwo)
);
impl_ColorPixel!(RGBA32UI);
impl_RenderablePixel!(RGBA32UI);

/// A red, green, blue and alpha 32-bit unsigned integral pixel format, accessed as normalized
/// floating pixels.
#[derive(Clone, Copy, Debug)]
pub struct NormRGBA32UI;

impl_Pixel!(
  NormRGBA32UI,
  (u32, u32, u32, u32),
  u32,
  NormUnsigned,
  Format::RGBA(Size::ThirtyTwo, Size::ThirtyTwo, Size::ThirtyTwo, Size::ThirtyTwo)
);
impl_ColorPixel!(NormRGBA32UI);
impl_RenderablePixel!(NormRGBA32UI);

/// A red, green, blue and alpha 32-bit floating pixel format.
#[derive(Clone, Copy, Debug)]
pub struct RGBA32F;

impl_Pixel!(
  RGBA32F,
  (f32, f32, f32, f32),
  f32,
  Floating,
  Format::RGBA(Size::ThirtyTwo, Size::ThirtyTwo, Size::ThirtyTwo, Size::ThirtyTwo)
);
impl_ColorPixel!(RGBA32F);
impl_RenderablePixel!(RGBA32F);

/// A red, green and blue pixel format in which:
///
///   - The red channel is on 11 bits.
///   - The green channel is on 11 bits, too.
///   - The blue channel is on 10 bits.
#[derive(Clone, Copy, Debug)]
pub struct R11G11B10F;

impl_Pixel!(
  R11G11B10F,
  (f32, f32, f32, f32),
  f32,
  Floating,
  Format::RGB(Size::Eleven, Size::Eleven, Size::Ten)
);
impl_ColorPixel!(R11G11B10F);
impl_RenderablePixel!(R11G11B10F);

/// A depth 32-bit floating pixel format.
#[derive(Clone, Copy, Debug)]
pub struct Depth32F;

impl_Pixel!(Depth32F, f32, f32, Floating, Format::Depth(Size::ThirtyTwo));
impl_DepthPixel!(Depth32F);
