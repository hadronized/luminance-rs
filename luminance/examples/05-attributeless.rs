//! This program demonstrates how to render a triangle without sending anything to the GPU. This is
//! a not-so-famous technique to reduce the bandwidth and procedurally generate all the required
//! data to perform the render. The trick lives in ordering the GPU to render a certain number of
//! vertices and “spawn” the vertices’ data directly in the vertex shader by using the identifier of
//! the vertex currently being mapped over.
//!
//! Press <escape> to quit or close the window.
//!
//! https://docs.rs/luminance

use luminance::context::GraphicsContext as _;
use luminance::framebuffer::Framebuffer;
use luminance::render_state::RenderState;
use luminance::shader::program::Program;
use luminance::tess::{Mode, TessBuilder};
use luminance_glfw::{Action, GlfwSurface, Key, Surface, WindowEvent, WindowDim, WindowOpt};

const VS: &'static str = include_str!("attributeless-vs.glsl");
const FS: &'static str = include_str!("simple-fs.glsl");

fn main() {
  let mut surface = GlfwSurface::new(
    WindowDim::Windowed(960, 540),
    "Hello, world!",
    WindowOpt::default(),
  )
  .expect("GLFW surface creation");

  // we don’t use a Vertex type anymore (i.e. attributeless, so we use the unit () type)
  let (program, _) = Program::<(), (), ()>::from_strings(None, VS, None, FS).expect("program creation");

  // yet, we still need to tell luminance to render a certain number of vertices (even if we send no
  // attributes / data); in our case, we’ll just render a triangle, which has three vertices
  let tess = TessBuilder::new(&mut surface)
    .set_vertex_nb(3)
    .set_mode(Mode::Triangle)
    .build()
    .unwrap();

  let mut back_buffer = surface.back_buffer().unwrap();
  let mut resize = false;

  'app: loop {
    for event in surface.poll_events() {
      match event {
        WindowEvent::Close | WindowEvent::Key(Key::Escape, _, Action::Release, _) => break 'app,

        WindowEvent::FramebufferSize(..) => {
          resize = true;
        }

        _ => (),
      }
    }

    if resize {
      back_buffer = surface.back_buffer().unwrap();
      resize = true;
    }

    surface
      .pipeline_builder()
      .pipeline(&back_buffer, [0., 0., 0., 0.], |_, shd_gate| {
        shd_gate.shade(&program, |_, rdr_gate| {
          rdr_gate.render(RenderState::default(), |tess_gate| {
            // render the tessellation to the surface the regular way and let the vertex shader’s
            // magic do the rest!
            tess_gate.render(&mut surface, &tess);
          });
        });
      });

    surface.swap_buffers();
  }
}
