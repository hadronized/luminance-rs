//! Shading gates.
//!
//! A shading gate is a _pipeline node_ that allows to share shader [`Program`] for deeper nodes.
//!
//! [`Program`]: crate::shader::Program

use crate::{
  backend::shading_gate::ShadingGate as ShadingGateBackend,
  render_gate::RenderGate,
  shader::{Program, ProgramInterface, UniformInterface},
  vertex::Vertex,
};

/// A shading gate.
///
/// This is obtained after entering a [`PipelineGate`].
///
/// # Parametricity
///
/// - `B` is the backend type.
///
/// [`PipelineGate`]: crate::pipeline::PipelineGate
pub struct ShadingGate<'a, B> {
  pub(crate) backend: &'a mut B,
}

impl<'a, B> ShadingGate<'a, B>
where
  B: ShadingGateBackend,
{
  /// Enter a [`ShadingGate`] by using a shader [`Program`].
  ///
  /// The argument closure is given two arguments:
  ///
  /// - A [`ProgramInterface`], that allows to pass values (via [`ProgramInterface::set`]) to the
  ///   in-use shader [`Program`] and/or perform dynamic lookup of uniforms.
  /// - A [`RenderGate`], allowing to create deeper nodes in the graphics pipeline.
  pub fn shade<E, V, Out, Uni, F>(
    &mut self,
    program: &mut Program<B, V, Out, Uni>,
    f: F,
  ) -> Result<(), E>
  where
    V: Vertex,
    Uni: UniformInterface<B>,
    F: for<'b> FnOnce(ProgramInterface<'b, B>, &'b Uni, RenderGate<'b, B>) -> Result<(), E>,
  {
    unsafe {
      self.backend.apply_shader_program(&mut program.repr);
    }

    let render_gate = RenderGate {
      backend: self.backend,
    };
    let program_interface = ProgramInterface {
      program: &mut program.repr,
    };

    f(program_interface, &program.uni, render_gate)
  }
}
