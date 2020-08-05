//! This program shows how to render two triangles that live in the same GPU tessellation. This is
//! called “tessellation views” in luminance and can help you implement plenty of situations. One
//! of the most interesting use case is for particles effect: you can allocate a big tessellation
//! object on the GPU and view it to render only the living particles.
//!
//! Press <space> to change the viewing method.
//! Press <escape> to quit or close the window.
//!
//! https://docs.rs/luminance
//!
//! Bonus: for interested peeps, you’ll notice here the concept of slice. Unfortunately, the current
//! Index trait doesn’t allow us to use it (:(). More information on an RFC to try to change that
//! here:
//!
//! https://github.com/rust-lang/rfcs/pull/2473

mod common;

use crate::common::{Semantics, Vertex, VertexColor, VertexPosition};
use glfw::{Action, Context as _, Key, WindowEvent};
use luminance::context::GraphicsContext as _;
use luminance::pipeline::PipelineState;
use luminance::render_state::RenderState;
use luminance::tess::{Mode, View};
use luminance_glfw::GlfwSurface;
use luminance_windowing::{WindowDim, WindowOpt};

const VS: &str = include_str!("simple-vs.glsl");
const FS: &str = include_str!("simple-fs.glsl");

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

fn main() {
  let dim = WindowDim::Windowed {
    width: 960,
    height: 540,
  };
  let mut surface = GlfwSurface::new_gl33("Hello, world!", WindowOpt::default().set_dim(dim))
    .expect("GLFW surface creation");

  let mut program = surface
    .new_shader_program::<Semantics, (), ()>()
    .from_strings(VS, None, None, FS)
    .expect("program creation")
    .ignore_warnings();

  // create a single GPU tessellation that holds both the triangles (like in 01-hello-world)
  let triangles = surface
    .new_tess()
    .set_vertices(&TRI_RED_BLUE_VERTICES[..])
    .set_mode(Mode::Triangle)
    .build()
    .unwrap();

  let mut back_buffer = surface.back_buffer().unwrap();

  let mut view_method = ViewMethod::Red;
  println!("now rendering view {:?}", view_method);

  let mut resize = false;

  'app: loop {
    surface.window.glfw.poll_events();
    for (_, event) in surface.events_rx.try_iter() {
      match event {
        WindowEvent::Close | WindowEvent::Key(Key::Escape, _, Action::Release, _) => break 'app,

        WindowEvent::Key(Key::Space, _, Action::Release, _) => {
          view_method = view_method.toggle();
          println!("now rendering view {:?}", view_method);
        }

        WindowEvent::FramebufferSize(..) => {
          resize = true;
        }

        _ => (),
      }
    }

    if resize {
      back_buffer = surface.back_buffer().unwrap();
      resize = false;
    }

    let render = surface
      .new_pipeline_gate()
      .pipeline(
        &back_buffer,
        &PipelineState::default(),
        |_, mut shd_gate| {
          shd_gate.shade(&mut program, |_, _, mut rdr_gate| {
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
      surface.window.swap_buffers();
    } else {
      break 'app;
    }
  }
}
