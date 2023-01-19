//! This program shows how to use uniform buffers to pass large chunks of data to shaders, that can be shared between
//! shaders and changed only once (instead of for every shader programs that require that data).
//!
//! Uniform buffers are often used to implement geometry instancing and various kinds of effects requiring shared
//! data and/or large amount of it.
//!
//! <https://docs.rs/luminance>

use std::f32::consts::PI;

use luminance::{
  backend::{Backend, Error},
  context::Context,
  dim::{Dim2, Size2},
  framebuffer::{Back, Framebuffer},
  pipeline::PipelineState,
  primitive::TriangleFan,
  render_state::RenderState,
  shader::{Program, ProgramBuilder, Std140, Uni, UniBuffer},
  vertex_entity::{VertexEntity, View},
  vertex_storage::Interleaved,
  Std140, Uniforms,
};

use crate::{
  shared::{FragSlot, Vertex},
  Example, InputAction, LoopFeedback, PlatformServices,
};

const VS: &str = include_str!("./uni-buffer-2d-vs.glsl");
const FS: &str = include_str!("./simple-fs.glsl");

const VERTICES: [Vertex; 4] = [
  Vertex::new(
    mint::Vector2 { x: -0.01, y: -0.01 },
    mint::Vector3 {
      x: 0.5,
      y: 1.,
      z: 0.5,
    },
  ),
  Vertex::new(
    mint::Vector2 { x: 0.01, y: -0.01 },
    mint::Vector3 {
      x: 0.5,
      y: 1.,
      z: 0.5,
    },
  ),
  Vertex::new(
    mint::Vector2 { x: 0.01, y: 0.01 },
    mint::Vector3 {
      x: 0.5,
      y: 1.,
      z: 0.5,
    },
  ),
  Vertex::new(
    mint::Vector2 { x: -0.01, y: 0.01 },
    mint::Vector3 {
      x: 0.5,
      y: 1.,
      z: 0.5,
    },
  ),
];

#[derive(Uniforms)]
struct ShaderUniforms {
  #[uniform(name = "Positions")]
  positions: Uni<UniBuffer<UniBlock, Std140>>,
}

#[derive(Debug, Std140)]
pub struct UniBlock {
  p: [mint::Vector2<f32>; 100],
}

pub struct LocalExample {
  square: VertexEntity<Vertex, TriangleFan, Interleaved<Vertex>>,
  program: Program<Vertex, (), TriangleFan, FragSlot, ShaderUniforms>,
  uni_buffer: UniBuffer<UniBlock, Std140>,
  back_buffer: Framebuffer<Dim2, Back<FragSlot>, Back<()>>,
}

impl Example for LocalExample {
  type Err = Error;

  const TITLE: &'static str = "Uniform buffer";

  fn bootstrap(
    [width, height]: [u32; 2],
    _: &mut impl PlatformServices,
    ctx: &mut Context<impl Backend>,
  ) -> Result<Self, Self::Err> {
    let square = ctx.new_vertex_entity(
      Interleaved::new().set_vertices(VERTICES),
      [],
      Interleaved::new(),
    )?;

    let program = ctx.new_program(
      ProgramBuilder::new()
        .add_vertex_stage(VS)
        .no_primitive_stage()
        .add_shading_stage(FS),
    )?;

    let uni_buffer = ctx.new_uni_buffer(
      UniBlock {
        p: [mint::Vector2 { x: 0., y: 0. }; 100],
      }
      .into(),
    )?;

    let back_buffer = ctx.back_buffer(Size2::new(width, height))?;

    Ok(Self {
      square,
      program,
      uni_buffer,
      back_buffer,
    })
  }

  fn render_frame(
    mut self,
    time: f32,
    actions: impl Iterator<Item = InputAction>,
    ctx: &mut Context<impl Backend>,
  ) -> Result<LoopFeedback<Self>, Self::Err> {
    for action in actions {
      match action {
        InputAction::Quit => return Ok(LoopFeedback::Exit),

        InputAction::Resized { width, height } => {
          self.back_buffer = ctx.back_buffer(Size2::new(width, height))?;
        }

        _ => (),
      }
    }

    let square = &self.square;
    let program = &self.program;
    let uni_buffer = &self.uni_buffer;

    // update the positions of the squares
    {
      ctx
        .sync_uni_buffer(uni_buffer)?
        .p
        .iter_mut()
        .enumerate()
        .for_each(|(i, p)| {
          let i = i as f32;
          let phi = i * 2. * PI * 0.01 + time * 0.2;
          let radius = 0.8;

          p.x = phi.cos() * radius;
          p.y = (phi + i).sin() * radius;
        });
    }

    ctx.with_framebuffer(&self.back_buffer, &PipelineState::default(), |mut frame| {
      let in_use_uni_buffer = frame.use_uni_buffer(uni_buffer)?;

      frame.with_program(program, |mut frame| {
        frame.update(|mut update, unis| update.set(&unis.positions, &in_use_uni_buffer))?;

        frame.with_render_state(&RenderState::default(), |mut frame| {
          frame.render_vertex_entity(square.view(..).set_instance_count(100))
        })
      })
    })?;

    Ok(LoopFeedback::Continue(self))
  }
}
