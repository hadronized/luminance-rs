//! This program shows how to tweak the render state in order to render two simple triangles with
//! different parameters.
//!
//! From this tutorial on, vertex types and semantics are taken from a common.rs file.
//!
//! Press <space> to switch which triangle is rendered atop of which.
//! Press <b> to activate additive blending or disable it.
//! Press <escape> to quit or close the window.
//!
//! https://docs.rs/luminance

mod common;

use crate::common::{Semantics, Vertex, VertexPosition, VertexColor};
use luminance::blending::{Equation, Factor};
use luminance::context::GraphicsContext;
use luminance::depth_test::DepthTest;
use luminance::framebuffer::Framebuffer;
use luminance::render_state::RenderState;
use luminance::shader::program::Program;
use luminance::tess::{Mode, TessBuilder};
use luminance_glfw::event::{Action, Key, WindowEvent};
use luminance_glfw::surface::{GlfwSurface, Surface, WindowDim, WindowOpt};

const VS: &'static str = include_str!("simple-vs.glsl");
const FS: &'static str = include_str!("simple-fs.glsl");

pub const TRI_RED_BLUE_VERTICES: [Vertex; 6] = [
  // first triangle – a red one
  Vertex { pos: VertexPosition::new([0.5, -0.5]), rgb: VertexColor::new([1., 0., 0.]) },
  Vertex { pos: VertexPosition::new([0.0, 0.5]), rgb: VertexColor::new([1., 0., 0.]) },
  Vertex { pos: VertexPosition::new([-0.5, -0.5]), rgb: VertexColor::new([1., 0., 0.]) },
  // second triangle, a blue one
  Vertex { pos: VertexPosition::new([-0.5, 0.5]), rgb: VertexColor::new([0., 0., 1.]) },
  Vertex { pos: VertexPosition::new([0.0, -0.5]), rgb: VertexColor::new([0., 0., 1.]) },
  Vertex { pos: VertexPosition::new([0.5, 0.5]), rgb: VertexColor::new([0., 0., 1.]) },
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

type Blending = Option<(Equation, Factor, Factor)>;

// toggle between no blending and additive blending
fn toggle_blending(blending: Blending) -> Blending {
  match blending {
    None => Some((Equation::Additive, Factor::One, Factor::One)),
    _ => None,
  }
}

fn main() {
  let mut surface = GlfwSurface::new(
    WindowDim::Windowed(960, 540),
    "Hello, world!",
    WindowOpt::default(),
  )
  .expect("GLFW surface creation");

  let (program, _) = Program::<Semantics, (), ()>::from_strings(None, VS, None, FS).expect("program creation");

  // create a red and blue triangles
  let red_triangle = TessBuilder::new(&mut surface)
    .add_vertices(&TRI_RED_BLUE_VERTICES[0..3])
    .set_mode(Mode::Triangle)
    .build()
    .unwrap();
  let blue_triangle = TessBuilder::new(&mut surface)
    .add_vertices(&TRI_RED_BLUE_VERTICES[3..6])
    .set_mode(Mode::Triangle)
    .build()
    .unwrap();

  let mut back_buffer = Framebuffer::back_buffer(surface.size());

  let mut blending = None;
  let mut depth_method = DepthMethod::Under;
  println!("now rendering red triangle {:?} the blue one", depth_method);

  'app: loop {
    for event in surface.poll_events() {
      match event {
        WindowEvent::Close | WindowEvent::Key(Key::Escape, _, Action::Release, _) => break 'app,

        WindowEvent::Key(Key::Space, _, Action::Release, _) => {
          depth_method = depth_method.toggle();
          println!("now rendering red triangle {:?} the blue one", depth_method);
        }

        WindowEvent::Key(Key::B, _, Action::Release, _) => {
          blending = toggle_blending(blending);
          println!("now blending with {:?}", blending);
        }

        WindowEvent::FramebufferSize(width, height) => {
          back_buffer = Framebuffer::back_buffer([width as u32, height as u32]);
        }

        _ => (),
      }
    }

    surface
      .pipeline_builder()
      .pipeline(&back_buffer, [0., 0., 0., 0.], |_, shd_gate| {
        shd_gate.shade(&program, |rdr_gate, _| {
          let render_state = RenderState::default()
          // let’s disable the depth test so that every fragment (i.e. pixels) will rendered to every
          // time we have to draw a part of a triangle
          .set_depth_test(DepthTest::Off)
          // set the blending we decided earlier
          .set_blending(blending);

          rdr_gate.render(render_state, |tess_gate| match depth_method {
            DepthMethod::Under => {
              tess_gate.render(&mut surface, (&red_triangle).into());
              tess_gate.render(&mut surface, (&blue_triangle).into());
            }

            DepthMethod::Atop => {
              tess_gate.render(&mut surface, (&blue_triangle).into());
              tess_gate.render(&mut surface, (&red_triangle).into());
            }
          });
        });
      });

    surface.swap_buffers();
  }
}
