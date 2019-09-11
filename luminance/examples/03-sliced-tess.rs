//! This program shows how to render two triangles that live in the same GPU tessellation. This is
//! called “sliced tessellation” in luminance and can help you implement plenty of situations. One
//! of the most interesting use case is for particles effect: you can allocate a big tessellation
//! object on the GPU and slice it to render only the living particles.
//!
//! Press <space> to change the slicing method.
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

use crate::common::{Semantics, Vertex, VertexPosition, VertexColor};
use luminance::context::GraphicsContext as _;
use luminance::framebuffer::Framebuffer;
use luminance::render_state::RenderState;
use luminance::shader::program::Program;
use luminance::tess::{Mode, TessBuilder, TessSliceIndex};
use luminance_glfw::{Action, GlfwSurface, Key, Surface, WindowEvent, WindowDim, WindowOpt};

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

// Convenience type to select which slice to render.
#[derive(Copy, Clone, Debug)]
enum SliceMethod {
  Red,  // draw the red triangle
  Blue, // draw the blue triangle
  Both, // draw both the triangles
}

impl SliceMethod {
  fn toggle(self) -> Self {
    match self {
      SliceMethod::Red => SliceMethod::Blue,
      SliceMethod::Blue => SliceMethod::Both,
      SliceMethod::Both => SliceMethod::Red,
    }
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

  // create a single GPU tessellation that holds both the triangles (like in 01-hello-world)
  let triangles = TessBuilder::new(&mut surface)
    .add_vertices(TRI_RED_BLUE_VERTICES)
    .set_mode(Mode::Triangle)
    .build()
    .unwrap();

  let mut back_buffer = surface.back_buffer().unwrap();

  let mut slice_method = SliceMethod::Red;
  println!("now rendering slice {:?}", slice_method);

  let mut resize = false;

  'app: loop {
    for event in surface.poll_events() {
      match event {
        WindowEvent::Close | WindowEvent::Key(Key::Escape, _, Action::Release, _) => break 'app,

        WindowEvent::Key(Key::Space, _, Action::Release, _) => {
          slice_method = slice_method.toggle();
          println!("now rendering slice {:?}", slice_method);
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

    surface
      .pipeline_builder()
      .pipeline(&back_buffer, [0., 0., 0., 0.], |_, shd_gate| {
        shd_gate.shade(&program, |_, rdr_gate| {
          rdr_gate.render(RenderState::default(), |tess_gate| {
            let slice = match slice_method {
              // the red triangle is at slice [..3]; you can also use the TessSlice::one_sub
              // combinator if the start element is 0; it’s also possible to use [..=2] for
              // inclusive ranges
              SliceMethod::Red => triangles.slice(..3), // TessSlice::one_slice(&triangles, 0, 3),
              // the blue triangle is at slice [3..]
              SliceMethod::Blue => triangles.slice(3..), // TessSlice::one_slice(&triangles, 3, 6),
              // both triangles are at slice [0..6] or [..], but we’ll use the faster
              // TessSlice::one_whole combinator; this combinator is also if you invoke the From or
              // Into method on (&triangles) (we did that in 02-render-state)
              SliceMethod::Both => triangles.slice(..), // TessSlice::one_whole(&triangles)
            };

            // render the dynamically selected slice
            tess_gate.render(&mut surface, slice);
          });
        });
      });

    surface.swap_buffers();
  }
}
