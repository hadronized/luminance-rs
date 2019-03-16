//! Vertex restart related features.

/// Whether or not vertex restart is enabled.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum VertexRestart {
  /// Vertex restart is enabled.
  Enabled,
  /// Vertex restart is disabled.
  Disabled,
}
