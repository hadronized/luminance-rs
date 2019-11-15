//! GPU render state.
//!
//! Such a state controls how the GPU must operate some fixed pipeline functionality, such as the
//! blending, depth test or face culling operations.

use crate::blending::{Equation, Factor};
use crate::depth_test::DepthComparison;
use crate::face_culling::FaceCulling;

/// GPU render state.
///
/// You can get a default value with `RenderState::default` and set the operations you want with the
/// various `RenderState::set_*` methods.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct RenderState {
  /// Blending configuration.
  pub(crate) blending: Option<(Equation, Factor, Factor)>,
  /// Depth test configuration.
  pub(crate) depth_test: Option<DepthComparison>,
  /// Face culling configuration.
  pub(crate) face_culling: Option<FaceCulling>,
}

impl RenderState {
  /// Override the blending configuration.
  pub fn set_blending<B>(self, blending: B) -> Self
  where
    B: Into<Option<(Equation, Factor, Factor)>>,
  {
    RenderState {
      blending: blending.into(),
      ..self
    }
  }

  /// Blending configuration.
  pub fn blending(self) -> Option<(Equation, Factor, Factor)> {
    self.blending
  }

  /// Override the depth test configuration.
  pub fn set_depth_test<D>(self, depth_test: D) -> Self
  where
    D: Into<Option<DepthComparison>>,
  {
    let depth_test = depth_test.into();
    RenderState { depth_test, ..self }
  }

  /// Depth test configuration.
  pub fn depth_test(self) -> Option<DepthComparison> {
    self.depth_test
  }

  /// Override the face culling configuration.
  pub fn set_face_culling<FC>(self, face_culling: FC) -> Self
  where
    FC: Into<Option<FaceCulling>>,
  {
    RenderState {
      face_culling: face_culling.into(),
      ..self
    }
  }

  /// Face culling configuration.
  pub fn face_culling(self) -> Option<FaceCulling> {
    self.face_culling
  }
}

impl Default for RenderState {
  /// The default `RenderState`.
  ///
  ///   - `blending`: `None`
  ///   - `depth_test`: `Some(DepthComparison::Less)`
  ///   - `face_culling`: `None`
  fn default() -> Self {
    RenderState {
      blending: None,
      depth_test: Some(DepthComparison::Less),
      face_culling: None,
    }
  }
}
