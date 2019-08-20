//! This program shows how to render two simple triangles, query the texels from the rendered
//! framebuffer and output them in a texture.
//!
//! Press <escape> to quit or close the window.
//!
//! https://docs.rs/luminance

mod common;

use common::{Semantics, Vertex, VertexColor, VertexPosition};
use image::{ColorType, save_buffer};
use luminance::context::GraphicsContext as _;
use luminance::framebuffer::Framebuffer;
use luminance::render_state::RenderState;
use luminance::shader::program::Program;
use luminance::tess::{Mode, TessBuilder};
use luminance::texture::{Flat, Dim2};
use luminance::pixel::RGB8UI;
use luminance_derive::Vertex;
use luminance_glfw::{Action, GlfwSurface, Key, Surface, WindowEvent, WindowDim, WindowOpt};

// We get the shader at compile time from local files
const VS: &'static str = include_str!("simple-vs.glsl");
const FS: &'static str = include_str!("simple-fs-unsigned.glsl");

// The vertices. We define two triangles.
const TRI_VERTICES: [Vertex; 6] = [
  // first triangle – an RGB one
  Vertex { pos: VertexPosition::new([0.5, -0.5]), rgb: VertexColor::new([0., 1., 0.]) },
  Vertex { pos: VertexPosition::new([0.0, 0.5]), rgb: VertexColor::new([0., 0., 1.]) },
  Vertex { pos: VertexPosition::new([-0.5, -0.5]), rgb: VertexColor::new([1., 0., 0.]) },
  // second triangle, a purple one, positioned differently
  Vertex { pos: VertexPosition::new([-0.5, 0.5]), rgb: VertexColor::new([1., 0.2, 1.]) },
  Vertex { pos: VertexPosition::new([0.0, -0.5]), rgb: VertexColor::new([0.2, 1., 1.]) },
  Vertex { pos: VertexPosition::new([0.5, 0.5]), rgb: VertexColor::new([0.2, 0.2, 1.]) },
];

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Vertex)]
#[vertex(sem = "Semantics")]
struct Positions {
  pos: VertexPosition
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Vertex)]
#[vertex(sem = "Semantics")]
struct Colors {
  color: VertexColor
}

fn main() {
  // first thing first: we create a new surface to render to and get events from
  let mut surface = GlfwSurface::new(
    WindowDim::Windowed(960, 540),
    "Hello, world!",
    WindowOpt::default(),
  ).expect("GLFW surface creation");

  // we need a program to “shade” our triangles and to tell luminance which is the input vertex
  // type, and we’re not interested in the other two type variables for this sample
  let (program, _) = Program::<Semantics, (), ()>::from_strings(None, VS, None, FS).expect("program creation");

  // create tessellation for direct geometry; that is, tessellation that will render vertices by
  // taking one after another in the provided slice
  let tris = TessBuilder::new(&mut surface)
    .add_vertices(TRI_VERTICES)
    .set_mode(Mode::Triangle)
    .build()
    .unwrap();

  // whether the image has been generated on disk
  let mut generated = false;

  // the back buffer, which we will make our render into (we make it mutable so that we can change
  // it whenever the window dimensions change)
  let fb = Framebuffer::<Flat, Dim2, RGB8UI, ()>::new(&mut surface, [960, 540], 0).unwrap();

  'app: loop {
    // for all the events on the surface
    for event in surface.poll_events() {
      match event {
        // if we close the window or press escape, quit the main loop (i.e. quit the application)
        WindowEvent::Close | WindowEvent::Key(Key::Escape, _, Action::Release, _) => break 'app,

        _ => (),
      }
    }

    // create a new dynamic pipeline that will render to the back buffer and must clear it with
    // pitch black prior to do any render to it
    surface
      .pipeline_builder()
      .pipeline(&fb, [0., 0., 0., 0.], |_, shd_gate| {
        // start shading with our program
        shd_gate.shade(&program, |rdr_gate, _| {
          // start rendering things with the default render state provided by luminance
          rdr_gate.render(RenderState::default(), |tess_gate| {
            let tess = &tris;

            // pick the right tessellation to use depending on the mode chosen
            // render the tessellation to the surface
            tess_gate.render(&mut surface, tess.into());
          });
        });
      });

    if !generated {
      // the backbuffer contains our texels
      let texels = fb.color_slot().get_raw_texels();
      // create a .png file and output it
      save_buffer("./rendered.png", &texels, 960, 540, ColorType::RGB(8)).unwrap();

      generated = true;
    }

    // finally, swap the backbuffer with the frontbuffer in order to render our triangles onto your
    // screen
    surface.swap_buffers();
  }
}
