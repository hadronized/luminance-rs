//! Depth test related features.

/// Whether or not depth test should be enabled.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum DepthTest {
  /// The depth test is enabled.
  On,
  /// The depth test is disabled.
  Off,
}
