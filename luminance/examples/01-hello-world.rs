//! This program shows how to render two simple triangles and is the hello world of luminance.
//!
//! The direct / indexed methods just show you how you’re supposed to use them (don’t try and find
//! any differences in the rendered images, because there’s none!).
//!
//! Press <space> to switch between direct tessellation and indexed tessellation.
//! Press <escape> to quit or close the window.
//!
//! https://docs.rs/luminance

use luminance::context::GraphicsContext as _;
use luminance::framebuffer::Framebuffer;
use luminance::render_state::RenderState;
use luminance::shader::program::Program;
use luminance::tess::{Mode, TessBuilder};
use luminance_derive::{Semantics, Vertex};
use luminance_glfw::{Action, GlfwSurface, Key, Surface, WindowEvent, WindowDim, WindowOpt};

// We get the shader at compile time from local files
const VS: &'static str = include_str!("simple-vs.glsl");
const FS: &'static str = include_str!("simple-fs.glsl");

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
  Color
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
  rgb: VertexColor
}

// The vertices. We define two triangles.
const TRI_VERTICES: [Vertex; 6] = [
  // First triangle – an RGB one.
  Vertex { pos: VertexPosition::new([0.5, -0.5]), rgb: VertexColor::new([0, 255, 0]) },
  Vertex { pos: VertexPosition::new([0.0, 0.5]), rgb: VertexColor::new([0, 0, 255]) },
  Vertex { pos: VertexPosition::new([-0.5, -0.5]), rgb: VertexColor::new([255, 0, 0]) },
  // Second triangle, a purple one, positioned differently.
  Vertex { pos: VertexPosition::new([-0.5, 0.5]), rgb: VertexColor::new([255, 51, 255]) },
  Vertex { pos: VertexPosition::new([0.0, -0.5]), rgb: VertexColor::new([51, 255, 255]) },
  Vertex { pos: VertexPosition::new([0.5, 0.5]), rgb: VertexColor::new([51, 51, 255]) },
];

// A small struct wrapper used to deinterleave positions.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Vertex)]
#[vertex(sem = "Semantics")]
struct Positions {
  pos: VertexPosition
}

// A small struct wrapper used to deinterleave colors.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Vertex)]
#[vertex(sem = "Semantics")]
struct Colors {
  #[vertex(normalized = "true")]
  color: VertexColor
}

// The vertices, deinterleaved versions. We still define two triangles.
const TRI_DEINT_POS_VERTICES: &[Positions] = &[
  Positions { pos: VertexPosition::new([0.5, -0.5]) },
  Positions { pos: VertexPosition::new([0.0, 0.5]) },
  Positions { pos: VertexPosition::new([-0.5, -0.5]) },
  Positions { pos: VertexPosition::new([-0.5, 0.5]) },
  Positions { pos: VertexPosition::new([0.0, -0.5]) },
  Positions { pos: VertexPosition::new([0.5, 0.5]) },
];

const TRI_DEINT_COLOR_VERTICES: &[Colors] = &[
  Colors { color: VertexColor::new([0, 255, 0]) },
  Colors { color: VertexColor::new([0, 0, 255]) },
  Colors { color: VertexColor::new([255, 0, 0]) },
  Colors { color: VertexColor::new([255, 51, 255]) },
  Colors { color: VertexColor::new([51, 255, 255]) },
  Colors { color: VertexColor::new([51, 51, 255]) },
];

// Indices into TRI_VERTICES to use to build up the triangles.
const TRI_INDICES: [u32; 6] = [
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
  let mut surface = GlfwSurface::new(
    WindowDim::Windowed(960, 540),
    "Hello, world!",
    WindowOpt::default(),
  ).expect("GLFW surface creation");

  // We need a program to “shade” our triangles and to tell luminance which is the input vertex
  // type, and we’re not interested in the other two type variables for this sample.
  let (program, _) = Program::<Semantics, (), ()>::from_strings(None, VS, None, FS).expect("program creation");

  // Create tessellation for direct geometry; that is, tessellation that will render vertices by
  // taking one after another in the provided slice.
  let direct_triangles = TessBuilder::new(&mut surface)
    .add_vertices(TRI_VERTICES)
    .set_mode(Mode::Triangle)
    .build()
    .unwrap();

  // Create indexed tessellation; that is, the vertices will be picked by using the indexes provided
  // by the second slice and this indexes will reference the first slice (useful not to duplicate
  // vertices on more complex objects than just two triangles).
  let indexed_triangles = TessBuilder::new(&mut surface)
    .add_vertices(TRI_VERTICES)
    .set_indices(TRI_INDICES)
    .set_mode(Mode::Triangle)
    .build()
    .unwrap();

  // Create direct, deinterleaved tesselations; such tessellations allow to separate vertex
  // attributes in several contiguous regions of memory.
  let direct_deinterleaved_triangles = TessBuilder::new(&mut surface)
    .add_vertices(TRI_DEINT_POS_VERTICES)
    .add_vertices(TRI_DEINT_COLOR_VERTICES)
    .set_mode(Mode::Triangle)
    .build()
    .unwrap();

  // Create indexed, deinterleaved tessellations; have your cake and fucking eat it, now.
  let indexed_deinterleaved_triangles = TessBuilder::new(&mut surface)
    .add_vertices(TRI_DEINT_POS_VERTICES)
    .add_vertices(TRI_DEINT_COLOR_VERTICES)
    .set_indices(TRI_INDICES)
    .set_mode(Mode::Triangle)
    .build()
    .unwrap();

  // The back buffer, which we will make our render into (we make it mutable so that we can change
  // it whenever the window dimensions change).
  let size = surface.size();
  let mut back_buffer = Framebuffer::back_buffer(&mut surface, size);
  let mut demo = TessMethod::Direct;
  let mut resize = false;

  println!("now rendering {:?}", demo);

  'app: loop {
    // For all the events on the surface.
    for event in surface.poll_events() {
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
      let size = surface.size();
      back_buffer = Framebuffer::back_buffer(&mut surface, size);
      resize = false;
    }

    // Create a new dynamic pipeline that will render to the back buffer and must clear it with
    // pitch black prior to do any render to it.
    surface
      .pipeline_builder()
      .pipeline(&back_buffer, [0., 0., 0., 0.], |_, shd_gate| {
        // Start shading with our program.
        shd_gate.shade(&program, |_, rdr_gate| {
          // Start rendering things with the default render state provided by luminance.
          rdr_gate.render(RenderState::default(), |tess_gate| {
            // Pick the right tessellation to use depending on the mode chosen.
            let tess = match demo {
              TessMethod::Direct => &direct_triangles,
              TessMethod::Indexed => &indexed_triangles,
              TessMethod::DirectDeinterleaved => &direct_deinterleaved_triangles,
              TessMethod::IndexedDeinterleaved => &indexed_deinterleaved_triangles,
            };

            // Render the tessellation to the surface.
            tess_gate.render(&mut surface, tess);
          });
        });
      });

    // Finally, swap the backbuffer with the frontbuffer in order to render our triangles onto your
    // screen.
    surface.swap_buffers();
  }
}
