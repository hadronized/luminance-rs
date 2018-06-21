//! This program shows how to render two simple triangles and is the hello world of luminance.
//!
//! The direct / indexed methods just show you how you’re supposed to use them (don’t try and find
//! any differences in the rendered images, because there’s none!).
//!
//! Press <space> to switch between direct tessellation and indexed tessellation.
//! Press <escape> to quit or close the window.
//!
//! https://docs.rs/luminance

extern crate luminance;
extern crate luminance_glfw;

use luminance::framebuffer::Framebuffer;
use luminance::shader::program::Program;
use luminance::tess::{Mode, Tess};
use luminance::render_state::RenderState;
use luminance_glfw::event::{Action, Key, WindowEvent};
use luminance_glfw::surface::{GlfwSurface, Surface, WindowDim, WindowOpt};
use luminance::context::GraphicsContext;

// we get the shader at compile time from ./vs.glsl and ./fs.glsl
const VS: &'static str = include_str!("vs.glsl");
const FS: &'static str = include_str!("fs.glsl");

// type of the vertices of our triangles; here we use a 2D position ([f32; 2]) and a RGB color
// ([f32; 3]).
type Vertex = ([f32; 2], [f32; 3]);

// The vertices. We define two triangles.
const TRI_VERTICES: [Vertex; 6] = [
  // first triangle – an RGB one
  ([ 0.5, -0.5], [0., 1., 0.]),
  ([ 0.0,  0.5], [0., 0., 1.]),
  ([-0.5, -0.5], [1., 0., 0.]),
  // second triangle, a purple one, positioned differently
  ([-0.5,  0.5], [1., 0.2, 1.]),
  ([ 0.0, -0.5], [0.2, 1., 1.]),
  ([ 0.5,  0.5], [0.2, 0.2, 1.])
];

// Indices into TRI_VERTICES to use to build up the triangles.
const TRI_INDEXES: [u32; 6] = [
  0, 1, 2, // first triangle
  3, 4, 5 // second triangle
];

// Convenience type to demonstrate the difference between direct geometry and indirect (indexed)
// one.
#[derive(Copy, Clone, Debug)]
enum TessMethod {
  Direct,
  Indexed
}

impl TessMethod {
  fn toggle(self) -> Self {
    match self {
      TessMethod::Direct => TessMethod::Indexed,
      TessMethod::Indexed => TessMethod::Direct,
    }
  }
}

fn main() {
  // first thing first: we create a new surface to render to and get events from
  let mut surface = GlfwSurface::new(WindowDim::Windowed(960, 540), "Hello, world!", WindowOpt::default()).expect("GLFW surface creation");

  // we need a program to “shade” our triangles and to tell luminance which is the input vertex
  // type, and we’re not interested in the other two type variables for this sample
  let (program, _) = Program::<Vertex, (), ()>::from_strings(None, VS, None, FS).expect("program creation");

  // create tessellation for direct geometry; that is, tessellation that will render vertices by
  // taking one after another in the provided slice
  let direct_triangles = Tess::new(&mut surface, Mode::Triangle, &TRI_VERTICES[..], None);

  // create indexed tessellation; that is, the vertices will be picked by using the indexes provided
  // by the second slice and this indexes will reference the first slice (useful not to duplicate
  // vertices on more complex objects than just two triangles)
  let indexed_triangles = Tess::new(&mut surface, Mode::Triangle, &TRI_VERTICES[..], &TRI_INDEXES[..]);

  // the back buffer, which we will make our render into (we make it mutable so that we can change
  // it whenever the window dimensions change)
  let mut back_buffer = Framebuffer::default(surface.size());

  let mut demo = TessMethod::Direct;
  println!("now rendering {:?}", demo);

  'app: loop {
    // for all the events on the surface
    for event in surface.poll_events() {
      match event {
        // if we close the window or press escape, quit the main loop (i.e. quit the application)
        WindowEvent::Close | WindowEvent::Key(Key::Escape, _, Action::Release, _) => {
          break 'app
        }

        // if we hit the spacebar, change the kind of tessellation
        WindowEvent::Key(Key::Space, _, Action::Release, _) => {
          demo = demo.toggle();
          println!("now rendering {:?}", demo);
        }

        // handle window resizing
        WindowEvent::FramebufferSize(width, height) => {
          // simply ask another backbuffer at the right dimension (no allocation / reallocation)
          back_buffer = Framebuffer::default([width as u32, height as u32]);
        }

        _ => ()
      }
    }

    // create a new dynamic pipeline that will render to the back buffer and must clear it with
    // pitch black prior to do any render to it
    surface.pipeline_builder().pipeline(&back_buffer, [0., 0., 0., 0.], |_, shd_gate| {
      // start shading with our program
      shd_gate.shade(&program, |rdr_gate, _| {
        // start rendering things with the default render state provided by luminance
        rdr_gate.render(RenderState::default(), |tess_gate| {
          // pick the right tessellation to use depending on the mode chosen
          let tess = match demo {
            TessMethod::Direct => &direct_triangles,
            TessMethod::Indexed => &indexed_triangles
          };

          // render the tessellation to the surface
          tess_gate.render(&mut surface, tess.into());
        });
      });
    });

    // finally, swap the backbuffer with the frontbuffer in order to render our triangles onto your
    // screen
    surface.swap_buffers();
  }
}
