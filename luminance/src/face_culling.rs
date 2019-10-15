//! Face culling is the operation of removing triangles if they’re facing the screen in a specific
//! direction with a specific mode.

/// Face culling setup.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct FaceCulling {
  /// Face culling order.
  pub(crate) order: FaceCullingOrder,
  /// Face culling mode.
  pub(crate) mode: FaceCullingMode,
}

impl FaceCulling {
  /// Create a new [`FaceCulling`].
  pub fn new(order: FaceCullingOrder, mode: FaceCullingMode) -> Self {
    FaceCulling { order, mode }
  }
}

impl Default for FaceCulling {
  fn default() -> Self {
    FaceCulling::new(FaceCullingOrder::CCW, FaceCullingMode::Back)
  }
}

/// Should face culling be enabled?
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum FaceCullingState {
  /// Enable face culling.
  On,
  /// Disable face culling.
  Off,
}

/// Face culling order.
///
/// The order determines how a triangle is determined to be discarded. If the triangle’s vertices
/// wind up in the same direction as the `FaceCullingOrder`, it’s assigned the front side,
/// otherwise, it’s the back side.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum FaceCullingOrder {
  /// Clockwise order.
  CW,
  /// Counter-clockwise order.
  CCW,
}

/// Side to show and side to cull.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum FaceCullingMode {
  /// Cull the front side only.
  Front,
  /// Cull the back side only.
  Back,
  /// Always cull any triangle.
  Both,
}
