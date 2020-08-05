//! This program shows how to render two simple triangles, query the texels from the rendered
//! framebuffer and output them in a texture.
//!
//! Press <escape> to quit or close the window.
//!
//! https://docs.rs/luminance

mod common;

use crate::common::{Semantics, Vertex, VertexColor, VertexPosition};
use glfw::{Action, Context as _, Key, WindowEvent};
use image::{save_buffer, ColorType};
use luminance::context::GraphicsContext as _;
use luminance::pipeline::PipelineState;
use luminance::pixel::NormRGBA8UI;
use luminance::render_state::RenderState;
use luminance::tess::Mode;
use luminance::texture::{Dim2, Sampler};
use luminance_derive::Vertex;
use luminance_glfw::GlfwSurface;
use luminance_windowing::{WindowDim, WindowOpt};

// We get the shader at compile time from local files
const VS: &str = include_str!("simple-vs.glsl");
const FS: &str = include_str!("simple-fs.glsl");

// The vertices. We define two triangles.
const TRI_VERTICES: [Vertex; 6] = [
  // first triangle – an RGB one
  Vertex {
    pos: VertexPosition::new([0.5, -0.5]),
    rgb: VertexColor::new([0., 1., 0.]),
  },
  Vertex {
    pos: VertexPosition::new([0.0, 0.5]),
    rgb: VertexColor::new([0., 0., 1.]),
  },
  Vertex {
    pos: VertexPosition::new([-0.5, -0.5]),
    rgb: VertexColor::new([1., 0., 0.]),
  },
  // second triangle, a purple one, positioned differently
  Vertex {
    pos: VertexPosition::new([-0.5, 0.5]),
    rgb: VertexColor::new([1., 0.2, 1.]),
  },
  Vertex {
    pos: VertexPosition::new([0.0, -0.5]),
    rgb: VertexColor::new([0.2, 1., 1.]),
  },
  Vertex {
    pos: VertexPosition::new([0.5, 0.5]),
    rgb: VertexColor::new([0.2, 0.2, 1.]),
  },
];

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Vertex)]
#[vertex(sem = "Semantics")]
struct Positions {
  pos: VertexPosition,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Vertex)]
#[vertex(sem = "Semantics")]
struct Colors {
  color: VertexColor,
}

fn main() {
  // first thing first: we create a new surface to render to and get events from
  let dim = WindowDim::Windowed {
    width: 960,
    height: 540,
  };
  let mut surface = GlfwSurface::new_gl33("Hello, world!", WindowOpt::default().set_dim(dim))
    .expect("GLFW surface creation");

  // we need a program to “shade” our triangles and to tell luminance which is the input vertex
  // type, and we’re not interested in the other two type variables for this sample
  let mut program = surface
    .new_shader_program::<Semantics, (), ()>()
    .from_strings(VS, None, None, FS)
    .expect("program creation")
    .ignore_warnings();

  // create tessellation for direct geometry; that is, tessellation that will render vertices by
  // taking one after another in the provided slice
  let tris = surface
    .new_tess()
    .set_vertices(&TRI_VERTICES[..])
    .set_mode(Mode::Triangle)
    .build()
    .unwrap();

  // whether the image has been generated on disk
  let mut generated = false;

  // the back buffer, which we will make our render into (we make it mutable so that we can change
  // it whenever the window dimensions change)
  let mut fb = surface
    .new_framebuffer::<Dim2, NormRGBA8UI, ()>([960, 540], 0, Sampler::default())
    .unwrap();

  'app: loop {
    // for all the events on the surface
    surface.window.glfw.poll_events();
    for (_, event) in surface.events_rx.try_iter() {
      match event {
        // if we close the window or press escape, quit the main loop (i.e. quit the application)
        WindowEvent::Close | WindowEvent::Key(Key::Escape, _, Action::Release, _) => break 'app,

        _ => (),
      }
    }

    // create a new dynamic pipeline that will render to the back buffer and must clear it with
    // pitch black prior to do any render to it
    let render = surface
      .new_pipeline_gate()
      .pipeline(&fb, &PipelineState::default(), |_, mut shd_gate| {
        // start shading with our program
        shd_gate.shade(&mut program, |_, _, mut rdr_gate| {
          // start rendering things with the default render state provided by luminance
          rdr_gate.render(&RenderState::default(), |mut tess_gate| {
            // pick the right tessellation to use depending on the mode chosen
            // render the tessellation to the surface
            tess_gate.render(&tris)
          })
        })
      })
      .assume();

    if !generated {
      // the backbuffer contains our texels
      let texels = fb.color_slot().get_raw_texels().unwrap();
      // create a .png file and output it
      save_buffer("./rendered.png", &texels, 960, 540, ColorType::Rgba8).unwrap();

      generated = true;
    }

    // finally, swap the backbuffer with the frontbuffer in order to render our triangles onto your
    // screen
    if render.is_ok() {
      surface.window.swap_buffers();
    } else {
      break 'app;
    }
  }
}
