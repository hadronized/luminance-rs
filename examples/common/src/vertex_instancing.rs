//! This program shows you how to do *vertex instancing*, the easy way.
//!
//! <https://docs.rs/luminance>

use crate::{
  shared::{FragSlot, Instance, Vertex},
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
  shader::{Program, ProgramBuilder, Stage},
  vertex_entity::{VertexEntity, View},
  vertex_storage::Interleaved,
};

const VS: &'static str = include_str!("instancing-vs.glsl");
const FS: &'static str = include_str!("instancing-fs.glsl");

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

// Instances. We’ll be using five triangles.
const INSTANCES: [Instance; 5] = [
  Instance {
    position: mint::Vector2 { x: 0., y: 0. },
    weight: 1.,
  },
  Instance {
    position: mint::Vector2 { x: -0.5, y: 0.5 },
    weight: 1.,
  },
  Instance {
    position: mint::Vector2 { x: -0.25, y: -0.1 },
    weight: 1.,
  },
  Instance {
    position: mint::Vector2 { x: 0.45, y: 0.25 },
    weight: 1.,
  },
  Instance {
    position: mint::Vector2 { x: 0.6, y: -0.3 },
    weight: 1.,
  },
];

pub struct LocalExample {
  program: Program<Vertex, Triangle<Vertex>, FragSlot, ()>,
  triangle:
    VertexEntity<Vertex, Triangle<Vertex>, Interleaved<Vertex>, Instance, Interleaved<Instance>>,
  back_buffer: Framebuffer<Dim2, Back<FragSlot>, Back<()>>,
}

impl Example for LocalExample {
  type Err = luminance::backend::Error;

  const TITLE: &'static str = "Vertex instancing";

  fn bootstrap(
    [width, height]: [u32; 2],
    _: &mut impl PlatformServices,
    ctx: &mut Context<impl Backend>,
  ) -> Result<Self, Self::Err> {
    let program = ctx.new_program(
      ProgramBuilder::new()
        .add_vertex_stage(Stage::<Vertex, Vertex, ()>::new(VS))
        .no_primitive_stage()
        .add_shading_stage(Stage::<Vertex, FragSlot, ()>::new(FS)),
    )?;

    let triangle = ctx.new_vertex_entity(
      Interleaved::new().set_vertices(&TRI_VERTICES[..]),
      [],
      Interleaved::new().set_vertices(&INSTANCES[..]),
    )?;

    let back_buffer = ctx.back_buffer(Size2::new(width, height))?;

    Ok(Self {
      program,
      triangle,
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

        _ => (),
      }
    }

    // make instances go boop boop by changing their weight dynamically
    let instances = self.triangle.instance_data().vertices_mut();

    for (i, instance) in instances.iter_mut().enumerate() {
      let tcos = (t * (i + 1) as f32 * 0.5).cos().powf(2.);
      instance.weight = tcos;
    }

    ctx.update_instance_data(&mut self.triangle)?;

    let program = &self.program;
    let triangle = &self.triangle;

    ctx.with_framebuffer(&self.back_buffer, &PipelineState::default(), |mut frame| {
      frame.with_program(program, |mut frame| {
        frame.update(|mut update, _| update.query_set::<f32>("t", &t))?;

        frame.with_render_state(&RenderState::default(), |mut frame| {
          frame.render_vertex_entity(triangle.view(..).set_instance_count(5))
        })
      })
    })?;

    Ok(LoopFeedback::Continue(self))
  }
}
