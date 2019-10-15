//! Depth test related features.

/// Whether or not depth test should be enabled.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum DepthTest {
  /// The depth test is enabled.
  On,
  /// The depth test is disabled.
  Off,
}

/// Depth comparison to perform while depth test. `a` is the incoming fragment’s depth and b is the
/// fragment’s depth that is already stored.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum DepthComparison {
  /// Depth test never succeeds.
  Never,
  /// Depth test always succeeds.
  Always,
  /// Depth test succeeds if `a == b`.
  Equal,
  /// Depth test succeeds if `a != b`.
  NotEqual,
  /// Depth test succeeds if `a < b`.
  Less,
  /// Depth test succeeds if `a <= b`.
  LessOrEqual,
  /// Depth test succeeds if `a > b`.
  Greater,
  /// Depth test succeeds if `a >= b`.
  GreaterOrEqual,
}
