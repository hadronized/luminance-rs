//! GPU render state.
//!
//! Such a state controls how the GPU must operate some fixed pipeline functionality, such as the
//! blending, depth test or face culling operations.

use crate::{
  blending::{Blending, BlendingMode, Equation, Factor},
  depth_stencil::{Comparison, DepthTest, DepthWrite, StencilTest},
  face_culling::FaceCulling,
  scissor::Scissor,
};

/// GPU render state.
///
/// You can get a default value with `RenderState::default` and set the operations you want with the
/// various `RenderState::set_*` methods.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RenderState {
  /// Blending configuration.
  pub blending: BlendingMode,

  /// Depth test configuration.
  pub depth_test: DepthTest,

  /// Depth write configuration.
  pub depth_write: DepthWrite,

  /// Stencil test configuration.
  pub stencil_test: StencilTest,

  /// Face culling configuration.
  pub face_culling: FaceCulling,

  /// Scissor region configuration.
  pub scissor: Scissor,
}

impl Default for RenderState {
  /// The default `RenderState`.
  ///
  ///   - `blending`: `BlendingMode::Combined(Blending { equation: Equation::Additive, src: Factor::One, dst: Factor::Zero })`
  ///   - `depth_test`: `DepthTest::On(Comparison::Less)`
  ///   - `depth_write`: `DepthWrite::On`
  ///   - `stencil_test`: `StencilTest::Off`
  ///   - `face_culling`: `FaceCulling::default()`
  ///   - 'scissor`: `Scissor::Off`
  fn default() -> Self {
    RenderState {
      blending: BlendingMode::Combined(Blending {
        equation: Equation::Additive,
        src: Factor::One,
        dst: Factor::Zero,
      }),
      depth_test: DepthTest::On(Comparison::Less),
      depth_write: DepthWrite::On,
      stencil_test: StencilTest::Off,
      face_culling: FaceCulling::default(),
      scissor: Scissor::Off,
    }
  }
}
