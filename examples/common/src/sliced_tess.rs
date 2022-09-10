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

use crate::{
  shared::{Semantics, Vertex, VertexColor, VertexPosition},
  Example, InputAction, LoopFeedback, PlatformServices,
};
use luminance::vertex_entity::View as _;
use luminance_front::{
  context::GraphicsContext,
  framebuffer::Framebuffer,
  pipeline::PipelineState,
  render_state::RenderState,
  shader::Program,
  tess::{Mode, Tess},
  texture::Dim2,
  Backend,
};

const VS: &'static str = include_str!("simple-vs.glsl");
const FS: &'static str = include_str!("simple-fs.glsl");

pub const TRI_RED_BLUE_VERTICES: [Vertex; 6] = [
  // first triangle – a red one
  Vertex {
    pos: VertexPosition::new([0.5, -0.5]),
    rgb: VertexColor::new([1., 0., 0.]),
  },
  Vertex {
    pos: VertexPosition::new([0.0, 0.5]),
    rgb: VertexColor::new([1., 0., 0.]),
  },
  Vertex {
    pos: VertexPosition::new([-0.5, -0.5]),
    rgb: VertexColor::new([1., 0., 0.]),
  },
  // second triangle, a blue one
  Vertex {
    pos: VertexPosition::new([-0.5, 0.5]),
    rgb: VertexColor::new([0., 0., 1.]),
  },
  Vertex {
    pos: VertexPosition::new([0.0, -0.5]),
    rgb: VertexColor::new([0., 0., 1.]),
  },
  Vertex {
    pos: VertexPosition::new([0.5, 0.5]),
    rgb: VertexColor::new([0., 0., 1.]),
  },
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
  program: Program<Semantics, (), ()>,
  triangles: Tess<Vertex>,
  view_method: ViewMethod,
}

impl Example for LocalExample {
  fn bootstrap(
    _platform: &mut impl PlatformServices,
    context: &mut impl GraphicsContext<Backend = Backend>,
  ) -> Self {
    let program = context
      .new_shader_program::<Semantics, (), ()>()
      .from_strings(VS, None, None, FS)
      .expect("program creation")
      .ignore_warnings();

    // create a single GPU tessellation that holds both the triangles (like in 01-hello-world)
    let triangles = context
      .new_tess()
      .set_vertices(&TRI_RED_BLUE_VERTICES[..])
      .set_mode(Mode::Triangle)
      .build()
      .unwrap();

    let view_method = ViewMethod::Red;

    LocalExample {
      program,
      triangles,
      view_method,
    }
  }

  fn render_frame(
    mut self,
    _time: f32,
    back_buffer: Framebuffer<Dim2, (), ()>,
    actions: impl Iterator<Item = InputAction>,
    context: &mut impl GraphicsContext<Backend = Backend>,
  ) -> LoopFeedback<Self> {
    for action in actions {
      match action {
        InputAction::Quit => return LoopFeedback::Exit,
        InputAction::MainToggle => {
          self.view_method = self.view_method.toggle();
          log::info!("now rendering view {:?}", self.view_method);
        }

        _ => (),
      }
    }

    let program = &mut self.program;
    let triangles = &self.triangles;
    let view_method = self.view_method;

    let render = context
      .new_pipeline_gate()
      .pipeline(
        &back_buffer,
        &PipelineState::default(),
        |_, mut shd_gate| {
          shd_gate.shade(program, |_, _, mut rdr_gate| {
            rdr_gate.render(&RenderState::default(), |mut tess_gate| {
              let view = match view_method {
                // the red triangle is at slice [..3]; you can also use the TessView::sub
                // combinator if the start element is 0; it’s also possible to use [..=2] for
                // inclusive ranges
                ViewMethod::Red => triangles.view(..3), // TessView::slice(&triangles, 0, 3),
                // the blue triangle is at slice [3..]
                ViewMethod::Blue => triangles.view(3..), // TessView::slice(&triangles, 3, 6),
                // both triangles are at slice [0..6] or [..], but we’ll use the faster
                // TessView::whole combinator; this combinator is also if you invoke the From or
                // Into method on (&triangles) (we did that in 02-render-state)
                ViewMethod::Both => triangles.view(..), // TessView::whole(&triangles)
              }
              .unwrap();

              // render the dynamically selected view
              tess_gate.render(view)
            })
          })
        },
      )
      .assume();

    if render.is_ok() {
      LoopFeedback::Continue(self)
    } else {
      LoopFeedback::Exit
    }
  }
}
