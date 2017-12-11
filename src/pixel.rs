
//! Pixel formats types and function manipulation.
//!
//! The `Pixel` trait is used to reify a pixel type at runtime via `PixelFormat`.

use gl;
use gl::types::*;

/// Reify a static pixel format to runtime.
pub trait Pixel {
  /// Encoding of a single pixel. It should match the `PixelFormat` mapping.
  type Encoding;
  /// Raw encoding of a single pixel; i.e. that is, encoding of underlying values in contiguous
  /// texture memory. It should be match the `PixelFormat` mapping.
  type RawEncoding;

  /// Reify to `PixelFormat`.
  fn pixel_format() -> PixelFormat;
}

/// Constraint on `Pixel` for color ones.
pub trait ColorPixel: Pixel {}

/// Constraint on `Pixel` for depth ones.
pub trait DepthPixel: Pixel {}

/// Constaint on `Pixel` for renderable ones.
pub trait RenderablePixel: Pixel {}

/// A `PixelFormat` gathers a `Type` along with a `Format`.
#[derive(Clone, Copy, Debug)]
pub struct PixelFormat {
  /// Encoding type of the pixel format.
  pub encoding: Type,
  /// Format of the pixel format.
  pub format: Format
}

/// Pixel type.
#[derive(Clone, Copy, Debug)]
pub enum Type {
  Integral,
  Unsigned,
  Floating
}

/// Format of a pixel.
///
/// Whichever the constructor you choose, the carried `u8`s represents how many bytes are used to
/// represent each channel.
#[derive(Clone, Copy, Debug)]
pub enum Format {
    /// Holds a red-only channel.
  R(u8),
    /// Holds red and green channels.
  RG(u8, u8),
  /// Holds red, green and blue channels.
  RGB(u8, u8, u8),
  /// Holds red, green, blue and alpha channels.
  RGBA(u8, u8, u8, u8),
  /// Holds a depth channel.
  Depth(u8)
}

/// Does a `PixelFormat` represent a color?
pub fn is_color_pixel(f: PixelFormat) -> bool {
  match f.format {
    Format::Depth(_) => false,
    _ => true
  }
}

/// Does a `PixelFormat` represent depth information?
pub fn is_depth_pixel(f: PixelFormat) -> bool {
  !is_color_pixel(f)
}

macro_rules! impl_Pixel {
  ($t:ty, $encoding:ty, $raw_encoding:ty, $encoding_ty:expr, $format:expr) => {
    impl Pixel for $t {
      type Encoding = $encoding;
      type RawEncoding = $raw_encoding;
    
      fn pixel_format() -> PixelFormat {
        PixelFormat {
          encoding: $encoding_ty,
          format: $format
        }
      }
    }
  }
}

macro_rules! impl_ColorPixel {
  ($t:ty) => {
    impl ColorPixel for $t {}
  }
}

macro_rules! impl_DepthPixel {
  ($t:ty) => {
    impl DepthPixel for $t {}
  }
}

macro_rules! impl_RenderablePixel {
  ($t:ty) => {
    impl RenderablePixel for $t {}
  }
}

/// A red 8-bit signed integral pixel format.
#[derive(Clone, Copy, Debug)]
pub struct R8I;

impl_Pixel!(R8I, i8, i8, Type::Integral, Format::R(8));
impl_ColorPixel!(R8I);
impl_RenderablePixel!(R8I);

/// A red 8-bit unsigned integral pixel format.
#[derive(Clone, Copy, Debug)]
pub struct R8UI;

impl_Pixel!(R8UI, u8, u8, Type::Unsigned, Format::R(8));
impl_ColorPixel!(R8UI);
impl_RenderablePixel!(R8UI);

// --------------------

/// A red 16-bit signed integral pixel format.
#[derive(Clone, Copy, Debug)]
pub struct R16I;

impl_Pixel!(R16I, i16, i16, Type::Integral, Format::R(16));
impl_ColorPixel!(R16I);
impl_RenderablePixel!(R16I);

/// A red 16-bit unsigned integral pixel format.
#[derive(Clone, Copy, Debug)]
pub struct R16UI;

impl_Pixel!(R16UI, u16, u16, Type::Unsigned, Format::R(16));
impl_ColorPixel!(R16UI);
impl_RenderablePixel!(R16UI);

// --------------------

/// A red 32-bit signed integral pixel format.
#[derive(Clone, Copy, Debug)]
pub struct R32I;

impl_Pixel!(R32I, i32, i32, Type::Integral, Format::R(32));
impl_ColorPixel!(R32I);
impl_RenderablePixel!(R32I);

/// A red 32-bit unsigned integral pixel format.
#[derive(Clone, Copy, Debug)]
pub struct R32UI;

impl_Pixel!(R32UI, u32, u32, Type::Unsigned, Format::R(32));
impl_ColorPixel!(R32UI);
impl_RenderablePixel!(R32UI);

/// A red 32-bit floating pixel format.
#[derive(Clone, Copy, Debug)]
pub struct R32F;

impl_Pixel!(R32F, f32, f32, Type::Floating, Format::R(32));
impl_ColorPixel!(R32F);
impl_RenderablePixel!(R32F);

// --------------------

/// A red and green 8-bit signed integral pixel format.
#[derive(Clone, Copy, Debug)]
pub struct RG8I;

impl_Pixel!(RG8I, (i8, i8), i8, Type::Integral, Format::RG(8, 8));
impl_ColorPixel!(RG8I);
impl_RenderablePixel!(RG8I);

/// A red and green 8-bit unsigned integral pixel format.
#[derive(Clone, Copy, Debug)]
pub struct RG8UI;

impl_Pixel!(RG8UI, (u8, u8), u8, Type::Unsigned, Format::RG(8, 8));
impl_ColorPixel!(RG8UI);
impl_RenderablePixel!(RG8UI);

// --------------------

/// A red and green 16-bit signed integral pixel format.
#[derive(Clone, Copy, Debug)]
pub struct RG16I;

impl_Pixel!(RG16I, (i16, i16), i16, Type::Integral, Format::RG(16, 16));
impl_ColorPixel!(RG16I);
impl_RenderablePixel!(RG16I);

/// A red and green 16-bit unsigned integral pixel format.
#[derive(Clone, Copy, Debug)]
pub struct RG16UI;

impl_Pixel!(RG16UI, (u16, u16), u16, Type::Unsigned, Format::RG(16, 16));
impl_ColorPixel!(RG16UI);
impl_RenderablePixel!(RG16UI);

// --------------------

/// A red and green 32-bit signed integral pixel format.
#[derive(Clone, Copy, Debug)]
pub struct RG32I;

impl_Pixel!(RG32I, (i32, i32), i32, Type::Integral, Format::RG(32, 32));
impl_ColorPixel!(RG32I);
impl_RenderablePixel!(RG32I);

/// A red and green 32-bit unsigned integral pixel format.
#[derive(Clone, Copy, Debug)]
pub struct RG32UI;

impl_Pixel!(RG32UI, (u32, u32), u32, Type::Unsigned, Format::RG(32, 32));
impl_ColorPixel!(RG32UI);
impl_RenderablePixel!(RG32UI);

/// A red and green 32-bit floating pixel format.
#[derive(Clone, Copy, Debug)]
pub struct RG32F;

impl_Pixel!(RG32F, (f32, f32), f32, Type::Floating, Format::RG(32, 32));
impl_ColorPixel!(RG32F);
impl_RenderablePixel!(RG32F);

// --------------------

/// A red, green and blue 8-bit signed integral pixel format.
#[derive(Clone, Copy, Debug)]
pub struct RGB8I;

impl_Pixel!(RGB8I, (i8, i8, i8), i8, Type::Integral, Format::RGB(8, 8, 8));
impl_ColorPixel!(RGB8I);
impl_RenderablePixel!(RGB8I);

/// A red, green and blue 8-bit unsigned integral pixel format.
#[derive(Clone, Copy, Debug)]
pub struct RGB8UI;

impl_Pixel!(RGB8UI, (u8, u8, u8), u8, Type::Unsigned, Format::RGB(8, 8, 8));
impl_ColorPixel!(RGB8UI);
impl_RenderablePixel!(RGB8UI);

// --------------------

/// A red, green and blue 16-bit signed integral pixel format.
#[derive(Clone, Copy, Debug)]
pub struct RGB16I;

impl_Pixel!(RGB16I, (i16, i16, i16), i16, Type::Integral, Format::RGB(16, 16, 16));
impl_ColorPixel!(RGB16I);
impl_RenderablePixel!(RGB16I);

/// A red, green and blue 16-bit unsigned integral pixel format.
#[derive(Clone, Copy, Debug)]
pub struct RGB16UI;

impl_Pixel!(RGB16UI, (u16, u16, u16), u16, Type::Unsigned, Format::RGB(16, 16, 16));
impl_ColorPixel!(RGB16UI);
impl_RenderablePixel!(RGB16UI);

// --------------------

/// A red, green and blue 32-bit signed integral pixel format.
#[derive(Clone, Copy, Debug)]
pub struct RGB32I;

impl_Pixel!(RGB32I, (i32, i32, i32), i32, Type::Integral, Format::RGB(32, 32, 32));
impl_ColorPixel!(RGB32I);
impl_RenderablePixel!(RGB32I);

/// A red, green and blue 32-bit unsigned integral pixel format.
#[derive(Clone, Copy, Debug)]
pub struct RGB32UI;

impl_Pixel!(RGB32UI, (u32, u32, u32), u32, Type::Unsigned, Format::RGB(32, 32, 32));
impl_ColorPixel!(RGB32UI);
impl_RenderablePixel!(RGB32UI);

/// A red, green and blue 32-bit floating pixel format.
#[derive(Clone, Copy, Debug)]
pub struct RGB32F;

impl_Pixel!(RGB32F, (f32, f32, f32), f32, Type::Floating, Format::RGB(32, 32, 32));
impl_ColorPixel!(RGB32F);
impl_RenderablePixel!(RGB32F);

// --------------------

/// A red, green, blue and alpha 8-bit signed integral pixel format.
#[derive(Clone, Copy, Debug)]
pub struct RGBA8I;

impl_Pixel!(RGBA8I, (i8, i8, i8, i8), i8, Type::Integral, Format::RGBA(8, 8, 8, 8));
impl_ColorPixel!(RGBA8I);
impl_RenderablePixel!(RGBA8I);

/// A red, green, blue and alpha 8-bit unsigned integral pixel format.
#[derive(Clone, Copy, Debug)]
pub struct RGBA8UI;

impl_Pixel!(RGBA8UI, (u8, u8, u8, u8), u8, Type::Unsigned, Format::RGBA(8, 8, 8, 8));
impl_ColorPixel!(RGBA8UI);
impl_RenderablePixel!(RGBA8UI);

// --------------------

/// A red, green, blue and alpha 16-bit signed integral pixel format.
#[derive(Clone, Copy, Debug)]
pub struct RGBA16I;

impl_Pixel!(RGBA16I, (i16, i16, i16, i16), i16, Type::Integral, Format::RGBA(16, 16, 16, 16));
impl_ColorPixel!(RGBA16I);
impl_RenderablePixel!(RGBA16I);

/// A red, green, blue and alpha 16-bit unsigned integral pixel format.
#[derive(Clone, Copy, Debug)]
pub struct RGBA16UI;

impl_Pixel!(RGBA16UI, (u16, u16, u16, u16), u16, Type::Unsigned, Format::RGBA(16, 16, 16, 16));
impl_ColorPixel!(RGBA16UI);
impl_RenderablePixel!(RGBA16UI);

// --------------------

/// A red, green, blue and alpha 32-bit signed integral pixel format.
#[derive(Clone, Copy, Debug)]
pub struct RGBA32I;

impl_Pixel!(RGBA32I, (i32, i32, i32, i32), i32, Type::Integral, Format::RGBA(32, 32, 32, 32));
impl_ColorPixel!(RGBA32I);
impl_RenderablePixel!(RGBA32I);

/// A red, green, blue and alpha 32-bit unsigned integral pixel format.
#[derive(Clone, Copy, Debug)]
pub struct RGBA32UI;

impl_Pixel!(RGBA32UI, (u32, u32, u32, u32), u32, Type::Unsigned, Format::RGBA(32, 32, 32, 32));
impl_ColorPixel!(RGBA32UI);
impl_RenderablePixel!(RGBA32UI);

/// A red, green, blue and alpha 32-bit floating pixel format.
#[derive(Clone, Copy, Debug)]
pub struct RGBA32F;

impl_Pixel!(RGBA32F, (f32, f32, f32, f32), f32, Type::Floating, Format::RGBA(32, 32, 32, 32));
impl_ColorPixel!(RGBA32F);
impl_RenderablePixel!(RGBA32F);

// --------------------

/// A depth 32-bit floating pixel format.
#[derive(Clone, Copy, Debug)]
pub struct Depth32F;

impl_Pixel!(Depth32F, f32, f32, Type::Floating, Format::Depth(32));
impl_DepthPixel!(Depth32F);

// OpenGL format, internal sized-format and type.
pub fn opengl_pixel_format(pf: PixelFormat) -> Option<(GLenum, GLenum, GLenum)> {
  match (pf.format, pf.encoding) {
    (Format::R(32), Type::Floating) => Some((gl::RED, gl::R32F, gl::FLOAT)),
    (Format::RGB(8, 8, 8), Type::Unsigned) => Some((gl::RGB_INTEGER, gl::RGB8UI, gl::UNSIGNED_BYTE)),
    (Format::RGBA(8, 8, 8, 8), Type::Unsigned) => Some((gl::RGBA_INTEGER, gl::RGBA8UI, gl::UNSIGNED_BYTE)),
    (Format::RGB(32, 32, 32), Type::Floating) => Some((gl::RGB, gl::RGB32F, gl::FLOAT)),
    (Format::RGBA(32, 32, 32, 32), Type::Floating) => Some((gl::RGBA, gl::RGBA32F, gl::FLOAT)),
    (Format::Depth(32), Type::Floating) => Some((gl::DEPTH_COMPONENT, gl::DEPTH_COMPONENT32F, gl::FLOAT)),
    _ => panic!("unsupported pixel format {:?}", pf)
  }
}

// Return the number of components.
pub fn pixel_components(pf: PixelFormat) -> usize {
  match pf.format {
    Format::RGB(_, _, _) => 3,
    Format::RGBA(_, _, _, _) => 4,
    Format::Depth(_) => 1,
    _ => panic!("unsupported pixel format")
  }
}
