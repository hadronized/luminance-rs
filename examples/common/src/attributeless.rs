//! This program demonstrates how to render a triangle without sending anything to the GPU. This is
//! a not-so-famous technique to reduce the bandwidth and procedurally generate all the required
//! data to perform the render. The trick lives in ordering the GPU to render a certain number of
//! vertices and “spawn” the vertices’ data directly in the vertex shader by using the identifier of
//! the vertex currently being mapped over.
//!
//! Press <escape> to quit or close the window.
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
  shader::{Program, ProgramBuilder},
  vertex_entity::{VertexEntity, VertexEntityBuilder, View},
};

use crate::{shared::FragSlot, Example, InputAction, LoopFeedback, PlatformServices};

const VS: &'static str = include_str!("attributeless-vs.glsl");
const FS: &'static str = include_str!("simple-fs.glsl");

pub struct LocalExample {
  program: Program<(), (), Triangle, FragSlot, ()>,
  attributeless: VertexEntity<(), Triangle, ()>,
  back_buffer: Framebuffer<Dim2, Back<FragSlot>, Back<()>>,
}

impl Example for LocalExample {
  type Err = luminance::backend::Error;

  const TITLE: &'static str = "Attributeless";

  fn bootstrap(
    [width, height]: [u32; 2],
    _platform: &mut impl PlatformServices,
    ctx: &mut Context<impl Backend>,
  ) -> Result<Self, Self::Err> {
    // we don’t use a Vertex type anymore (i.e. attributeless, so we use the unit () type)
    let program = ctx.new_program(
      ProgramBuilder::new()
        .add_vertex_stage(VS)
        .no_primitive_stage()
        .add_shading_stage(FS),
    )?;

    // yet, we still need to tell luminance to render a certain number of vertices (even if we send no
    // attributes / data); in our case, we’ll just render a triangle, which has three vertices
    let attributeless = ctx.new_vertex_entity(VertexEntityBuilder::new())?;

    let back_buffer = ctx.back_buffer(Size2::new(width, height))?;

    Ok(Self {
      program,
      attributeless,
      back_buffer,
    })
  }

  fn render_frame(
    mut self,
    _time: f32,
    actions: impl Iterator<Item = InputAction>,
    ctx: &mut Context<impl Backend>,
  ) -> Result<LoopFeedback<Self>, Self::Err> {
    for action in actions {
      match action {
        InputAction::Quit => return Ok(LoopFeedback::Exit),
        _ => (),
      }
    }

    let program = &mut self.program;
    let attributeles = &self.attributeless;

    ctx.with_framebuffer(&self.back_buffer, &PipelineState::default(), |mut frame| {
      frame.with_program(program, |mut frame| {
        frame.with_render_state(&RenderState::default(), |mut frame| {
          frame.render_vertex_entity(attributeles.view(..3))
        })
      })
    })?;

    Ok(LoopFeedback::Continue(self))
  }
}
