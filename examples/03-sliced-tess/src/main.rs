//! This program shows how to render two triangles that live in the same GPU tessellation. This is
//! called “sliced tessellation” in luminance and can help you implement plenty of situations. One
//! of the most interesting is for particles effect: you can allocate a big tessellation object on
//! the GPU and slice it to render only the living particles.
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

extern crate luminance;
extern crate luminance_glfw;

use luminance::framebuffer::Framebuffer;
use luminance::shader::program::Program;
use luminance::tess::{Mode, Tess, TessRender};
use luminance::render_state::RenderState;
use luminance_glfw::event::{Action, Key, WindowEvent};
use luminance_glfw::surface::{GlfwSurface, Surface, WindowDim, WindowOpt};
use luminance::context::GraphicsContext;

const VS: &'static str = include_str!("vs.glsl");
const FS: &'static str = include_str!("fs.glsl");

type Vertex = ([f32; 2], [f32; 3]);

const TRI_VERTICES: [Vertex; 6] = [
  // first triangle – a red one
  ([ 0.5, -0.5], [1., 0., 0.]),
  ([ 0.0,  0.5], [1., 0., 0.]),
  ([-0.5, -0.5], [1., 0., 0.]),
  // second triangle, a blue one
  ([-0.5,  0.5], [0., 0., 1.]),
  ([ 0.0, -0.5], [0., 0., 1.]),
  ([ 0.5,  0.5], [0., 0., 1.])
];

// Convenience type to demonstrate how the depth test influences the rendering of two triangles.
#[derive(Copy, Clone, Debug)]
enum SliceMethod { 
  Red, // draw the red triangle
  Blue, // draw the blue triangle
  Both // draw both the triangles
}

impl SliceMethod {
  fn toggle(self) -> Self {
    match self {
      SliceMethod::Red => SliceMethod::Blue,
      SliceMethod::Blue => SliceMethod::Both,
      SliceMethod::Both => SliceMethod::Red
    }
  }
}

fn main() {
  let mut surface = GlfwSurface::new(WindowDim::Windowed(960, 540), "Hello, world!", WindowOpt::default()).expect("GLFW surface creation");

  let (program, _) = Program::<Vertex, (), ()>::from_strings(None, VS, None, FS).expect("program creation");

  // create a single GPU tessellation that holds both the triangles (like in 01-hello-world)
  let triangles = Tess::new(&mut surface, Mode::Triangle, &TRI_VERTICES[..], None);

  let mut back_buffer = Framebuffer::default(surface.size());

  let mut slice_method = SliceMethod::Red;
  println!("now rendering slice {:?}", slice_method);

  'app: loop {
    for event in surface.poll_events() {
      match event {
        WindowEvent::Close | WindowEvent::Key(Key::Escape, _, Action::Release, _) => {
          break 'app
        }

        WindowEvent::Key(Key::Space, _, Action::Release, _) => {
          slice_method = slice_method.toggle();
          println!("now rendering slice {:?}", slice_method);
        }

        WindowEvent::FramebufferSize(width, height) => {
          back_buffer = Framebuffer::default([width as u32, height as u32]);
        }

        _ => ()
      }
    }

    surface.pipeline_builder().pipeline(&back_buffer, [0., 0., 0., 0.], |_, shd_gate| {
      shd_gate.shade(&program, |rdr_gate, _| {
        rdr_gate.render(RenderState::default(), |tess_gate| {
          let slice = match slice_method {
            // the red triangle is at slice [0..3]; you can also use the TessRender::one_sub
            // combinator if the start element is 0
            SliceMethod::Red => TessRender::one_slice(&triangles, 0, 3),
            // the blue triangle is at slice [3..6]
            SliceMethod::Blue => TessRender::one_slice(&triangles, 3, 6),
            // both triangles are at slice [0..6] or [..], but we’ll use the faster
            // TessRender::one_whole combinator; this combinator is also if you invoke the From or
            // Into method on (&triangles) (we did that in 02-render-state)
            SliceMethod::Both => TessRender::one_whole(&triangles)
          };


          // render the dynamically selected slice
          tess_gate.render(&mut surface, slice);
        });
      });
    });

    surface.swap_buffers();
  }
}
