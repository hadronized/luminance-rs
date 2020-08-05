//! This program shows how to render two simple triangles and is the hello world of luminance.
//!
//! The direct / indexed methods just show you how you’re supposed to use them (don’t try and find
//! any differences in the rendered images, because there’s none!).
//!
//! Press <space> to switch between direct tessellation and indexed tessellation.
//! Press <escape> to quit or close the window.
//!
//! https://docs.rs/luminance

use glfw::{Action, Context as _, Key, WindowEvent};
use luminance::context::GraphicsContext as _;
use luminance::pipeline::PipelineState;
use luminance::render_state::RenderState;
use luminance::tess::Mode;
use luminance_derive::{Semantics, Vertex};
use luminance_glfw::GlfwSurface;
use luminance_windowing::{WindowDim, WindowOpt};

// We get the shader at compile time from local files
const VS: &str = include_str!("simple-vs.glsl");
const FS: &str = include_str!("simple-fs.glsl");

// Vertex semantics. Those are needed to instruct the GPU how to select vertex’s attributes from
// the memory we fill at render time, in shaders. You don’t have to worry about them; just keep in
// mind they’re mandatory and act as “protocol” between GPU’s memory regions and shaders.
//
// We derive Semantics automatically and provide the mapping as field attributes.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Semantics)]
pub enum Semantics {
  // - Reference vertex positions with the "co" variable in vertex shaders.
  // - The underlying representation is [f32; 2], which is a vec2 in GLSL.
  // - The wrapper type you can use to handle such a semantics is VertexPosition.
  #[sem(name = "co", repr = "[f32; 2]", wrapper = "VertexPosition")]
  Position,
  // - Reference vertex colors with the "color" variable in vertex shaders.
  // - The underlying representation is [u8; 3], which is a uvec3 in GLSL.
  // - The wrapper type you can use to handle such a semantics is VertexColor.
  #[sem(name = "color", repr = "[u8; 3]", wrapper = "VertexColor")]
  Color,
}

// Our vertex type.
//
// We derive the Vertex trait automatically and we associate to each field the semantics that must
// be used on the GPU. The proc-macro derive Vertex will make sur for us every field we use have a
// mapping to the type you specified as semantics.
//
// Currently, we need to use #[repr(C))] to ensure Rust is not going to move struct’s fields around.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Vertex)]
#[vertex(sem = "Semantics")]
struct Vertex {
  pos: VertexPosition,
  // Here, we can use the special normalized = <bool> construct to state whether we want integral
  // vertex attributes to be available as normalized floats in the shaders, when fetching them from
  // the vertex buffers. If you set it to "false" or ignore it, you will get non-normalized integer
  // values (i.e. value ranging from 0 to 255 for u8, for instance).
  #[vertex(normalized = "true")]
  rgb: VertexColor,
}

// The vertices. We define two triangles.
const TRI_VERTICES: [Vertex; 6] = [
  // First triangle – an RGB one.
  Vertex::new(
    VertexPosition::new([0.5, -0.5]),
    VertexColor::new([0, 255, 0]),
  ),
  Vertex::new(
    VertexPosition::new([0.0, 0.5]),
    VertexColor::new([0, 0, 255]),
  ),
  Vertex::new(
    VertexPosition::new([-0.5, -0.5]),
    VertexColor::new([255, 0, 0]),
  ),
  // Second triangle, a purple one, positioned differently.
  Vertex::new(
    VertexPosition::new([-0.5, 0.5]),
    VertexColor::new([255, 51, 255]),
  ),
  Vertex::new(
    VertexPosition::new([0.0, -0.5]),
    VertexColor::new([51, 255, 255]),
  ),
  Vertex::new(
    VertexPosition::new([0.5, 0.5]),
    VertexColor::new([51, 51, 255]),
  ),
];

// The vertices, deinterleaved versions. We still define two triangles.
const TRI_DEINT_POS_VERTICES: &[VertexPosition] = &[
  VertexPosition::new([0.5, -0.5]),
  VertexPosition::new([0.0, 0.5]),
  VertexPosition::new([-0.5, -0.5]),
  VertexPosition::new([-0.5, 0.5]),
  VertexPosition::new([0.0, -0.5]),
  VertexPosition::new([0.5, 0.5]),
];

const TRI_DEINT_COLOR_VERTICES: &[VertexColor] = &[
  VertexColor::new([0, 255, 0]),
  VertexColor::new([0, 0, 255]),
  VertexColor::new([255, 0, 0]),
  VertexColor::new([255, 51, 255]),
  VertexColor::new([51, 255, 255]),
  VertexColor::new([51, 51, 255]),
];

// Indices into TRI_VERTICES to use to build up the triangles.
const TRI_INDICES: [u8; 6] = [
  0, 1, 2, // First triangle.
  3, 4, 5, // Second triangle.
];

// Convenience type to demonstrate the difference between direct geometry and indirect (indexed)
// one.
#[derive(Copy, Clone, Debug)]
enum TessMethod {
  Direct,
  Indexed,
  DirectDeinterleaved,
  IndexedDeinterleaved,
}

impl TessMethod {
  fn toggle(self) -> Self {
    match self {
      TessMethod::Direct => TessMethod::Indexed,
      TessMethod::Indexed => TessMethod::DirectDeinterleaved,
      TessMethod::DirectDeinterleaved => TessMethod::IndexedDeinterleaved,
      TessMethod::IndexedDeinterleaved => TessMethod::Direct,
    }
  }
}

fn main() {
  // First thing first: we create a new surface to render to and get events from.
  let dim = WindowDim::Windowed {
    width: 960,
    height: 540,
  };
  let mut surface = GlfwSurface::new_gl33(
    "Hello, world; from OpenGL 3.3!",
    WindowOpt::default().set_dim(dim),
  )
  .expect("GLFW surface creation");

  // We need a program to “shade” our triangles and to tell luminance which is the input vertex
  // type, and we’re not interested in the other two type variables for this sample.

  let mut program = surface
    .new_shader_program::<Semantics, (), ()>()
    .from_strings(VS, None, None, FS)
    .expect("program creation")
    .ignore_warnings();

  // Create tessellation for direct geometry; that is, tessellation that will render vertices by
  // taking one after another in the provided slice.
  let direct_triangles = surface
    .new_tess()
    .set_vertices(&TRI_VERTICES[..])
    .set_mode(Mode::Triangle)
    .build()
    .unwrap();

  // Create indexed tessellation; that is, the vertices will be picked by using the indexes provided
  // by the second slice and this indexes will reference the first slice (useful not to duplicate
  // vertices on more complex objects than just two triangles).
  let indexed_triangles = surface
    .new_tess()
    .set_vertices(&TRI_VERTICES[..])
    .set_indices(&TRI_INDICES[..])
    .set_mode(Mode::Triangle)
    .build()
    .unwrap();

  // Create direct, deinterleaved tesselations; such tessellations allow to separate vertex
  // attributes in several contiguous regions of memory.
  let direct_deinterleaved_triangles = surface
    .new_deinterleaved_tess::<Vertex, ()>()
    .set_attributes(&TRI_DEINT_POS_VERTICES[..])
    .set_attributes(&TRI_DEINT_COLOR_VERTICES[..])
    .set_mode(Mode::Triangle)
    .build()
    .unwrap();

  // Create indexed, deinterleaved tessellations; have your cake and fucking eat it, now.
  let indexed_deinterleaved_triangles = surface
    .new_deinterleaved_tess::<Vertex, ()>()
    .set_attributes(&TRI_DEINT_POS_VERTICES[..])
    .set_attributes(&TRI_DEINT_COLOR_VERTICES[..])
    .set_indices(&TRI_INDICES[..])
    .set_mode(Mode::Triangle)
    .build()
    .unwrap();

  //// The back buffer, which we will make our render into (we make it mutable so that we can change
  //// it whenever the window dimensions change).
  let mut back_buffer = surface.back_buffer().unwrap();
  let mut demo = TessMethod::Direct;
  let mut resize = false;

  println!("now rendering {:?}", demo);

  'app: loop {
    // For all the events on the surface.
    surface.window.glfw.poll_events();
    for (_, event) in surface.events_rx.try_iter() {
      match event {
        // If we close the window or press escape, quit the main loop (i.e. quit the application).
        WindowEvent::Close | WindowEvent::Key(Key::Escape, _, Action::Release, _) => break 'app,

        // If we hit the spacebar, change the kind of tessellation.
        WindowEvent::Key(Key::Space, _, Action::Release, _) => {
          demo = demo.toggle();
          println!("now rendering {:?}", demo);
        }

        // Handle window resizing.
        WindowEvent::FramebufferSize(..) => {
          resize = true;
        }

        _ => (),
      }
    }

    if resize {
      // Simply ask another backbuffer at the right dimension (no allocation / reallocation).
      back_buffer = surface.back_buffer().unwrap();
      resize = false;
    }

    // Create a new dynamic pipeline that will render to the back buffer and must clear it with
    // pitch black prior to do any render to it.
    let render = surface
      .new_pipeline_gate()
      .pipeline(
        &back_buffer,
        &PipelineState::default(),
        |_, mut shd_gate| {
          // Start shading with our program.
          shd_gate.shade(&mut program, |_, _, mut rdr_gate| {
            // Start rendering things with the default render state provided by luminance.
            rdr_gate.render(&RenderState::default(), |mut tess_gate| {
              // Pick the right tessellation to use depending on the mode chosen and render it to the
              // surface.
              match demo {
                TessMethod::Direct => tess_gate.render(&direct_triangles),
                TessMethod::Indexed => tess_gate.render(&indexed_triangles),
                TessMethod::DirectDeinterleaved => {
                  tess_gate.render(&direct_deinterleaved_triangles)
                }
                TessMethod::IndexedDeinterleaved => {
                  tess_gate.render(&indexed_deinterleaved_triangles)
                }
              }
            })
          })
        },
      )
      .assume();

    // Finally, swap the backbuffer with the frontbuffer in order to render our triangles onto your
    // screen.
    if render.is_ok() {
      surface.window.swap_buffers();
    } else {
      break 'app;
    }
  }
}
