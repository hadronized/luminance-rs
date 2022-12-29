//! This example shows you how to lookup dynamically uniforms into shaders to implement various kind
//! of situations. This feature is very likely to be interesting for anyone who would like to
//! implement a GUI, where the interface of the shader programs are not known statically, for
//! instance.
//!
//! This example looks up the time and the triangle position on the fly, without using the uniform
//! interface.
//!
//! Press the <left>, <right>, <up>, <down> actions to move the triangle on the screen.
//!
//! <https://docs.rs/luminance>

use crate::{
  shared::{FragSlot, Vertex},
  Example, InputAction, LoopFeedback, PlatformServices,
};
use luminance::{
  backend::Backend,
  context::Context,
  dim::{Dim2, Size2},
  framebuffer::{Back, Framebuffer},
  pipeline::PipelineState,
  primitive::Triangle,
  render_state::RenderState,
  shader::{Program, ProgramBuilder},
  vertex_entity::{VertexEntity, View},
  vertex_storage::Interleaved,
};

const VS: &'static str = include_str!("displacement-vs.glsl");
const FS: &'static str = include_str!("displacement-fs.glsl");

// Only one triangle this time.
const TRI_VERTICES: [Vertex; 3] = [
  // triangle – an RGB one
  //
  Vertex::new(
    mint::Vector2 { x: 0.5, y: -0.5 },
    mint::Vector3 {
      x: 0.,
      y: 1.,
      z: 0.,
    },
  ),
  Vertex::new(
    mint::Vector2 { x: 0., y: 0.5 },
    mint::Vector3 {
      x: 0.,
      y: 0.,
      z: 1.,
    },
  ),
  Vertex::new(
    mint::Vector2 { x: -0.5, y: -0.5 },
    mint::Vector3 {
      x: 1.,
      y: 0.,
      z: 0.,
    },
  ),
];

pub struct LocalExample {
  program: Program<Vertex, (), Triangle, FragSlot, ()>, // no uniform environment; we want dynamic lookups
  triangle: VertexEntity<Vertex, Triangle, Interleaved<Vertex>>,
  triangle_pos: mint::Vector2<f32>,
  back_buffer: Framebuffer<Dim2, Back<FragSlot>, Back<()>>,
}

impl Example for LocalExample {
  type Err = luminance::backend::Error;

  const TITLE: &'static str = "Dynamic uniform lookup";

  fn bootstrap(
    [width, height]: [u32; 2],
    _: &mut impl PlatformServices,
    ctx: &mut Context<impl Backend>,
  ) -> Result<Self, Self::Err> {
    // notice that we don’t set a uniform interface here: we’re going to look it up on the fly
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
    let triangle_pos = mint::Vector2 { x: 0., y: 0. };
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

        InputAction::Left => {
          self.triangle_pos.x -= 0.1;
        }

        InputAction::Right => {
          self.triangle_pos.x += 0.1;
        }

        InputAction::Forward => {
          self.triangle_pos.y += 0.1;
        }

        InputAction::Backward => {
          self.triangle_pos.y -= 0.1;
        }

        _ => (),
      }
    }

    let program = &self.program;
    let triangle = &self.triangle;
    let triangle_pos = &self.triangle_pos;

    ctx.with_framebuffer(&self.back_buffer, &PipelineState::default(), |mut frame| {
      frame.with_program(program, |mut frame| {
        frame.update(|mut update, _| {
          update.query_set::<f32>("t", &t)?;
          update.query_set::<mint::Vector2<f32>>("triangle_pos", triangle_pos)
        })?;

        frame.with_render_state(&RenderState::default(), |mut frame| {
          frame.render_vertex_entity(triangle.view(..))
        })
      })
    })?;

    Ok(LoopFeedback::Continue(self))
  }
}
