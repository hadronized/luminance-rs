//! GPU render state.
//!
//! Such a state controls how the GPU must operate some fixed pipeline functionality, such as the
//! blending, depth test or face culling operations.

use crate::blending::{Equation, Factor};
use crate::depth_test::DepthTest;
use crate::face_culling::FaceCulling;

/// GPU render state.
///
/// You can get a default value with `RenderState::default` and set the operations you want with the
/// various `RenderState::set_*` methods.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct RenderState {
  pub(crate) blending: Option<(Equation, Factor, Factor)>,
  pub(crate) depth_test: DepthTest,
  pub(crate) face_culling: Option<FaceCulling>,
}

impl RenderState {
  pub fn set_blending<B>(self, blending: B) -> Self
  where B: Into<Option<(Equation, Factor, Factor)>> {
    RenderState {
      blending: blending.into(),
      ..self
    }
  }

  pub fn blending(&self) -> Option<(Equation, Factor, Factor)> {
    self.blending
  }

  pub fn set_depth_test(self, depth_test: DepthTest) -> Self {
    RenderState { depth_test, ..self }
  }

  pub fn depth_test(&self) -> DepthTest {
    self.depth_test
  }

  pub fn set_face_culling<FC>(self, face_culling: FC) -> Self
  where FC: Into<Option<FaceCulling>> {
    RenderState {
      face_culling: face_culling.into(),
      ..self
    }
  }

  pub fn face_culling(&self) -> Option<FaceCulling> {
    self.face_culling
  }
}

impl Default for RenderState {
  /// The default `RenderState`.
  ///
  ///   - `blending`: `None`
  ///   - `depth_test`: `DepthTest::On`
  ///   - `face_culling`: `None`
  fn default() -> Self {
    RenderState {
      blending: None,
      depth_test: DepthTest::On,
      face_culling: None,
    }
  }
}
