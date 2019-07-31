//! > This program is a sequel to 08-shader-uniforms-adapt. Be sure to have read it first.
//!
//! This example shows you how to lookup dynamically uniforms into shaders to implement various kind
//! of situations. This feature is very likely to be interesting for anyone who would like to
//! implement a GUI, where the interface of the shader programs are not known statically, for
//! instance.
//!
//! This example looks up the time and the triangle position on the fly, without using the uniform
//! interface.
//!
//! Press the <a>, <s>, <d>, <z> or the arrow keys to move the triangle on the screen.
//! Press <escape> to quit or close the window.
//!
//! https://docs.rs/luminance

mod common;

use crate::common::{Semantics, Vertex, VertexPosition, VertexColor};
use luminance::context::GraphicsContext;
use luminance::framebuffer::Framebuffer;
use luminance::render_state::RenderState;
use luminance::shader::program::Program;
use luminance::tess::{Mode, TessBuilder};
use luminance_glfw::event::{Action, Key, WindowEvent};
use luminance_glfw::surface::{GlfwSurface, Surface, WindowDim, WindowOpt};
use std::time::Instant;

const VS: &'static str = include_str!("displacement-vs.glsl");
const FS: &'static str = include_str!("displacement-fs.glsl");

// Only one triangle this time.
const TRI_VERTICES: [Vertex; 3] = [
  Vertex { pos: VertexPosition::new([0.5, -0.5]), rgb: VertexColor::new([1., 0., 0.]) },
  Vertex { pos: VertexPosition::new([0.0, 0.5]), rgb: VertexColor::new([0., 1., 0.]) },
  Vertex { pos: VertexPosition::new([-0.5, -0.5]), rgb: VertexColor::new([0., 0., 1.]) },
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
    .set_mode(Mode::Triangle)
    .build()
    .unwrap();

  let mut back_buffer = Framebuffer::back_buffer(surface.size());

  let mut triangle_pos = [0., 0.];

  let start_t = Instant::now();

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

        WindowEvent::FramebufferSize(width, height) => {
          back_buffer = Framebuffer::back_buffer([width as u32, height as u32]);
        }

        _ => (),
      }
    }

    let elapsed = start_t.elapsed();
    let t64 = elapsed.as_secs() as f64 + (elapsed.subsec_millis() as f64 * 1e-3);
    let t = t64 as f32;

    surface
      .pipeline_builder()
      .pipeline(&back_buffer, [0., 0., 0., 0.], |_, shd_gate| {
        shd_gate.shade(&program, |rdr_gate, iface| {
          let query = iface.query();

          if let Ok(time_u) = query.ask("t") {
            time_u.update(t);
          }

          if let Ok(triangle_pos_u) = query.ask("triangle_pos") {
            triangle_pos_u.update(triangle_pos);
          }

          // the `ask` function is type-safe: if you try to get a uniform which type is not
          // correctly reified from the source, you get a TypeMismatch runtime error
          //if let Err(e) = query.ask::<i32>("triangle_pos") {
          //  eprintln!("{:?}", e);
          //}

          rdr_gate.render(RenderState::default(), |tess_gate| {
            tess_gate.render(&mut surface, (&triangle).into());
          });
        });
      });

    surface.swap_buffers();
  }
}
