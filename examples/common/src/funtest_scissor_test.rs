use luminance::{
  backend::{Backend, Error},
  context::Context,
  dim::{Dim2, Size2},
  framebuffer::{Back, Framebuffer},
  pipeline::PipelineState,
  primitive::TriangleFan,
  render_state::RenderState,
  scissor::Scissor,
  shader::{Program, ProgramBuilder},
  vertex_entity::{VertexEntity, VertexEntityBuilder, View},
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
out vec4 frag;

void main() {
  frag = vec4(1., .5, .5, 1.);
}";

pub struct LocalExample {
  program: Program<(), (), TriangleFan, FragSlot, ()>,
  tess: VertexEntity<(), TriangleFan, ()>,
  is_active: bool,
  back_buffer: Framebuffer<Dim2, Back<FragSlot>, Back<()>>,
}

impl Example for LocalExample {
  type Err = Error;

  const TITLE: &'static str = "funtest-scissor-test";

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

    let tess = ctx.new_vertex_entity(VertexEntityBuilder::new())?;

    let back_buffer = ctx.back_buffer(Size2::new(width, height))?;

    Ok(LocalExample {
      program,
      tess,
      is_active: true,
      back_buffer,
    })
  }

  fn render_frame(
    mut self,
    _: f32,
    actions: impl Iterator<Item = InputAction>,
    ctx: &mut Context<impl Backend>,
  ) -> Result<LoopFeedback<Self>, Self::Err> {
    for action in actions {
      match action {
        InputAction::PrimaryReleased => {
          self.is_active = !self.is_active;
          log::info!(
            "scissor test is {}",
            if self.is_active { "active" } else { "inactive" },
          );
        }

        InputAction::Quit => return Ok(LoopFeedback::Exit),
        _ => (),
      }
    }

    let &Size2 { width, height } = self.back_buffer.size();
    let (w2, h2) = (width as u32 / 2, height as u32 / 2);
    let program = &self.program;
    let tess = &self.tess;

    let rdr_state = if self.is_active {
      RenderState::default().set_scissor(Scissor::On {
        x: w2 - w2 / 2,
        y: h2 - h2 / 2,
        width: w2,
        height: h2,
      })
    } else {
      RenderState::default()
    };

    ctx.with_framebuffer(&self.back_buffer, &PipelineState::default(), |mut frame| {
      frame.with_program(program, |mut frame| {
        frame.with_render_state(&rdr_state, |mut frame| {
          frame.render_vertex_entity(tess.view(..4))
        })
      })
    })?;

    Ok(LoopFeedback::Continue(self))
  }
}
