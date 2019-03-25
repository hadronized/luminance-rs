//! Face culling is the operation of removing triangles if they’re facing the screen in a specific
//! direction with a specific mode.

/// Face culling setup.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct FaceCulling {
  pub(crate) order: FaceCullingOrder,
  pub(crate) mode: FaceCullingMode,
}

impl FaceCulling {
  pub fn new(order: FaceCullingOrder, mode: FaceCullingMode) -> Self {
    FaceCulling { order, mode }
  }
}

impl Default for FaceCulling {
  fn default() -> Self {
    FaceCulling::new(FaceCullingOrder::CCW, FaceCullingMode::Back)
  }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub(crate) enum FaceCullingState {
  On,
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
