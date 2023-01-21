//! This program shows how to tweak the render state in order to render two simple triangles with
//! different parameters.
//!
//! From this tutorial on, vertex types and semantics are taken from a common.rs file.
//!
//! Press the <main action> to switch which triangle is rendered atop of which.
//! Press the <auxiliary action> to activate additive blending or disable it.
//!
//! <https://docs.rs/luminance>

use luminance::{
  backend::Backend,
  blending::{Blending, BlendingMode, Equation, Factor},
  context::Context,
  depth_stencil::DepthTest,
  dim::{Dim2, Size2},
  framebuffer::{Back, Framebuffer},
  pipeline::PipelineState,
  primitive::Triangle,
  render_state::RenderState,
  shader::{Program, ProgramBuilder},
  vertex_entity::{VertexEntity, VertexEntityBuilder, View},
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

// Convenience type to demonstrate how the depth test influences the rendering of two triangles.
#[derive(Copy, Clone, Debug)]
enum DepthMethod {
  Under, // draw the red triangle under the blue one
  Atop,  // draw the red triangle atop the blue one
}

impl DepthMethod {
  fn toggle(self) -> Self {
    match self {
      DepthMethod::Under => DepthMethod::Atop,
      DepthMethod::Atop => DepthMethod::Under,
    }
  }
}

// toggle between no blending and additive blending
fn toggle_blending(blending: BlendingMode) -> BlendingMode {
  match blending {
    BlendingMode::Off => BlendingMode::Combined(Blending {
      equation: Equation::Additive,
      src: Factor::One,
      dst: Factor::One,
    }),

    _ => BlendingMode::Off,
  }
}

pub struct LocalExample {
  program: Program<Vertex, (), Triangle, FragSlot, ()>,
  red_triangle: VertexEntity<Vertex, Triangle, Interleaved<Vertex>>,
  blue_triangle: VertexEntity<Vertex, Triangle, Interleaved<Vertex>>,
  blending: BlendingMode,
  depth_method: DepthMethod,
  back_buffer: Framebuffer<Dim2, Back<FragSlot>, Back<()>>,
}

impl Example for LocalExample {
  type Err = luminance::backend::Error;

  const TITLE: &'static str = "Render State";

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

    // create a red and blue triangles
    let red_triangle = ctx.new_vertex_entity(
      VertexEntityBuilder::new()
        .add_vertices(Interleaved::new().set_vertices(&TRI_RED_BLUE_VERTICES[0..3])),
    )?;

    let blue_triangle = ctx.new_vertex_entity(
      VertexEntityBuilder::new()
        .add_vertices(Interleaved::new().set_vertices(&TRI_RED_BLUE_VERTICES[3..6])),
    )?;

    let blending = BlendingMode::Off;
    let depth_method = DepthMethod::Under;
    let back_buffer = ctx.back_buffer(Size2::new(width, height))?;

    Ok(Self {
      program,
      red_triangle,
      blue_triangle,
      blending,
      depth_method,
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
          self.depth_method = self.depth_method.toggle();
          log::info!("now rendering {:?}", self.depth_method);
        }

        InputAction::AuxiliaryToggle => {
          self.blending = toggle_blending(self.blending);
          log::info!("now blending with {:?}", self.blending);
        }

        _ => (),
      }
    }

    let back_buffer = &self.back_buffer;
    let program = &mut self.program;
    let red_triangle = &self.red_triangle;
    let blue_triangle = &self.blue_triangle;
    let blending = self.blending;
    let depth_method = self.depth_method;

    let render_state = RenderState::default()
            // let’s disable the depth test so that every fragment (i.e. pixels) will be rendered to every
            // time we have to draw a part of a triangle
            .set_depth_test(DepthTest::Off)
            // set the blending we decided earlier
            .set_blending(blending);

    ctx.with_framebuffer(back_buffer, &PipelineState::default(), |mut frame| {
      frame.with_program(program, |mut frame| {
        frame.with_render_state(&render_state, |mut frame| match depth_method {
          DepthMethod::Under => {
            frame.render_vertex_entity(red_triangle.view(..))?;
            frame.render_vertex_entity(blue_triangle.view(..))
          }

          DepthMethod::Atop => {
            frame.render_vertex_entity(blue_triangle.view(..))?;
            frame.render_vertex_entity(red_triangle.view(..))
          }
        })
      })
    })?;

    Ok(LoopFeedback::Continue(self))
  }
}
