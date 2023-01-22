use luminance::{
  backend::{Backend, Error},
  context::Context,
  dim::{Dim2, Size2},
  framebuffer::{Back, Framebuffer},
  pipeline::PipelineState,
  primitive::TriangleFan,
  render_state::RenderState,
  shader::{Program, ProgramBuilder, Uni},
  vertex_entity::{VertexEntity, VertexEntityBuilder, View},
  Uniforms,
};

use crate::{shared::FragSlot, Example, InputAction, LoopFeedback, PlatformServices};

const VS: &str = "
const vec2[4] POSITIONS = vec2[](
  vec2(-1., -1.),
  vec2( 1., -1.),
  vec2( 1.,  1.),
  vec2(-1.,  1.)
);

void main() {
  gl_Position = vec4(POSITIONS[gl_VertexID], 0., 1.);
}";

const FS: &str = "
out vec3 frag;

uniform dvec3 color;

void main() {
  frag = vec3(color);
}";

#[derive(Debug, Uniforms)]
struct ShaderInterface {
  color: Uni<mint::Vector3<f64>>,
}

pub struct LocalExample {
  program: Program<(), (), TriangleFan, FragSlot, ShaderInterface>,
  quad: VertexEntity<(), TriangleFan, ()>,
  back_buffer: Framebuffer<Dim2, Back<FragSlot>, Back<()>>,
}

impl Example for LocalExample {
  type Err = Error;

  const TITLE: &'static str = "funtest-gl33-f64-uniform";

  fn bootstrap(
    [width, height]: [u32; 2],
    _: &mut impl PlatformServices,
    ctx: &mut Context<impl Backend>,
  ) -> Result<Self, Self::Err> {
    let program = ctx.new_program(
      ProgramBuilder::new()
        .add_vertex_stage(VS)
        .no_primitive_stage()
        .add_shading_stage(FS),
    )?;

    let quad = ctx.new_vertex_entity(VertexEntityBuilder::new())?;

    let back_buffer = ctx.back_buffer(Size2::new(width, height))?;

    Ok(LocalExample {
      program,
      quad,
      back_buffer,
    })
  }

  fn render_frame(
    self,
    t: f32,
    actions: impl Iterator<Item = InputAction>,
    ctx: &mut Context<impl Backend>,
  ) -> Result<LoopFeedback<Self>, Self::Err> {
    for action in actions {
      match action {
        InputAction::Quit => return Ok(LoopFeedback::Exit),
        _ => (),
      }
    }

    let t = t as f64;
    let color = mint::Vector3 {
      x: t.cos(),
      y: 0.3,
      z: t.sin(),
    };

    let program = &self.program;
    let quad = &self.quad;

    ctx.with_framebuffer(&self.back_buffer, &PipelineState::default(), |mut frame| {
      frame.with_program(program, |mut frame| {
        frame.update(|mut update, unis| update.set(&unis.color, &color))?;
        frame.with_render_state(&RenderState::default(), |mut frame| {
          frame.render_vertex_entity(quad.view(..4))
        })
      })
    })?;

    Ok(LoopFeedback::Continue(self))
  }
}
