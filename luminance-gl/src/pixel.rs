use luminance::pixel::{Format, PixelFormat, Size, Type};
use gl::types::*;

// OpenGL format, internal sized-format and type.
pub(crate) fn opengl_pixel_format(pf: PixelFormat) -> Option<(GLenum, GLenum, GLenum)> {
  match (pf.format, pf.encoding) {
    // red channel
    (Format::R(Size::Eight), Type::NormUnsigned) => Some((gl::RED, gl::R8, gl::UNSIGNED_BYTE)),
    (Format::R(Size::Eight), Type::NormIntegral) => Some((gl::RED, gl::R8_SNORM, gl::BYTE)),
    (Format::R(Size::Eight), Type::Integral) => Some((gl::RED_INTEGER, gl::R8I, gl::BYTE)),
    (Format::R(Size::Eight), Type::Unsigned) => Some((gl::RED_INTEGER, gl::R8UI, gl::UNSIGNED_BYTE)),

    (Format::R(Size::Sixteen), Type::NormUnsigned) => Some((gl::RED_INTEGER, gl::R16, gl::UNSIGNED_SHORT)),
    (Format::R(Size::Sixteen), Type::NormIntegral) => Some((gl::RED_INTEGER, gl::R16_SNORM, gl::SHORT)),
    (Format::R(Size::Sixteen), Type::Integral) => Some((gl::RED_INTEGER, gl::R16I, gl::SHORT)),
    (Format::R(Size::Sixteen), Type::Unsigned) => Some((gl::RED_INTEGER, gl::R16UI, gl::UNSIGNED_SHORT)),

    (Format::R(Size::ThirtyTwo), Type::NormUnsigned) => Some((gl::RED_INTEGER, gl::RED, gl::UNSIGNED_INT)),
    (Format::R(Size::ThirtyTwo), Type::NormIntegral) => Some((gl::RED_INTEGER, gl::RED, gl::INT)),
    (Format::R(Size::ThirtyTwo), Type::Integral) => Some((gl::RED_INTEGER, gl::R32I, gl::INT)),
    (Format::R(Size::ThirtyTwo), Type::Unsigned) => Some((gl::RED_INTEGER, gl::R32UI, gl::UNSIGNED_INT)),
    (Format::R(Size::ThirtyTwo), Type::Floating) => Some((gl::RED, gl::R32F, gl::FLOAT)),

    // red, blue channels
    (Format::RG(Size::Eight, Size::Eight), Type::NormUnsigned) => Some((gl::RG, gl::RG8, gl::UNSIGNED_BYTE)),
    (Format::RG(Size::Eight, Size::Eight), Type::NormIntegral) => Some((gl::RG, gl::RG8_SNORM, gl::BYTE)),
    (Format::RG(Size::Eight, Size::Eight), Type::Integral) => Some((gl::RG_INTEGER, gl::RG8I, gl::BYTE)),
    (Format::RG(Size::Eight, Size::Eight), Type::Unsigned) => Some((gl::RG_INTEGER, gl::RG8UI, gl::UNSIGNED_BYTE)),

    (Format::RG(Size::Sixteen, Size::Sixteen), Type::NormUnsigned) => Some((gl::RG, gl::RG16, gl::UNSIGNED_SHORT)),
    (Format::RG(Size::Sixteen, Size::Sixteen), Type::NormIntegral) => Some((gl::RG, gl::RG16_SNORM, gl::SHORT)),
    (Format::RG(Size::Sixteen, Size::Sixteen), Type::Integral) => Some((gl::RG_INTEGER, gl::RG16I, gl::SHORT)),
    (Format::RG(Size::Sixteen, Size::Sixteen), Type::Unsigned) => Some((gl::RG_INTEGER, gl::RG16UI, gl::UNSIGNED_SHORT)),

    (Format::RG(Size::ThirtyTwo, Size::ThirtyTwo), Type::NormUnsigned) => Some((gl::RG, gl::RG, gl::UNSIGNED_INT)),
    (Format::RG(Size::ThirtyTwo, Size::ThirtyTwo), Type::NormIntegral) => Some((gl::RG, gl::RG, gl::INT)),
    (Format::RG(Size::ThirtyTwo, Size::ThirtyTwo), Type::Integral) => Some((gl::RG_INTEGER, gl::RG32I, gl::INT)),
    (Format::RG(Size::ThirtyTwo, Size::ThirtyTwo), Type::Unsigned) => Some((gl::RG_INTEGER, gl::RG32UI, gl::UNSIGNED_INT)),
    (Format::RG(Size::ThirtyTwo, Size::ThirtyTwo), Type::Floating) => Some((gl::RG, gl::RG32F, gl::FLOAT)),

    // red, blue, green channels
    (Format::RGB(Size::Eight, Size::Eight, Size::Eight), Type::NormUnsigned) => Some((gl::RGB, gl::RGB8, gl::UNSIGNED_BYTE)),
    (Format::RGB(Size::Eight, Size::Eight, Size::Eight), Type::NormIntegral) => Some((gl::RGB, gl::RGB8_SNORM, gl::BYTE)),
    (Format::RGB(Size::Eight, Size::Eight, Size::Eight), Type::Integral) => Some((gl::RGB_INTEGER, gl::RGB8I, gl::BYTE)),
    (Format::RGB(Size::Eight, Size::Eight, Size::Eight), Type::Unsigned) => Some((gl::RGB_INTEGER, gl::RGB8UI, gl::UNSIGNED_BYTE)),

    (Format::RGB(Size::Sixteen, Size::Sixteen, Size::Sixteen), Type::NormUnsigned) => Some((gl::RGB, gl::RGB16, gl::UNSIGNED_SHORT)),
    (Format::RGB(Size::Sixteen, Size::Sixteen, Size::Sixteen), Type::NormIntegral) => Some((gl::RGB, gl::RGB16_SNORM, gl::SHORT)),
    (Format::RGB(Size::Sixteen, Size::Sixteen, Size::Sixteen), Type::Integral) => Some((gl::RGB_INTEGER, gl::RGB16I, gl::SHORT)),
    (Format::RGB(Size::Sixteen, Size::Sixteen, Size::Sixteen), Type::Unsigned) => Some((gl::RGB_INTEGER, gl::RGB16UI, gl::UNSIGNED_SHORT)),

    (Format::RGB(Size::Eleven, Size::Eleven, Size::Ten), Type::Floating) => Some((gl::RGB, gl::R11F_G11F_B10F, gl::FLOAT)),

    (Format::RGB(Size::ThirtyTwo, Size::ThirtyTwo, Size::ThirtyTwo), Type::NormUnsigned) => Some((gl::RGB, gl::RGB, gl::UNSIGNED_INT)),
    (Format::RGB(Size::ThirtyTwo, Size::ThirtyTwo, Size::ThirtyTwo), Type::NormIntegral) => Some((gl::RGB, gl::RGB, gl::INT)),
    (Format::RGB(Size::ThirtyTwo, Size::ThirtyTwo, Size::ThirtyTwo), Type::Integral) => Some((gl::RGB_INTEGER, gl::RGB32I, gl::INT)),
    (Format::RGB(Size::ThirtyTwo, Size::ThirtyTwo, Size::ThirtyTwo), Type::Unsigned) => Some((gl::RGB_INTEGER, gl::RGB32UI, gl::UNSIGNED_INT)),
    (Format::RGB(Size::ThirtyTwo, Size::ThirtyTwo, Size::ThirtyTwo), Type::Floating) => Some((gl::RGB, gl::RGB32F, gl::FLOAT)),

    // red, blue, green, alpha channels
    (Format::RGBA(Size::Eight, Size::Eight, Size::Eight, Size::Eight), Type::NormUnsigned) => Some((gl::RGBA, gl::RGBA8, gl::UNSIGNED_BYTE)),
    (Format::RGBA(Size::Eight, Size::Eight, Size::Eight, Size::Eight), Type::NormIntegral) => Some((gl::RGBA, gl::RGBA8_SNORM, gl::BYTE)),
    (Format::RGBA(Size::Eight, Size::Eight, Size::Eight, Size::Eight), Type::Integral) => Some((gl::RGBA_INTEGER, gl::RGBA8I, gl::BYTE)),
    (Format::RGBA(Size::Eight, Size::Eight, Size::Eight, Size::Eight), Type::Unsigned) => Some((gl::RGBA_INTEGER, gl::RGBA8UI, gl::UNSIGNED_BYTE)),

    (Format::RGBA(Size::Sixteen, Size::Sixteen, Size::Sixteen, Size::Sixteen), Type::NormUnsigned) => Some((gl::RGBA, gl::RGBA16, gl::UNSIGNED_SHORT)),
    (Format::RGBA(Size::Sixteen, Size::Sixteen, Size::Sixteen, Size::Sixteen), Type::NormIntegral) => Some((gl::RGBA, gl::RGBA16_SNORM, gl::SHORT)),
    (Format::RGBA(Size::Sixteen, Size::Sixteen, Size::Sixteen, Size::Sixteen), Type::Integral) => Some((gl::RGBA_INTEGER, gl::RGBA16I, gl::SHORT)),
    (Format::RGBA(Size::Sixteen, Size::Sixteen, Size::Sixteen, Size::Sixteen), Type::Unsigned) => Some((gl::RGBA_INTEGER, gl::RGBA16UI, gl::UNSIGNED_SHORT)),

    (Format::RGBA(Size::ThirtyTwo, Size::ThirtyTwo, Size::ThirtyTwo, Size::ThirtyTwo), Type::NormUnsigned) => Some((gl::RGBA, gl::RGBA, gl::UNSIGNED_INT)),
    (Format::RGBA(Size::ThirtyTwo, Size::ThirtyTwo, Size::ThirtyTwo, Size::ThirtyTwo), Type::NormIntegral) => Some((gl::RGBA, gl::RGBA, gl::INT)),
    (Format::RGBA(Size::ThirtyTwo, Size::ThirtyTwo, Size::ThirtyTwo, Size::ThirtyTwo), Type::Integral) => Some((gl::RGBA_INTEGER, gl::RGBA32I, gl::INT)),
    (Format::RGBA(Size::ThirtyTwo, Size::ThirtyTwo, Size::ThirtyTwo, Size::ThirtyTwo), Type::Unsigned) => Some((gl::RGBA_INTEGER, gl::RGBA32UI, gl::UNSIGNED_INT)),
    (Format::RGBA(Size::ThirtyTwo, Size::ThirtyTwo, Size::ThirtyTwo, Size::ThirtyTwo), Type::Floating) => Some((gl::RGBA, gl::RGBA32F, gl::FLOAT)),

    (Format::Depth(Size::ThirtyTwo), Type::Floating) => Some((gl::DEPTH_COMPONENT, gl::DEPTH_COMPONENT32F, gl::FLOAT)),

    _ => None
  }
}
