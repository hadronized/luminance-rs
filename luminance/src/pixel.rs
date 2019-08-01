//! Pixel formats types and function manipulation.
//!
//! The `Pixel` trait is used to reify a pixel type at runtime via `PixelFormat`.

use crate::metagl::*;

/// Reify a static pixel format at runtime.
pub unsafe trait Pixel {
  /// Encoding of a single pixel. It should match the `PixelFormat` mapping.
  type Encoding;
  /// Raw encoding of a single pixel; i.e. that is, encoding of underlying values in contiguous
  /// texture memory. It should be match the `PixelFormat` mapping.
  type RawEncoding;

  /// The type of sampler required to access this pixel format
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
pub unsafe trait SamplerType {
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

/// Pixel type.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Type {
  Integral,
  Unsigned,
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

/// Size in bits a pixel channel can be.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Size {
  Eight,
  Ten,
  Eleven,
  Sixteen,
  ThirtyTwo,
}

/// Does a `PixelFormat` represent a color?
pub fn is_color_pixel(f: PixelFormat) -> bool {
  match f.format {
    Format::Depth(_) => false,
    _ => true,
  }
}

/// Does a `PixelFormat` represent depth information?
pub fn is_depth_pixel(f: PixelFormat) -> bool {
  !is_color_pixel(f)
}

/// The integral sample type
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Integral;

unsafe impl SamplerType for Integral {
  fn sample_type() -> Type {
    Type::Integral
  }
}

/// The integral sample type
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Unsigned;

unsafe impl SamplerType for Unsigned {
  fn sample_type() -> Type {
    Type::Unsigned
  }
}

/// The integral sample type
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

/// A red 8-bit unsigned integral pixel format.
#[derive(Clone, Copy, Debug)]
pub struct R8UI;

impl_Pixel!(R8UI, u8, u8, Unsigned, Format::R(Size::Eight));
impl_ColorPixel!(R8UI);
impl_RenderablePixel!(R8UI);

// --------------------

/// A red 16-bit signed integral pixel format.
#[derive(Clone, Copy, Debug)]
pub struct R16I;

impl_Pixel!(R16I, i16, i16, Integral, Format::R(Size::Sixteen));
impl_ColorPixel!(R16I);
impl_RenderablePixel!(R16I);

/// A red 16-bit unsigned integral pixel format.
#[derive(Clone, Copy, Debug)]
pub struct R16UI;

impl_Pixel!(R16UI, u16, u16, Unsigned, Format::R(Size::Sixteen));
impl_ColorPixel!(R16UI);
impl_RenderablePixel!(R16UI);

// --------------------

/// A red 32-bit signed integral pixel format.
#[derive(Clone, Copy, Debug)]
pub struct R32I;

impl_Pixel!(R32I, i32, i32, Integral, Format::R(Size::ThirtyTwo));
impl_ColorPixel!(R32I);
impl_RenderablePixel!(R32I);

/// A red 32-bit unsigned integral pixel format.
#[derive(Clone, Copy, Debug)]
pub struct R32UI;

impl_Pixel!(R32UI, u32, u32, Unsigned, Format::R(Size::ThirtyTwo));
impl_ColorPixel!(R32UI);
impl_RenderablePixel!(R32UI);

/// A red 32-bit floating pixel format.
#[derive(Clone, Copy, Debug)]
pub struct R32F;

impl_Pixel!(R32F, f32, f32, Floating, Format::R(Size::ThirtyTwo));
impl_ColorPixel!(R32F);
impl_RenderablePixel!(R32F);

// --------------------

/// A red and green 8-bit signed integral pixel format.
#[derive(Clone, Copy, Debug)]
pub struct RG8I;

impl_Pixel!(RG8I, (i8, i8), i8, Integral, Format::RG(Size::Eight, Size::Eight));
impl_ColorPixel!(RG8I);
impl_RenderablePixel!(RG8I);

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

// --------------------

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

// --------------------

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

// --------------------

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

// --------------------

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

// --------------------

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

// --------------------

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

// --------------------

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

// --------------------

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

// --------------------

/// A depth 32-bit floating pixel format.
#[derive(Clone, Copy, Debug)]
pub struct Depth32F;

impl_Pixel!(Depth32F, f32, f32, Floating, Format::Depth(Size::ThirtyTwo));
impl_DepthPixel!(Depth32F);

// OpenGL format, internal sized-format and type.
pub(crate) fn opengl_pixel_format(pf: PixelFormat) -> Option<(GLenum, GLenum, GLenum)> {
  match (pf.format, pf.encoding) {
    (Format::R(Size::Eight), Type::Integral) => Some((gl::RED_INTEGER, gl::R8I, gl::BYTE)),
    (Format::R(Size::Eight), Type::Unsigned) => Some((gl::RED_INTEGER, gl::R8UI, gl::UNSIGNED_BYTE)),
    (Format::R(Size::Sixteen), Type::Integral) => Some((gl::RED_INTEGER, gl::R16I, gl::SHORT)),
    (Format::R(Size::Sixteen), Type::Unsigned) => Some((gl::RED_INTEGER, gl::R16UI, gl::UNSIGNED_SHORT)),
    (Format::R(Size::ThirtyTwo), Type::Integral) => Some((gl::RED_INTEGER, gl::R32I, gl::INT)),
    (Format::R(Size::ThirtyTwo), Type::Unsigned) => Some((gl::RED_INTEGER, gl::R32UI, gl::UNSIGNED_INT)),
    (Format::R(Size::ThirtyTwo), Type::Floating) => Some((gl::RED, gl::R32F, gl::FLOAT)),

    (Format::RG(Size::Eight, Size::Eight), Type::Integral) => Some((gl::RG_INTEGER, gl::RG8I, gl::BYTE)),
    (Format::RG(Size::Eight, Size::Eight), Type::Unsigned) => Some((gl::RG_INTEGER, gl::RG8UI, gl::UNSIGNED_BYTE)),
    (Format::RG(Size::Sixteen, Size::Sixteen), Type::Integral) => Some((gl::RG_INTEGER, gl::RG16I, gl::SHORT)),
    (Format::RG(Size::Sixteen, Size::Sixteen), Type::Unsigned) => Some((gl::RG_INTEGER, gl::RG16UI, gl::UNSIGNED_SHORT)),
    (Format::RG(Size::ThirtyTwo, Size::ThirtyTwo), Type::Integral) => Some((gl::RG_INTEGER, gl::RG32I, gl::INT)),
    (Format::RG(Size::ThirtyTwo, Size::ThirtyTwo), Type::Unsigned) => Some((gl::RG_INTEGER, gl::RG32UI, gl::UNSIGNED_INT)),
    (Format::RG(Size::ThirtyTwo, Size::ThirtyTwo), Type::Floating) => Some((gl::RG, gl::RG32F, gl::FLOAT)),

    (Format::RGB(Size::Eight, Size::Eight, Size::Eight), Type::Integral) => Some((gl::RGB_INTEGER, gl::RGB8I, gl::BYTE)),
    (Format::RGB(Size::Eight, Size::Eight, Size::Eight), Type::Unsigned) => Some((gl::RGB_INTEGER, gl::RGB8UI, gl::UNSIGNED_BYTE)),
    (Format::RGB(Size::Sixteen, Size::Sixteen, Size::Sixteen), Type::Integral) => Some((gl::RGB_INTEGER, gl::RGB16I, gl::SHORT)),
    (Format::RGB(Size::Sixteen, Size::Sixteen, Size::Sixteen), Type::Unsigned) => Some((gl::RGB_INTEGER, gl::RGB16UI, gl::UNSIGNED_SHORT)),
    (Format::RGB(Size::Eleven, Size::Eleven, Size::Ten), Type::Floating) => Some((gl::RGB, gl::R11F_G11F_B10F, gl::FLOAT)),
    (Format::RGB(Size::ThirtyTwo, Size::ThirtyTwo, Size::ThirtyTwo), Type::Integral) => Some((gl::RGB_INTEGER, gl::RGB32I, gl::INT)),
    (Format::RGB(Size::ThirtyTwo, Size::ThirtyTwo, Size::ThirtyTwo), Type::Unsigned) => Some((gl::RGB_INTEGER, gl::RGB32UI, gl::UNSIGNED_INT)),
    (Format::RGB(Size::ThirtyTwo, Size::ThirtyTwo, Size::ThirtyTwo), Type::Floating) => Some((gl::RGB, gl::RGB32F, gl::FLOAT)),

    (Format::RGBA(Size::Eight, Size::Eight, Size::Eight, Size::Eight), Type::Integral) => Some((gl::RGBA_INTEGER, gl::RGBA8I, gl::BYTE)),
    (Format::RGBA(Size::Eight, Size::Eight, Size::Eight, Size::Eight), Type::Unsigned) => Some((gl::RGBA_INTEGER, gl::RGBA8UI, gl::UNSIGNED_BYTE)),
    (Format::RGBA(Size::Sixteen, Size::Sixteen, Size::Sixteen, Size::Sixteen), Type::Integral) => Some((gl::RGBA_INTEGER, gl::RGBA16I, gl::SHORT)),
    (Format::RGBA(Size::Sixteen, Size::Sixteen, Size::Sixteen, Size::Sixteen), Type::Unsigned) => Some((gl::RGBA_INTEGER, gl::RGBA16UI, gl::UNSIGNED_SHORT)),
    (Format::RGBA(Size::ThirtyTwo, Size::ThirtyTwo, Size::ThirtyTwo, Size::ThirtyTwo), Type::Integral) => Some((gl::RGBA_INTEGER, gl::RGBA32I, gl::INT)),
    (Format::RGBA(Size::ThirtyTwo, Size::ThirtyTwo, Size::ThirtyTwo, Size::ThirtyTwo), Type::Unsigned) => Some((gl::RGBA_INTEGER, gl::RGBA32UI, gl::UNSIGNED_INT)),
    (Format::RGBA(Size::ThirtyTwo, Size::ThirtyTwo, Size::ThirtyTwo, Size::ThirtyTwo), Type::Floating) => Some((gl::RGBA, gl::RGBA32F, gl::FLOAT)),

    (Format::Depth(Size::ThirtyTwo), Type::Floating) => Some((gl::DEPTH_COMPONENT, gl::DEPTH_COMPONENT32F, gl::FLOAT)),

    _ => None
  }
}

// Return the number of components.
pub(crate) fn pixel_components(pf: PixelFormat) -> usize {
  match pf.format {
    Format::RGB(_, _, _) => 3,
    Format::RGBA(_, _, _, _) => 4,
    Format::Depth(_) => 1,
    _ => panic!("unsupported pixel format"),
  }
}
