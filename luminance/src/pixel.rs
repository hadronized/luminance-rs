/// Reify a static pixel format at runtime.
pub trait Pixel {
  /// Raw encoding of a single pixel; i.e. that is, encoding of underlying values in contiguous
  /// texture memory, without taking into account channels. It should match the [`PixelFormat`]
  /// mapping.
  type RawEncoding: Copy + Default;

  type Type: PixelType;

  /// Reify to [`PixelFormat`].
  fn pixel_format() -> PixelFormat;
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

  /// Return the number of channels.
  pub fn channels_len(self) -> usize {
    match self.format {
      Format::R(_) => 1,
      Format::RG(_, _) => 2,
      Format::RGB(_, _, _) => 3,
      Format::RGBA(_, _, _, _) => 4,
      Format::SRGB(_, _, _) => 3,
      Format::SRGBA(_, _, _, _) => 4,
      Format::Depth(_) => 1,
      Format::DepthStencil(_, _) => 2,
    }
  }
}

pub trait PixelType {
  fn pixel_type() -> Type;
}

/// Pixel type.
///
/// - Normalized integer types: [`NormIntegral`] and [`NormUnsigned`] represent integer types
///   (signed and unsigned, respectively). However, they are _normalized_ when used in shader
///   stages, i.e. fetching from them will yield a floating-point value. That value is
///   comprised between `0.0` and `1.0`.
/// - Integer types: [`Integral`] and [`Unsigned`] allows to store signed and unsigned integers,
///   respectively.
/// - Floating-point types: currently, only [`Floating`] is supported.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
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
/// Whichever the constructor you choose, the carried [`Size`]s represent how many bits are used to
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
  /// Holds a red, green and blue channels in sRGB colorspace.
  SRGB(Size, Size, Size),
  /// Holds a red, green and blue channels in sRGB colorspace, plus an alpha channel.
  SRGBA(Size, Size, Size, Size),
  /// Holds a depth channel.
  Depth(Size),
  /// Holds a depth+stencil channel.
  DepthStencil(Size, Size),
}

impl Format {
  /// Size (in bytes) of a pixel that a format represents.
  pub fn bytes_len(self) -> usize {
    let bits = match self {
      Format::R(r) => r.bits_len(),
      Format::RG(r, g) => r.bits_len() + g.bits_len(),
      Format::RGB(r, g, b) => r.bits_len() + g.bits_len() + b.bits_len(),
      Format::RGBA(r, g, b, a) => r.bits_len() + g.bits_len() + b.bits_len() + a.bits_len(),
      Format::SRGB(r, g, b) => r.bits_len() + g.bits_len() + b.bits_len(),
      Format::SRGBA(r, g, b, a) => r.bits_len() + g.bits_len() + b.bits_len() + a.bits_len(),
      Format::Depth(d) => d.bits_len(),
      Format::DepthStencil(d, s) => d.bits_len() + s.bits_len(),
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
  pub fn bits_len(self) -> usize {
    match self {
      Size::Eight => 8,
      Size::Ten => 10,
      Size::Eleven => 11,
      Size::Sixteen => 16,
      Size::ThirtyTwo => 32,
    }
  }
}

macro_rules! mk_pixel_type {
  ($name:ident) => {
    #[derive(Copy, Clone, Debug, PartialEq, Eq)]
    pub struct $name;

    impl PixelType for $name {
      fn pixel_type() -> Type {
        Type::$name
      }
    }
  };
}

mk_pixel_type!(NormIntegral);
mk_pixel_type!(NormUnsigned);
mk_pixel_type!(Integral);
mk_pixel_type!(Unsigned);
mk_pixel_type!(Floating);

macro_rules! impl_Pixel {
  ($t:ty, $raw_encoding:ty, $encoding_ty:ident, $format:expr) => {
    impl Pixel for $t {
      type RawEncoding = $raw_encoding;

      type Type = $encoding_ty;

      fn pixel_format() -> PixelFormat {
        PixelFormat {
          encoding: Type::$encoding_ty,
          format: $format,
        }
      }
    }
  };
}

/// A red 8-bit signed integral pixel format.
#[derive(Clone, Copy, Debug)]
pub struct R8I;

impl_Pixel!(R8I, i8, Integral, Format::R(Size::Eight));

/// A red 8-bit signed integral pixel format, accessed as normalized floating pixels.
#[derive(Clone, Copy, Debug)]
pub struct NormR8I;

impl_Pixel!(NormR8I, i8, NormIntegral, Format::R(Size::Eight));

/// A red 8-bit unsigned integral pixel format.
#[derive(Clone, Copy, Debug)]
pub struct R8UI;

impl_Pixel!(R8UI, u8, Unsigned, Format::R(Size::Eight));

/// A red 8-bit unsigned integral pixel format, accessed as normalized floating pixels.
#[derive(Clone, Copy, Debug)]
pub struct NormR8UI;

impl_Pixel!(NormR8UI, u8, NormUnsigned, Format::R(Size::Eight));

/// A red 16-bit signed integral pixel format.
#[derive(Clone, Copy, Debug)]
pub struct R16I;

impl_Pixel!(R16I, i16, Integral, Format::R(Size::Sixteen));

/// A red 16-bit signed integral pixel format, accessed as normalized floating pixels.
#[derive(Clone, Copy, Debug)]
pub struct NormR16I;

impl_Pixel!(NormR16I, i16, NormIntegral, Format::R(Size::Sixteen));

/// A red 16-bit unsigned integral pixel format.
#[derive(Clone, Copy, Debug)]
pub struct R16UI;

impl_Pixel!(R16UI, u16, Unsigned, Format::R(Size::Sixteen));

/// A red 16-bit unsigned integral pixel format, accessed as normalized floating pixels.
#[derive(Clone, Copy, Debug)]
pub struct NormR16UI;

impl_Pixel!(NormR16UI, u16, NormUnsigned, Format::R(Size::Sixteen));

/// A red 32-bit signed integral pixel format.
#[derive(Clone, Copy, Debug)]
pub struct R32I;

impl_Pixel!(R32I, i32, Integral, Format::R(Size::ThirtyTwo));

/// A red 32-bit signed integral pixel format, accessed as normalized floating pixels.
#[derive(Clone, Copy, Debug)]
pub struct NormR32I;

impl_Pixel!(NormR32I, i32, NormIntegral, Format::R(Size::ThirtyTwo));

/// A red 32-bit unsigned integral pixel format.
#[derive(Clone, Copy, Debug)]
pub struct R32UI;

impl_Pixel!(R32UI, u32, Unsigned, Format::R(Size::ThirtyTwo));

/// A red 32-bit unsigned integral pixel format, accessed as normalized floating pixels.
#[derive(Clone, Copy, Debug)]
pub struct NormR32UI;

impl_Pixel!(NormR32UI, u32, NormUnsigned, Format::R(Size::ThirtyTwo));

/// A red 32-bit floating pixel format.
#[derive(Clone, Copy, Debug)]
pub struct R32F;

impl_Pixel!(R32F, f32, Floating, Format::R(Size::ThirtyTwo));

/// A red and green 8-bit signed integral pixel format.
#[derive(Clone, Copy, Debug)]
pub struct RG8I;

impl_Pixel!(RG8I, i8, Integral, Format::RG(Size::Eight, Size::Eight));

/// A red and green 8-bit integral pixel format, accessed as normalized floating pixels.
#[derive(Clone, Copy, Debug)]
pub struct NormRG8I;

impl_Pixel!(
  NormRG8I,
  i8,
  NormIntegral,
  Format::RG(Size::Eight, Size::Eight)
);

/// A red and green 8-bit unsigned integral pixel format.
#[derive(Clone, Copy, Debug)]
pub struct RG8UI;

impl_Pixel!(RG8UI, u8, Unsigned, Format::RG(Size::Eight, Size::Eight));

/// A red and green 8-bit unsigned integral pixel format, accessed as normalized floating pixels.
#[derive(Clone, Copy, Debug)]
pub struct NormRG8UI;

impl_Pixel!(
  NormRG8UI,
  u8,
  NormUnsigned,
  Format::RG(Size::Eight, Size::Eight)
);

/// A red and green 16-bit signed integral pixel format.
#[derive(Clone, Copy, Debug)]
pub struct RG16I;

impl_Pixel!(
  RG16I,
  i16,
  Integral,
  Format::RG(Size::Sixteen, Size::Sixteen)
);

/// A red and green 16-bit integral pixel format, accessed as normalized floating pixels.
#[derive(Clone, Copy, Debug)]
pub struct NormRG16I;

impl_Pixel!(
  NormRG16I,
  i16,
  NormIntegral,
  Format::RG(Size::Sixteen, Size::Sixteen)
);

/// A red and green 16-bit unsigned integral pixel format.
#[derive(Clone, Copy, Debug)]
pub struct RG16UI;

impl_Pixel!(
  RG16UI,
  u16,
  Unsigned,
  Format::RG(Size::Sixteen, Size::Sixteen)
);

/// A red and green 16-bit unsigned integral pixel format, accessed as normalized floating pixels.
#[derive(Clone, Copy, Debug)]
pub struct NormRG16UI;

impl_Pixel!(
  NormRG16UI,
  u16,
  NormUnsigned,
  Format::RG(Size::Sixteen, Size::Sixteen)
);

/// A red and green 32-bit signed integral pixel format.
#[derive(Clone, Copy, Debug)]
pub struct RG32I;

impl_Pixel!(
  RG32I,
  i32,
  Integral,
  Format::RG(Size::ThirtyTwo, Size::ThirtyTwo)
);

/// A red and green 32-bit signed integral pixel format, accessed as normalized floating pixels.
#[derive(Clone, Copy, Debug)]
pub struct NormRG32I;

impl_Pixel!(
  NormRG32I,
  i32,
  NormIntegral,
  Format::RG(Size::ThirtyTwo, Size::ThirtyTwo)
);

/// A red and green 32-bit unsigned integral pixel format.
#[derive(Clone, Copy, Debug)]
pub struct RG32UI;

impl_Pixel!(
  RG32UI,
  u32,
  Unsigned,
  Format::RG(Size::ThirtyTwo, Size::ThirtyTwo)
);

/// A red and green 32-bit unsigned integral pixel format, accessed as normalized floating pixels.
#[derive(Clone, Copy, Debug)]
pub struct NormRG32UI;

impl_Pixel!(
  NormRG32UI,
  u32,
  NormUnsigned,
  Format::RG(Size::ThirtyTwo, Size::ThirtyTwo)
);

/// A red and green 32-bit floating pixel format.
#[derive(Clone, Copy, Debug)]
pub struct RG32F;

impl_Pixel!(
  RG32F,
  f32,
  Floating,
  Format::RG(Size::ThirtyTwo, Size::ThirtyTwo)
);

/// A red, green and blue 8-bit signed integral pixel format.
#[derive(Clone, Copy, Debug)]
pub struct RGB8I;

impl_Pixel!(
  RGB8I,
  i8,
  Integral,
  Format::RGB(Size::Eight, Size::Eight, Size::Eight)
);

/// A red, green and blue 8-bit signed integral pixel format, accessed as normalized floating
/// pixels.
#[derive(Clone, Copy, Debug)]
pub struct NormRGB8I;

impl_Pixel!(
  NormRGB8I,
  i8,
  NormIntegral,
  Format::RGB(Size::Eight, Size::Eight, Size::Eight)
);

/// A red, green and blue 8-bit unsigned integral pixel format.
#[derive(Clone, Copy, Debug)]
pub struct RGB8UI;

impl_Pixel!(
  RGB8UI,
  u8,
  Unsigned,
  Format::RGB(Size::Eight, Size::Eight, Size::Eight)
);

/// A red, green and blue 8-bit unsigned integral pixel format, accessed as normalized floating
/// pixels.
#[derive(Clone, Copy, Debug)]
pub struct NormRGB8UI;

impl_Pixel!(
  NormRGB8UI,
  u8,
  NormUnsigned,
  Format::RGB(Size::Eight, Size::Eight, Size::Eight)
);

/// A red, green and blue 16-bit signed integral pixel format.
#[derive(Clone, Copy, Debug)]
pub struct RGB16I;

impl_Pixel!(
  RGB16I,
  i16,
  Integral,
  Format::RGB(Size::Sixteen, Size::Sixteen, Size::Sixteen)
);

/// A red, green and blue 16-bit signed integral pixel format, accessed as normalized floating
/// pixels.
#[derive(Clone, Copy, Debug)]
pub struct NormRGB16I;

impl_Pixel!(
  NormRGB16I,
  i16,
  NormIntegral,
  Format::RGB(Size::Sixteen, Size::Sixteen, Size::Sixteen)
);

/// A red, green and blue 16-bit unsigned integral pixel format.
#[derive(Clone, Copy, Debug)]
pub struct RGB16UI;

impl_Pixel!(
  RGB16UI,
  u16,
  Unsigned,
  Format::RGB(Size::Sixteen, Size::Sixteen, Size::Sixteen)
);

/// A red, green and blue 16-bit unsigned integral pixel format, accessed as normalized floating
/// pixels.
#[derive(Clone, Copy, Debug)]
pub struct NormRGB16UI;

impl_Pixel!(
  NormRGB16UI,
  u16,
  NormUnsigned,
  Format::RGB(Size::Sixteen, Size::Sixteen, Size::Sixteen)
);

/// A red, green and blue 32-bit signed integral pixel format.
#[derive(Clone, Copy, Debug)]
pub struct RGB32I;

impl_Pixel!(
  RGB32I,
  i32,
  Integral,
  Format::RGB(Size::ThirtyTwo, Size::ThirtyTwo, Size::ThirtyTwo)
);

/// A red, green and blue 32-bit signed integral pixel format, accessed as normalized floating
/// pixels.
#[derive(Clone, Copy, Debug)]
pub struct NormRGB32I;

impl_Pixel!(
  NormRGB32I,
  i32,
  NormIntegral,
  Format::RGB(Size::ThirtyTwo, Size::ThirtyTwo, Size::ThirtyTwo)
);

/// A red, green and blue 32-bit unsigned integral pixel format.
#[derive(Clone, Copy, Debug)]
pub struct RGB32UI;

impl_Pixel!(
  RGB32UI,
  u32,
  Unsigned,
  Format::RGB(Size::ThirtyTwo, Size::ThirtyTwo, Size::ThirtyTwo)
);

/// A red, green and blue 32-bit unsigned integral pixel format, accessed as normalized floating
/// pixels.
#[derive(Clone, Copy, Debug)]
pub struct NormRGB32UI;

impl_Pixel!(
  NormRGB32UI,
  u32,
  NormUnsigned,
  Format::RGB(Size::ThirtyTwo, Size::ThirtyTwo, Size::ThirtyTwo)
);

/// A red, green and blue 32-bit floating pixel format.
#[derive(Clone, Copy, Debug)]
pub struct RGB32F;

impl_Pixel!(
  RGB32F,
  f32,
  Floating,
  Format::RGB(Size::ThirtyTwo, Size::ThirtyTwo, Size::ThirtyTwo)
);

/// A red, green, blue and alpha 8-bit signed integral pixel format.
#[derive(Clone, Copy, Debug)]
pub struct RGBA8I;

impl_Pixel!(
  RGBA8I,
  i8,
  Integral,
  Format::RGBA(Size::Eight, Size::Eight, Size::Eight, Size::Eight)
);

/// A red, green, blue and alpha 8-bit signed integral pixel format, accessed as normalized floating
/// pixels.
#[derive(Clone, Copy, Debug)]
pub struct NormRGBA8I;

impl_Pixel!(
  NormRGBA8I,
  i8,
  NormIntegral,
  Format::RGBA(Size::Eight, Size::Eight, Size::Eight, Size::Eight)
);

/// A red, green, blue and alpha 8-bit unsigned integral pixel format.
#[derive(Clone, Copy, Debug)]
pub struct RGBA8UI;

impl_Pixel!(
  RGBA8UI,
  u8,
  Unsigned,
  Format::RGBA(Size::Eight, Size::Eight, Size::Eight, Size::Eight)
);

/// A red, green, blue and alpha 8-bit unsigned integral pixel format, accessed as normalized
/// floating pixels.
#[derive(Clone, Copy, Debug)]
pub struct NormRGBA8UI;

impl_Pixel!(
  NormRGBA8UI,
  u8,
  NormUnsigned,
  Format::RGBA(Size::Eight, Size::Eight, Size::Eight, Size::Eight)
);

/// A red, green, blue and alpha 16-bit signed integral pixel format.
#[derive(Clone, Copy, Debug)]
pub struct RGBA16I;

impl_Pixel!(
  RGBA16I,
  i16,
  Integral,
  Format::RGBA(Size::Sixteen, Size::Sixteen, Size::Sixteen, Size::Sixteen)
);

/// A red, green, blue and alpha 16-bit signed integral pixel format, accessed as normalized
/// floating pixels.
#[derive(Clone, Copy, Debug)]
pub struct NormRGBA16I;

impl_Pixel!(
  NormRGBA16I,
  i16,
  NormIntegral,
  Format::RGBA(Size::Sixteen, Size::Sixteen, Size::Sixteen, Size::Sixteen)
);

/// A red, green, blue and alpha 16-bit unsigned integral pixel format.
#[derive(Clone, Copy, Debug)]
pub struct RGBA16UI;

impl_Pixel!(
  RGBA16UI,
  u16,
  Unsigned,
  Format::RGBA(Size::Sixteen, Size::Sixteen, Size::Sixteen, Size::Sixteen)
);

/// A red, green, blue and alpha 16-bit unsigned integral pixel format, accessed as normalized
/// floating pixels.
#[derive(Clone, Copy, Debug)]
pub struct NormRGBA16UI;

impl_Pixel!(
  NormRGBA16UI,
  u16,
  NormUnsigned,
  Format::RGBA(Size::Sixteen, Size::Sixteen, Size::Sixteen, Size::Sixteen)
);

/// A red, green, blue and alpha 32-bit signed integral pixel format.
#[derive(Clone, Copy, Debug)]
pub struct RGBA32I;

impl_Pixel!(
  RGBA32I,
  i32,
  Integral,
  Format::RGBA(
    Size::ThirtyTwo,
    Size::ThirtyTwo,
    Size::ThirtyTwo,
    Size::ThirtyTwo
  )
);

/// A red, green, blue and alpha 32-bit signed integral pixel format, accessed as normalized
/// floating pixels.
#[derive(Clone, Copy, Debug)]
pub struct NormRGBA32I;

impl_Pixel!(
  NormRGBA32I,
  i32,
  NormIntegral,
  Format::RGBA(
    Size::ThirtyTwo,
    Size::ThirtyTwo,
    Size::ThirtyTwo,
    Size::ThirtyTwo
  )
);

/// A red, green, blue and alpha 32-bit unsigned integral pixel format.
#[derive(Clone, Copy, Debug)]
pub struct RGBA32UI;

impl_Pixel!(
  RGBA32UI,
  u32,
  Unsigned,
  Format::RGBA(
    Size::ThirtyTwo,
    Size::ThirtyTwo,
    Size::ThirtyTwo,
    Size::ThirtyTwo
  )
);

/// A red, green, blue and alpha 32-bit unsigned integral pixel format, accessed as normalized
/// floating pixels.
#[derive(Clone, Copy, Debug)]
pub struct NormRGBA32UI;

impl_Pixel!(
  NormRGBA32UI,
  u32,
  NormUnsigned,
  Format::RGBA(
    Size::ThirtyTwo,
    Size::ThirtyTwo,
    Size::ThirtyTwo,
    Size::ThirtyTwo
  )
);

/// A red, green, blue and alpha 32-bit floating pixel format.
#[derive(Clone, Copy, Debug)]
pub struct RGBA32F;

impl_Pixel!(
  RGBA32F,
  f32,
  Floating,
  Format::RGBA(
    Size::ThirtyTwo,
    Size::ThirtyTwo,
    Size::ThirtyTwo,
    Size::ThirtyTwo
  )
);

/// A red, green and blue pixel format in which:
///
///   - The red channel is on 11 bits.
///   - The green channel is on 11 bits, too.
///   - The blue channel is on 10 bits.
#[derive(Clone, Copy, Debug)]
pub struct R11G11B10F;

impl_Pixel!(
  R11G11B10F,
  f32,
  Floating,
  Format::RGB(Size::Eleven, Size::Eleven, Size::Ten)
);

/// An 8-bit unsigned integral red, green and blue pixel format in sRGB colorspace.
#[derive(Clone, Copy, Debug)]
pub struct SRGB8UI;

impl_Pixel!(
  SRGB8UI,
  u8,
  NormUnsigned,
  Format::SRGB(Size::Eight, Size::Eight, Size::Eight)
);

/// An 8-bit unsigned integral red, green and blue pixel format in sRGB colorspace, with linear alpha channel.
#[derive(Clone, Copy, Debug)]
pub struct SRGBA8UI;

impl_Pixel!(
  SRGBA8UI,
  u8,
  NormUnsigned,
  Format::SRGBA(Size::Eight, Size::Eight, Size::Eight, Size::Eight)
);

/// A depth 32-bit floating pixel format.
#[derive(Clone, Copy, Debug)]
pub struct Depth32F;

impl_Pixel!(Depth32F, f32, Floating, Format::Depth(Size::ThirtyTwo));

/// A depth 24-bit + stencil 8-bit pixel format.
#[derive(Clone, Copy, Debug)]
pub struct Depth32FStencil8;

impl_Pixel!(
  Depth32FStencil8,
  f32,
  Floating,
  Format::DepthStencil(Size::ThirtyTwo, Size::Eight)
);
