//! Face culling is the operation of removing triangles if they’re facing the screen in a specific
//! direction with a specific mode.

/// Face culling setup.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum FaceCulling {
  Off,

  On {
    /// Face culling order.
    order: FaceCullingOrder,
    /// Face culling mode.
    face: FaceCullingFace,
  },
}

/// Default implementation of [`FaceCulling`].
///
/// - Order is [`FaceCullingOrder::CCW`].
/// - Face is [`FaceCullingFace::Back`].
impl Default for FaceCulling {
  fn default() -> Self {
    FaceCulling::On {
      order: FaceCullingOrder::CCW,
      face: FaceCullingFace::Back,
    }
  }
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
pub enum FaceCullingFace {
  /// Cull the front side only.
  Front,
  /// Cull the back side only.
  Back,
  /// Always cull any triangle.
  Both,
}
