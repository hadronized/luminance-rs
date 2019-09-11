//! This program shows you how to do *vertex instancing*, the easy way.
//!
//! Press <escape> to quit or close the window.
//!
//! https://docs.rs/luminance

mod common;

use crate::common::{
  Instance, Semantics, Vertex, VertexPosition, VertexColor, VertexInstancePosition, VertexWeight
};
use luminance::context::GraphicsContext;
use luminance::framebuffer::Framebuffer;
use luminance::render_state::RenderState;
use luminance::shader::program::Program;
use luminance::tess::{Mode, TessBuilder};
use luminance_glfw::{Action, GlfwSurface, Key, Surface, WindowEvent, WindowDim, WindowOpt};
use std::time::Instant;

const VS: &'static str = include_str!("instancing-vs.glsl");
const FS: &'static str = include_str!("instancing-fs.glsl");

// Only one triangle this time.
const TRI_VERTICES: [Vertex; 3] = [
  Vertex { pos: VertexPosition::new([0.5, -0.5]), rgb: VertexColor::new([1., 0., 0.]) },
  Vertex { pos: VertexPosition::new([0.0, 0.5]), rgb: VertexColor::new([0., 1., 0.]) },
  Vertex { pos: VertexPosition::new([-0.5, -0.5]), rgb: VertexColor::new([0., 0., 1.]) },
];

// Instances. We’ll be using five triangles.
const INSTANCES: [Instance; 5] = [
  Instance { pos: VertexInstancePosition::new([0., 0.]), w: VertexWeight::new(0.1) },
  Instance { pos: VertexInstancePosition::new([-0.5, 0.5]), w: VertexWeight::new(0.5) },
  Instance { pos: VertexInstancePosition::new([-0.25, -0.1]), w: VertexWeight::new(0.1) },
  Instance { pos: VertexInstancePosition::new([0.45, 0.25]), w: VertexWeight::new(0.75) },
  Instance { pos: VertexInstancePosition::new([0.6, -0.3]), w: VertexWeight::new(0.3) },
];

fn main() {
  let mut surface = GlfwSurface::new(
    WindowDim::Windowed(960, 540),
    "Hello, world!",
    WindowOpt::default(),
  )
  .expect("GLFW surface creation");

  // notice that we don’t set a uniform interface here: we’re going to look it up on the fly
  let program = Program::<Semantics, (), ()>::from_strings(None, VS, None, FS)
    .expect("program creation")
    .0;

  let triangle = TessBuilder::new(&mut surface)
    .add_vertices(TRI_VERTICES)
    .add_instances(INSTANCES)
    .set_mode(Mode::Triangle)
    .build()
    .unwrap();

  let size = surface.size();
  let mut back_buffer = Framebuffer::back_buffer(&mut surface, size);

  let mut triangle_pos = [0., 0.];

  let start_t = Instant::now();
  let mut resize = false;

  'app: loop {
    for event in surface.poll_events() {
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
      let size = surface.size();
      back_buffer = Framebuffer::back_buffer(&mut surface, size);
      resize = false;
    }

    let elapsed = start_t.elapsed();
    let t64 = elapsed.as_secs() as f64 + (elapsed.subsec_millis() as f64 * 1e-3);
    let t = t64 as f32;

    surface
      .pipeline_builder()
      .pipeline(&back_buffer, [0., 0., 0., 0.], |_, shd_gate| {
        shd_gate.shade(&program, |iface, rdr_gate| {
          let query = iface.query();

          if let Ok(time_u) = query.ask("t") {
            time_u.update(t);
          }

          rdr_gate.render(RenderState::default(), |tess_gate| {
            tess_gate.render(&mut surface, &triangle);
          });
        });
      });

    surface.swap_buffers();
  }
}
