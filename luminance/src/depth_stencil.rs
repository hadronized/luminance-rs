//! Depth test related features.

/// Comparison to perform for depth / stencil operations. `a` is the incoming fragment’s data and b is the fragment’s
/// data that is already stored.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Comparison {
  /// Test never succeeds.
  Never,
  /// Test always succeeds.
  Always,
  /// Test succeeds if `a == b`.
  Equal,
  /// Test succeeds if `a != b`.
  NotEqual,
  /// Test succeeds if `a < b`.
  Less,
  /// Test succeeds if `a <= b`.
  LessOrEqual,
  /// Test succeeds if `a > b`.
  Greater,
  /// Test succeeds if `a >= b`.
  GreaterOrEqual,
}

/// Depth test, either enabled with a [`Comparison`] function, or disabled.
///
/// If you disable depth test, fragments will always be blended, whatever the order in which they are written.
#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq)]
pub enum DepthTest {
  /// Depth test is disabled.
  Off,

  /// Depth test is enabled and depth data will be compared with the carried [`Comparison`] value.
  On(Comparison),
}

/// Depth write mode.
#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq)]
pub enum DepthWrite {
  /// Will write depth data.
  On,

  /// Will not write depth data.
  Off,
}

/// Stencil test, either enabled with a [`Comparison`] function and reference / mask values, and operations, or
/// disabled.
///
/// If you disable depth test, fragments will always be blended, whatever the order in which they are written.
#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq)]
pub enum StencilTest {
  /// Stencil test is disabled.
  Off,

  /// Stencil test is enabled
  On {
    /// Comparison to apply to make a fragment pass the test.
    comparison: Comparison,

    /// Reference value for the comparison.
    reference: u8,

    /// The mask to apply on the fragment stencil value.
    mask: u8,

    /// Action to take when the depth test passes but not the stencil test.
    depth_passes_stencil_fails: StencilOp,

    /// Action to take when the stencil test passes but not the depth test.
    depth_fails_stencil_passes: StencilOp,

    /// Action to take when both the depth and stencil tests pass.
    depth_stencil_pass: StencilOp,
  },
}

/// Possible stencil operations.
#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq)]
pub enum StencilOp {
  /// Keep the current value.
  Keep,

  /// Set the stencil value to zero.
  Zero,

  /// Replace the stencil value.
  Replace,

  /// Increment the stencil value.
  ///
  /// If the stencil value reaches the maximum possible value, it is clamped.
  Increment,

  /// Increment the stencil value.
  ///
  /// If the stencil value reaches the maximum possible value, it wraps around back to `0`.
  IncrementWrap,

  /// Decrement the stencil value.
  ///
  /// If the stencil value reaches 0, it is clamped.
  Decrement,

  /// Decrement the stencil value.
  ///
  /// If the stencil value reaches 0, it wraps back to the maximum value.
  DecrementWrap,

  /// Bit-wise inversion.
  Invert,
}
