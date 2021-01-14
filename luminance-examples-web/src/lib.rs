//! This program shows how to render two simple triangles and is the hello world of luminance.
//!
//! The direct / indexed methods just show you how you’re supposed to use them (don’t try and find
//! any differences in the rendered images, because there’s none!).
//!
//! Press <space> to switch between direct tessellation and indexed tessellation.
//! Press <escape> to quit or close the window.
//!
//! https://docs.rs/luminance

use luminance::{Semantics, Vertex};
use luminance_front::context::GraphicsContext as _;
use luminance_front::pipeline::PipelineState;
use luminance_front::render_state::RenderState;
use luminance_front::shader::Program;
use luminance_front::tess::{Deinterleaved, Interleaved, Mode, Tess};
use luminance_web_sys::WebSysWebGL2Surface;
use wasm_bindgen::prelude::*;

// We get the shader at compile time from local files
const VS: &'static str = include_str!("../shaders/simple-vs.glsl");
const FS: &'static str = include_str!("../shaders/simple-fs.glsl");

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
// be used on the GPU. The proc-macro derive Vertex will make sure for us every field we use has a
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

/// A convenient type to return as opaque to JS.
#[wasm_bindgen]
pub struct Scene {
  surface: WebSysWebGL2Surface,
  program: Program<Semantics, (), ()>,
  direct_triangles: Tess<Vertex, (), (), Interleaved>,
  indexed_triangles: Tess<Vertex, u8, (), Interleaved>,
  direct_deinterleaved_triangles: Tess<Vertex, (), (), Deinterleaved>,
  indexed_deinterleaved_triangles: Tess<Vertex, u8, (), Deinterleaved>,
  tess_method: TessMethod,
}

/// Get the whole scene and expose it on the JS side.
#[wasm_bindgen]
pub fn get_scene(canvas_name: &str) -> Scene {
  // First thing first: we create a new surface to render to and get events from.
  let mut surface = WebSysWebGL2Surface::new(canvas_name).expect("web-sys surface");

  // We need a program to “shade” our triangles and to tell luminance which is the input vertex
  // type, and we’re not interested in the other two type variables for this sample.
  let program = surface
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

  let tess_method = TessMethod::Direct;

  Scene {
    surface,
    program,
    direct_triangles,
    indexed_triangles,
    direct_deinterleaved_triangles,
    indexed_deinterleaved_triangles,
    tess_method,
  }
}

#[wasm_bindgen]
pub fn toggle_tess_method(scene: &mut Scene) {
  let prev_meth = scene.tess_method;

  scene.tess_method = scene.tess_method.toggle();

  web_sys::console::log_1(
    &format!(
      "toggling tess method from {:?} to {:?}",
      prev_meth, scene.tess_method
    )
    .into(),
  );
}

#[wasm_bindgen]
pub fn render_scene(scene: &mut Scene) {
  let Scene {
    surface,
    tess_method,
    ref mut program,
    ref direct_triangles,
    ref indexed_triangles,
    ref direct_deinterleaved_triangles,
    ref indexed_deinterleaved_triangles,
  } = scene;
  let back_buffer = surface.back_buffer().unwrap();

  // Create a new dynamic pipeline that will render to the back buffer and must clear it with
  // pitch black prior to do any render to it.
  scene
    .surface
    .new_pipeline_gate()
    .pipeline(
      &back_buffer,
      &PipelineState::default(),
      |_, mut shd_gate| {
        // Start shading with our program.
        shd_gate.shade(program, |_, _, mut rdr_gate| {
          // Start rendering things with the default render state provided by luminance.
          rdr_gate.render(&RenderState::default(), |mut tess_gate| {
            // Pick the right tessellation to use depending on the mode chosen and render it to the
            // surface.
            match tess_method {
              TessMethod::Direct => tess_gate.render(direct_triangles),
              TessMethod::Indexed => tess_gate.render(indexed_triangles),
              TessMethod::DirectDeinterleaved => tess_gate.render(direct_deinterleaved_triangles),
              TessMethod::IndexedDeinterleaved => tess_gate.render(indexed_deinterleaved_triangles),
            }
          })
        })
      },
    )
    .assume()
    .into_result()
    .unwrap()
}
