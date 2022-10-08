//! This program shows how to render two triangles that live in the same GPU tessellation. This is
//! called “tessellation views” in luminance and can help you implement plenty of situations. One
//! of the most interesting use case is for particles effect: you can allocate a big tessellation
//! object on the GPU and view it to render only the living particles.
//!
//! Press the <main action> to change the viewing method.
//!
//! <https://docs.rs/luminance>
//!
//! Bonus: for interested peeps, you’ll notice here the concept of slice. Unfortunately, the current
//! Index trait doesn’t allow us to use it (:(). More information on an RFC to try to change that
//! here:
//!
//! <https://github.com/rust-lang/rfcs/pull/2473>

use luminance::{
  backend::Backend,
  context::Context,
  dim::{Dim2, Size2},
  framebuffer::{Back, Framebuffer},
  pipeline::PipelineState,
  primitive::Triangle,
  render_state::RenderState,
  shader::{Program, ProgramBuilder, Stage},
  vertex_entity::{VertexEntity, View as _},
  vertex_storage::Interleaved,
};
use mint::{Vector2, Vector3};

use crate::{
  shared::{FragSlot, Vertex},
  Example, InputAction, LoopFeedback, PlatformServices,
};

const VS: &'static str = include_str!("simple-vs.glsl");
const FS: &'static str = include_str!("simple-fs.glsl");

pub const TRI_RED_BLUE_VERTICES: [Vertex; 6] = [
  // first triangle – a red one
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
      x: 1.,
      y: 0.,
      z: 0.,
    },
  ),
  Vertex::new(
    Vector2 { x: -0.5, y: -0.5 },
    Vector3 {
      x: 1.,
      y: 0.,
      z: 0.,
    },
  ),
  // second triangle, a blue one
  Vertex::new(
    Vector2 { x: -0.5, y: 0.5 },
    Vector3 {
      x: 0.,
      y: 0.,
      z: 1.,
    },
  ),
  Vertex::new(
    Vector2 { x: 0.0, y: -0.5 },
    Vector3 {
      x: 0.,
      y: 0.,
      z: 1.,
    },
  ),
  Vertex::new(
    Vector2 { x: 0.5, y: 0.5 },
    Vector3 {
      x: 0.,
      y: 0.,
      z: 1.,
    },
  ),
];

// Convenience type to select which view to render.
#[derive(Copy, Clone, Debug)]
enum ViewMethod {
  Red,  // draw the red triangle
  Blue, // draw the blue triangle
  Both, // draw both the triangles
}

impl ViewMethod {
  fn toggle(self) -> Self {
    match self {
      ViewMethod::Red => ViewMethod::Blue,
      ViewMethod::Blue => ViewMethod::Both,
      ViewMethod::Both => ViewMethod::Red,
    }
  }
}

pub struct LocalExample {
  program: Program<Vertex, Triangle<Vertex>, FragSlot, ()>,
  triangles: VertexEntity<Vertex, Triangle<Vertex>, Interleaved<Vertex>>,
  view_method: ViewMethod,
  back_buffer: Framebuffer<Dim2, Back<FragSlot>, Back<()>>,
}

impl Example for LocalExample {
  type Err = luminance::backend::Error;

  const TITLE: &'static str = "Sliced Tessellation";

  fn bootstrap(
    [width, height]: [u32; 2],
    _platform: &mut impl PlatformServices,
    ctx: &mut Context<impl Backend>,
  ) -> Result<Self, Self::Err> {
    let program = ctx.new_program(
      ProgramBuilder::new()
        .add_vertex_stage(Stage::<Vertex, Vertex, ()>::new(VS))
        .no_primitive_stage::<Triangle<Vertex>>()
        .add_shading_stage(Stage::<Vertex, FragSlot, ()>::new(FS)),
    )?;

    // create a single GPU tessellation that holds both the triangles (like in 01-hello-world)
    let triangles = ctx.new_vertex_entity(
      Interleaved::new().set_vertices(&TRI_RED_BLUE_VERTICES[..]),
      [],
    )?;

    let view_method = ViewMethod::Red;

    let back_buffer = ctx.back_buffer(Size2::new(width, height))?;

    Ok(LocalExample {
      program,
      triangles,
      view_method,
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
        InputAction::MainToggle => {
          self.view_method = self.view_method.toggle();
          log::info!("now rendering view {:?}", self.view_method);
        }

        _ => (),
      }
    }

    let back_buffer = &self.back_buffer;
    let program = &mut self.program;
    let triangles = &self.triangles;
    let view_method = self.view_method;

    ctx.with_framebuffer(back_buffer, &PipelineState::default(), |mut frame| {
      frame.with_program(program, |mut frame| {
        frame.with_render_state(&RenderState::default(), |mut frame| {
          let view = match view_method {
            // the red triangle is at slice [..3]; you can also use the TessView::sub
            // combinator if the start element is 0; it’s also possible to use [..=2] for
            // inclusive ranges
            ViewMethod::Red => triangles.view(..3),
            // the blue triangle is at slice [3..]
            ViewMethod::Blue => triangles.view(3..),
            // both triangles are at slice [0..6] or [..], but we’ll use the faster
            // TessView::whole combinator; this combinator is also if you invoke the From or
            // Into method on (&triangles) (we did that in 02-render-state)
            ViewMethod::Both => triangles.view(..), // TessView::whole(&triangles)
          };

          frame.render_vertex_entity(view)
        })
      })
    })?;

    Ok(LoopFeedback::Continue(self))
  }
}
