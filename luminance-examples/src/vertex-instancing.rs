//! This program shows you how to do *vertex instancing*, the easy way.
//!
//! Press <escape> to quit or close the window.
//!
//! https://docs.rs/luminance

mod common;

use crate::common::{
  Instance, Semantics, Vertex, VertexColor, VertexInstancePosition, VertexPosition, VertexWeight,
};
use glfw::{Action, Context as _, Key, WindowEvent};
use luminance::context::GraphicsContext as _;
use luminance::pipeline::PipelineState;
use luminance::render_state::RenderState;
use luminance::tess::Mode;
use luminance_glfw::GlfwSurface;
use luminance_windowing::{WindowDim, WindowOpt};
use std::time::Instant;

const VS: &str = include_str!("instancing-vs.glsl");
const FS: &str = include_str!("instancing-fs.glsl");

// Only one triangle this time.
const TRI_VERTICES: [Vertex; 3] = [
  Vertex {
    pos: VertexPosition::new([0.5, -0.5]),
    rgb: VertexColor::new([1., 0., 0.]),
  },
  Vertex {
    pos: VertexPosition::new([0.0, 0.5]),
    rgb: VertexColor::new([0., 1., 0.]),
  },
  Vertex {
    pos: VertexPosition::new([-0.5, -0.5]),
    rgb: VertexColor::new([0., 0., 1.]),
  },
];

// Instances. We’ll be using five triangles.
const INSTANCES: [Instance; 5] = [
  Instance {
    pos: VertexInstancePosition::new([0., 0.]),
    w: VertexWeight::new(0.1),
  },
  Instance {
    pos: VertexInstancePosition::new([-0.5, 0.5]),
    w: VertexWeight::new(0.5),
  },
  Instance {
    pos: VertexInstancePosition::new([-0.25, -0.1]),
    w: VertexWeight::new(0.1),
  },
  Instance {
    pos: VertexInstancePosition::new([0.45, 0.25]),
    w: VertexWeight::new(0.75),
  },
  Instance {
    pos: VertexInstancePosition::new([0.6, -0.3]),
    w: VertexWeight::new(0.3),
  },
];

fn main() {
  let dim = WindowDim::Windowed {
    width: 960,
    height: 540,
  };
  let mut surface = GlfwSurface::new_gl33("Hello, world!", WindowOpt::default().set_dim(dim))
    .expect("GLFW surface creation");

  // notice that we don’t set a uniform interface here: we’re going to look it up on the fly
  let mut program = surface
    .new_shader_program::<Semantics, (), ()>()
    .from_strings(VS, None, None, FS)
    .expect("program creation")
    .ignore_warnings();

  let triangle = surface
    .new_tess()
    .set_vertices(&TRI_VERTICES[..])
    .set_instances(&INSTANCES[..])
    .set_mode(Mode::Triangle)
    .build()
    .unwrap();

  let mut back_buffer = surface.back_buffer().unwrap();

  let mut triangle_pos = [0., 0.];

  let start_t = Instant::now();
  let mut resize = false;

  'app: loop {
    surface.window.glfw.poll_events();
    for (_, event) in surface.events_rx.try_iter() {
      match event {
        WindowEvent::Close | WindowEvent::Key(Key::Escape, _, Action::Release, _) => break 'app,

        WindowEvent::Key(Key::A, _, action, _) | WindowEvent::Key(Key::Left, _, action, _)
          if action == Action::Press || action == Action::Repeat =>
        {
          triangle_pos[0] -= 0.1;
        }

        WindowEvent::Key(Key::D, _, action, _) | WindowEvent::Key(Key::Right, _, action, _)
          if action == Action::Press || action == Action::Repeat =>
        {
          triangle_pos[0] += 0.1;
        }

        WindowEvent::Key(Key::Z, _, action, _) | WindowEvent::Key(Key::Up, _, action, _)
          if action == Action::Press || action == Action::Repeat =>
        {
          triangle_pos[1] += 0.1;
        }

        WindowEvent::Key(Key::S, _, action, _) | WindowEvent::Key(Key::Down, _, action, _)
          if action == Action::Press || action == Action::Repeat =>
        {
          triangle_pos[1] -= 0.1;
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

    let elapsed = start_t.elapsed();
    let t64 = elapsed.as_secs() as f64 + (elapsed.subsec_millis() as f64 * 1e-3);
    let t = t64 as f32;

    let render = surface
      .new_pipeline_gate()
      .pipeline(
        &back_buffer,
        &PipelineState::default(),
        |_, mut shd_gate| {
          shd_gate.shade(&mut program, |mut iface, _, mut rdr_gate| {
            if let Ok(ref time_u) = iface.query().unwrap().ask("t") {
              iface.set(time_u, t);
            }

            rdr_gate.render(&RenderState::default(), |mut tess_gate| {
              tess_gate.render(&triangle)
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
