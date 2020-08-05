//! This program shows how to render a triangle and change its position and color on the fly by
//! updating “shader uniforms”. Those are values stored on the GPU that remain constant for the
//! whole duration of a draw call (you typically change it between each draw call to customize each
//! draw).
//!
//! This example demonstrate how to add time to your shader to start building moving and animated
//! effects.
//!
//! Press the <a>, <s>, <d>, <z> or the arrow keys to move the triangle on the screen.
//! Press <escape> to quit or close the window.
//!
//! https://docs.rs/luminance

mod common;

use crate::common::{Semantics, Vertex, VertexColor, VertexPosition};
use glfw::{Action, Context as _, Key, WindowEvent};
use luminance::context::GraphicsContext as _;
use luminance::pipeline::PipelineState;
use luminance::render_state::RenderState;
use luminance::shader::Uniform;
use luminance::tess::Mode;
use luminance_derive::UniformInterface;
use luminance_glfw::GlfwSurface;
use luminance_windowing::{WindowDim, WindowOpt};
use std::time::Instant;

const VS: &str = include_str!("displacement-vs.glsl");
const FS: &str = include_str!("displacement-fs.glsl");

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

// Create a uniform interface. This is a type that will be used to customize the shader. In our
// case, we just want to pass the time and the position of the triangle, for instance.
//
// This macro only supports structs for now; you cannot use enums as uniform interfaces.
#[derive(Debug, UniformInterface)]
struct ShaderInterface {
  #[uniform(name = "t")]
  time: Uniform<f32>,
  triangle_pos: Uniform<[f32; 2]>,
}

fn main() {
  let dim = WindowDim::Windowed {
    width: 960,
    height: 540,
  };
  let mut surface = GlfwSurface::new_gl33("Hello, world!", WindowOpt::default().set_dim(dim))
    .expect("GLFW surface creation");

  // see the use of our uniform interface here as thirds type variable
  let mut program = surface
    .new_shader_program::<Semantics, (), ShaderInterface>()
    .from_strings(VS, None, None, FS)
    .expect("program creation")
    .ignore_warnings();

  let triangle = surface
    .new_tess()
    .set_vertices(&TRI_VERTICES[..])
    .set_mode(Mode::Triangle)
    .build()
    .unwrap();

  let mut back_buffer = surface.back_buffer().unwrap();

  // position of the triangle
  let mut triangle_pos = [0., 0.];

  // reference time
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

    // get the current monotonic time
    let elapsed = start_t.elapsed();
    let t64 = elapsed.as_secs() as f64 + (elapsed.subsec_millis() as f64 * 1e-3);
    let t = t64 as f32;

    let render = surface
      .new_pipeline_gate()
      .pipeline(
        &back_buffer,
        &PipelineState::default(),
        |_, mut shd_gate| {
          // notice the iface free variable, which type is &ShaderInterface
          shd_gate.shade(&mut program, |mut iface, uni, mut rdr_gate| {
            // update the time and triangle position on the GPU shader program
            iface.set(&uni.time, t);
            iface.set(&uni.triangle_pos, triangle_pos);

            rdr_gate.render(&RenderState::default(), |mut tess_gate| {
              // render the dynamically selected slice
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
