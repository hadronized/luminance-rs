//! This example demonstrates how to _map_ GPU tessellations to alter their attributes. It consists
//! of a triangle that you can move around by grabbing one of its three vertices and moving them
//! around.
//!
//! Press down the left click of your mouse / trackpad when your mouse is close to a vertex to grab
//! it, move it around and release the left click to put it at the place your cursor is.
//! Press <escape> to quit or close the window.
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
use mint::{Vector2, Vector3};

const VS: &str = include_str!("simple-vs.glsl");
const FS: &str = include_str!("simple-fs.glsl");

const TRI_VERTICES: [Vertex; 3] = [
  Vertex::new(
    Vector2 { x: 0.5, y: -0.5 },
    Vector3 {
      x: 0.,
      y: 1.,
      z: 0.,
    },
  ),
  Vertex::new(
    Vector2 { x: 0.0, y: 0.5 },
    Vector3 {
      x: 0.,
      y: 0.,
      z: 1.,
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
];

// when selecting a vertex, the vertex is “snapped” only if the distance (in pixels) between the
// position of the click and the vertex is minimal to this value (expressed in pixels, since it’s a
// distance)
const CLICK_RADIUS_PX: f32 = 20.;

// a simple convenient function to compute a distance between two [f32; 2]
fn distance(a: &Vector2<f32>, b: &Vector2<f32>) -> f32 {
  let x = b.x - a.x;
  let y = b.y - b.y;

  (x * x + y * y).sqrt()
}

// convert from screen space to window space
fn screen_to_window(a: &Vector2<f32>, w: f32, h: f32) -> Vector2<f32> {
  Vector2 {
    x: (1. + a.x) * 0.5 * w as f32,
    y: (1. - a.y) * 0.5 * h as f32,
  }
}

// convert from window space to screen space
fn window_to_screen(a: &Vector2<f32>, w: f32, h: f32) -> Vector2<f32> {
  Vector2 {
    x: a.x / w * 2. - 1.,
    y: 1. - a.y / h * 2.,
  }
}

pub struct LocalExample {
  program: Program<Vertex, (), Triangle, FragSlot, ()>,
  triangle: VertexEntity<Vertex, Triangle, Interleaved<Vertex>>,
  // current cursor position
  cursor_pos: Option<Vector2<f32>>,
  // when we press down a button, if we are to select a vertex, we need to know which one; this
  // variable contains its index (0, 1 or 2)
  selected: Option<usize>,
  // used to perform window-space / screen-space coordinates conversion
  window_dim: [f32; 2],
  back_buffer: Framebuffer<Dim2, Back<FragSlot>, Back<()>>,
}

impl Example for LocalExample {
  type Err = luminance::backend::Error;

  const TITLE: &'static str = "Interactive Triangle";

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

    let triangle = ctx.new_vertex_entity(
      Interleaved::new().set_vertices(TRI_VERTICES),
      [],
      Interleaved::new(),
    )?;

    let cursor_pos = None;
    let selected = None;

    let back_buffer = ctx.back_buffer(Size2::new(width, height))?;

    Ok(Self {
      program,
      triangle,
      cursor_pos,
      selected,
      window_dim: [width as f32, height as f32],
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
        InputAction::Quit => return Ok(LoopFeedback::Exit),

        // if we press down the primary action, we want to check whether a vertex is nearby; to do so,
        // we map the triangle’s vertices and look for one; we take the one with the minimal
        // distance that satisfies the distance rule defined at the top of this file
        // (CLICK_RADIUS_PX)
        InputAction::PrimaryPressed => {
          if let Some(ref cursor_pos) = self.cursor_pos {
            let vertices = self.triangle.vertices().vertices();

            for i in 0..3 {
              let [w, h] = self.window_dim;

              // convert the vertex position from screen space into window space
              let ws_pos = screen_to_window(&vertices[i].co, w, h);

              if distance(&ws_pos, cursor_pos) <= CLICK_RADIUS_PX {
                println!("selecting vertex i={}", i);
                self.selected = Some(i);
                break;
              }
            }
          }
        }

        InputAction::PrimaryReleased => {
          self.selected = None;
        }

        // whenever the cursor moves, if we have a selection, we set the position of that vertex to
        // the position of the cursor, and synchronize the GPU vertex entity
        InputAction::CursorMoved { x, y } => {
          let pos = Vector2 { x, y };
          self.cursor_pos = Some(pos);

          if let Some(selected) = self.selected {
            let vertices = self.triangle.vertices().vertices_mut();
            let [w, h] = self.window_dim;

            vertices[selected].co = window_to_screen(&pos, w, h);

            // update the vertices on the GPU
            ctx.update_vertices(&mut self.triangle)?;
          }
        }

        InputAction::Resized { width, height } => {
          self.window_dim = [width as _, height as _];
        }

        _ => (),
      }
    }

    let program = &self.program;
    let triangle = &self.triangle;

    ctx.with_framebuffer(&self.back_buffer, &PipelineState::default(), |mut frame| {
      frame.with_program(program, |mut frame| {
        frame.with_render_state(&RenderState::default(), |mut frame| {
          frame.render_vertex_entity(triangle.view(..))
        })
      })
    })?;

    Ok(LoopFeedback::Continue(self))
  }
}
