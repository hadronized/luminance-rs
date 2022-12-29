//! This program shows how to render a triangle and change its position and color on the fly by
//! updating “shader uniforms”. Those are values stored on the GPU that remain constant for the
//! whole duration of a draw call (you typically change it between each draw call to customize each
//! draw).
//!
//! This example demonstrate how to add time to your shader to start building moving and animated
//! effects.
//!
//! Press the <up action>, <down action>, <left action> and <right action> to move the triangle on
//! the screen.
//!
//! <https://docs.rs/luminance>

use luminance::{
  backend::Backend,
  context::Context,
  dim::{Dim2, Size2},
  framebuffer::{Back, Framebuffer},
  pipeline::PipelineState,
  primitive::Triangle,
  render_state::RenderState,
  shader::{Program, ProgramBuilder, Uni},
  vertex_entity::{VertexEntity, View},
  vertex_storage::Interleaved,
  Uniforms,
};
use mint::{Vector2, Vector3};

use crate::{
  shared::{FragSlot, Vertex},
  Example, InputAction, LoopFeedback, PlatformServices,
};

const VS: &'static str = include_str!("displacement-vs.glsl");
const FS: &'static str = include_str!("displacement-fs.glsl");

// Only one triangle this time.
const TRI_VERTICES: [Vertex; 3] = [
  Vertex::new(
    Vector2 { x: 0.5, y: -0.5 },
    Vector3 {
      x: 1.,
      y: 0.,
      z: 0.,
    },
  ),
  Vertex::new(
    Vector2 { x: 0.0, y: 0.5 },
    Vector3 {
      x: 0.,
      y: 1.,
      z: 0.,
    },
  ),
  Vertex::new(
    Vector2 { x: -0.5, y: -0.5 },
    Vector3 {
      x: 0.,
      y: 0.,
      z: 1.,
    },
  ),
];

// Create a uniform interface. This is a type that will be used to customize the shader. In our
// case, we just want to pass the time and the position of the triangle, for instance.
//
// This macro only supports structs for now; you cannot use enums as uniform interfaces.
#[derive(Debug, Uniforms)]
struct ShaderUniforms {
  #[uniform(name = "t")]
  time: Uni<f32>,
  triangle_pos: Uni<[f32; 2]>,
}

pub struct LocalExample {
  program: Program<Vertex, (), Triangle, FragSlot, ShaderUniforms>,
  triangle: VertexEntity<Vertex, Triangle, Interleaved<Vertex>>,
  triangle_pos: Vector2<f32>,
  back_buffer: Framebuffer<Dim2, Back<FragSlot>, Back<()>>,
}

impl Example for LocalExample {
  type Err = luminance::backend::Error;

  const TITLE: &'static str = "Shader Uniforms";

  fn bootstrap(
    [width, height]: [u32; 2],
    _platform: &mut impl PlatformServices,
    ctx: &mut Context<impl Backend>,
  ) -> Result<Self, Self::Err> {
    let program = ctx.new_program(
      ProgramBuilder::new()
        .add_vertex_stage(VS)
        .no_primitive_stage()
        .add_shading_stage(FS),
    )?;
    let triangle = ctx.new_vertex_entity(
      Interleaved::new().set_vertices(&TRI_VERTICES[..]),
      [],
      Interleaved::new(),
    )?;
    let triangle_pos = Vector2 { x: 0., y: 0. };
    let back_buffer = ctx.back_buffer(Size2::new(width, height))?;

    Ok(Self {
      program,
      triangle,
      triangle_pos,
      back_buffer,
    })
  }

  fn render_frame(
    mut self,
    t: f32,
    actions: impl Iterator<Item = InputAction>,
    ctx: &mut Context<impl Backend>,
  ) -> Result<LoopFeedback<Self>, Self::Err> {
    for action in actions {
      match action {
        InputAction::Quit => return Ok(LoopFeedback::Exit),
        InputAction::Left => self.triangle_pos.x -= 0.1,
        InputAction::Right => self.triangle_pos.x += 0.1,
        InputAction::Forward => self.triangle_pos.y += 0.1,
        InputAction::Backward => self.triangle_pos.y -= 0.1,
        _ => (),
      }
    }

    let program = &self.program;
    let triangle = &self.triangle;
    let triangle_pos = self.triangle_pos;

    ctx.with_framebuffer(&self.back_buffer, &PipelineState::default(), |mut frame| {
      frame.with_program(program, |mut frame| {
        frame.update(|mut program, unis| {
          program.set(&unis.time, &t)?;
          program.set(&unis.triangle_pos, triangle_pos.as_ref())
        })?;

        frame.with_render_state(&RenderState::default(), |mut frame| {
          frame.render_vertex_entity(triangle.view(..))
        })
      })
    })?;

    Ok(LoopFeedback::Continue(self))
  }
}
