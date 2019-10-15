//! That module exports blending-related types and functions.
//!
//! Given two pixels *src* and *dst* – source and destination, we associate each pixel a blending
//! factor – respectively, *srcK* and *dstK*. *src* is the pixel being computed, and *dst* is the
//! pixel that is already stored in the framebuffer.
//!
//! The pixels can be blended in several ways. See the documentation of [`Equation`] for further
//! details.
//!
//! The factors are encoded with [`Factor`].
//!
//! [`Equation`]: crate::blending::Equation
//! [`Factor`]: crate::blending::Factor

/// Whether or not enable blending.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum BlendingState {
  /// Enable blending.
  On,
  /// Disable blending.
  Off,
}

/// Blending equation. Used to state how blending factors and pixel data should be blended.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Equation {
  /// `Additive` represents the following blending equation:
  ///
  /// > `blended = src * srcK + dst * dstK`
  Additive,
  /// `Subtract` represents the following blending equation:
  ///
  /// > `blended = src * srcK - dst * dstK`
  Subtract,
  /// Because subtracting is not commutative, `ReverseSubtract` represents the following additional
  /// blending equation:
  ///
  /// > `blended = dst * dstK - src * srcK`
  ReverseSubtract,
  /// `Min` represents the following blending equation:
  ///
  /// > `blended = min(src, dst)`
  Min,
  /// `Max` represents the following blending equation:
  ///
  /// > `blended = max(src, dst)`
  Max,
}

/// Blending factors. Pixel data are multiplied by these factors to achieve several effects driven
/// by *blending equations*.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Factor {
  /// `1 * color = factor`
  One,
  /// `0 * color = 0`
  Zero,
  /// `src * color`
  SrcColor,
  /// `(1 - src) * color`
  SrcColorComplement,
  /// `dst * color`
  DestColor,
  /// `(1 - dst) * color`
  DestColorComplement,
  /// `srcA * color`
  SrcAlpha,
  /// `(1 - src) * color`
  SrcAlphaComplement,
  /// `dstA * color`
  DstAlpha,
  /// `(1 - dstA) * color`
  DstAlphaComplement,
  /// This behavior is still not well understood. Dammit.
  SrcAlphaSaturate,
}
