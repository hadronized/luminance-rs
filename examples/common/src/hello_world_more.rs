//! This program shows how to render two simple triangles with different configurations.
//!
//! The direct / indexed methods just show you how you’re supposed to use them (don’t try and find
//! any differences in the rendered images, because there’s none!).
//!
//! Press the <main action> to switch between methods to operate on vertex entities.
//!
//! <https://docs.rs/luminance>

use crate::{Example, InputAction, LoopFeedback, PlatformServices};
use luminance::{
  backend::Backend,
  context::Context,
  dim::{Dim2, Size2},
  framebuffer::Framebuffer,
  namespace,
  pipeline::PipelineState,
  primitive::Triangle,
  render_state::RenderState,
  shader::{Program, ProgramBuilder, Stage},
  vertex_entity::VertexEntity,
  vertex_storage::{Deinterleaved, Interleaved},
  RenderSlots, Vertex,
};

// We get the shader at compile time from local files
const VS: &'static str = include_str!("simple-vs.glsl");
const FS: &'static str = include_str!("simple-fs.glsl");

// Vertex namespace.
//
// A namespace is tag-like type that is used to spawn named indices, allowing to uniquely identify various piece of
// protocol information, such as positions, normals, colors, etc. Theoretically, namespaces and named indices can be
// used for anything and everything.
namespace! {
  VertexNamespace = { "pos", "rgb" }
}

// Our vertex type.
//
// We derive the Vertex trait automatically and map the type to the namespace, so that a mapping can be done between the
// namespace names and the vertex fields.
//
// Also, currently, we need to use #[repr(C))] to ensure Rust is not going to move struct’s fields around.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Vertex)]
#[vertex(namespace = "VertexNamespace")]
struct Vertex {
  pos: mint::Vector2<f32>,

  // Here, we can use the special normalized = <bool> construct to state whether we want integral
  // vertex attributes to be available as normalized floats in the shaders, when fetching them from
  // the vertex buffers. If you set it to "false" or ignore it, you will get non-normalized integer
  // values (i.e. value ranging from 0 to 255 for u8, for instance).
  #[vertex(normalized = "true")]
  rgb: mint::Vector3<u8>,
}

impl Vertex {
  const fn new(pos: mint::Vector2<f32>, rgb: mint::Vector3<u8>) -> Self {
    Self { pos, rgb }
  }
}

// The vertices. We define two triangles.
const TRI_VERTICES: [Vertex; 6] = [
  // First triangle – an RGB one.
  Vertex::new(
    mint::Vector2 { x: 0.5, y: -0.5 },
    mint::Vector3 { x: 0, y: 255, z: 0 },
  ),
  Vertex::new(
    mint::Vector2 { x: 0.0, y: 0.5 },
    mint::Vector3 { x: 0, y: 0, z: 255 },
  ),
  Vertex::new(
    mint::Vector2 { x: -0.5, y: -0.5 },
    mint::Vector3 { x: 255, y: 0, z: 0 },
  ),
  // Second triangle, a purple one, positioned differently.
  Vertex::new(
    mint::Vector2 { x: -0.5, y: 0.5 },
    mint::Vector3 {
      x: 255,
      y: 51,
      z: 255,
    },
  ),
  Vertex::new(
    mint::Vector2 { x: 0.0, y: -0.5 },
    mint::Vector3 {
      x: 51,
      y: 255,
      z: 255,
    },
  ),
  Vertex::new(
    mint::Vector2 { x: 0.5, y: 0.5 },
    mint::Vector3 {
      x: 51,
      y: 51,
      z: 255,
    },
  ),
];

// The vertices, deinterleaved versions. We still define two triangles.
const TRI_DEINT_POS_VERTICES: &[mint::Vector2<f32>] = &[
  mint::Vector2 { x: 0.5, y: -0.5 },
  mint::Vector2 { x: 0.0, y: 0.5 },
  mint::Vector2 { x: -0.5, y: -0.5 },
  mint::Vector2 { x: -0.5, y: 0.5 },
  mint::Vector2 { x: 0.0, y: -0.5 },
  mint::Vector2 { x: 0.5, y: 0.5 },
];

const TRI_DEINT_COLOR_VERTICES: &[mint::Vector3<u8>] = &[
  mint::Vector3 { x: 0, y: 255, z: 0 },
  mint::Vector3 { x: 0, y: 0, z: 255 },
  mint::Vector3 { x: 255, y: 0, z: 0 },
  mint::Vector3 {
    x: 255,
    y: 51,
    z: 255,
  },
  mint::Vector3 {
    x: 51,
    y: 255,
    z: 255,
  },
  mint::Vector3 {
    x: 51,
    y: 51,
    z: 255,
  },
];

// Indices into TRI_VERTICES to use to build up the triangles.
const TRI_INDICES: [u32; 6] = [
  0, 1, 2, // First triangle.
  3, 4, 5, // Second triangle.
];

// Another namespace for render slots (see below).
namespace! {
  RenderSlotNamespace = { "frag" }
}

// Render slots.
//
// A render slot represents the channels the end stage of a shader program is going to end up writing to. In our case,
// since we are only interested in rendering the color of each pixel, we will just have one single channel for the
// color.
#[derive(Clone, Copy, Debug, PartialEq, RenderSlots)]
#[slot(namespace = "RenderSlotNamespace")]
pub struct Slots {
  frag: mint::Vector3<f32>,
}

// Convenience type to demonstrate the difference between direct, indirect (indexed), interleaved and deinterleaved
// vertex entities.
#[derive(Copy, Clone, Debug)]
enum Method {
  Direct,
  Indexed,
  DirectDeinterleaved,
  IndexedDeinterleaved,
}

impl Method {
  fn toggle(self) -> Self {
    match self {
      Method::Direct => Method::Indexed,
      Method::Indexed => Method::DirectDeinterleaved,
      Method::DirectDeinterleaved => Method::IndexedDeinterleaved,
      Method::IndexedDeinterleaved => Method::Direct,
    }
  }
}

/// Local example; this will be picked by the example runner.
pub struct LocalExample {
  back_buffer: Framebuffer<Dim2, Slots, ()>,
  // the program will render by mapping our Vertex type as triangles to the color slot, containing a single color
  program: Program<Vertex, Triangle<Vertex>, Slots, ()>,
  direct_triangles: VertexEntity<Vertex, Interleaved<Vertex>>,
  indexed_triangles: VertexEntity<Vertex, Interleaved<Vertex>>,
  direct_deinterleaved_triangles: VertexEntity<Vertex, Deinterleaved<Vertex>>,
  indexed_deinterleaved_triangles: VertexEntity<Vertex, Deinterleaved<Vertex>>,
  method: Method,
}

impl Example for LocalExample {
  type Err = luminance::backend::Error;

  fn bootstrap(
    _platform: &mut impl PlatformServices,
    context: &mut Context<impl Backend>,
  ) -> Result<Self, Self::Err>
  where
    Self::Err: From<luminance::backend::Error>,
  {
    // We need a program to “shade” our triangles
    let program = context
      .new_program(
        ProgramBuilder::new()
          .add_vertex_stage(Stage::<Vertex, Vertex, ()>::new(VS))
          .no_primitive_stage::<Triangle<Vertex>>()
          .add_shading_stage(Stage::<Vertex, Slots, ()>::new(FS)),
      )
      .unwrap();

    // Create a vertex entity for direct geometry; that is, a vertex entity that will render vertices by
    // taking one after another in the provided slice.
    let direct_triangles = context
      .new_vertex_entity(Interleaved::new().set_vertices(&TRI_VERTICES[..]), [])
      .unwrap();

    // Indexed vertex entity; that is, the vertices will be picked by using the indexes provided
    // by the second slice and this indexes will reference the first slice (useful not to duplicate
    // vertices on more complex objects than just two triangles).
    let indexed_triangles = context
      .new_vertex_entity(
        Interleaved::new().set_vertices(&TRI_VERTICES[..]),
        &TRI_INDICES[..],
      )
      .unwrap();

    // Create a direct, deinterleaved vertex entity; such vertex entity allows to separate vertex
    // attributes in several contiguous regions of memory.
    let direct_deinterleaved_triangles = context
      .new_vertex_entity(
        Deinterleaved::new()
          .set_components::<"pos">(&TRI_DEINT_POS_VERTICES[..])
          .set_components::<"rgb">(&TRI_DEINT_COLOR_VERTICES[..]),
        [],
      )
      .unwrap();

    // Create an indexed, deinterleaved vertex entity.
    let indexed_deinterleaved_triangles = context
      .new_vertex_entity(
        Deinterleaved::new()
          .set_components::<"pos">(&TRI_DEINT_POS_VERTICES[..])
          .set_components::<"rgb">(&TRI_DEINT_COLOR_VERTICES[..]),
        &TRI_INDICES[..],
      )
      .unwrap();

    let method = Method::Direct;

    let back_buffer = context.back_buffer(Size2::new(800, 600)).unwrap();

    Ok(Self {
      back_buffer,
      program,
      direct_triangles,
      indexed_triangles,
      direct_deinterleaved_triangles,
      indexed_deinterleaved_triangles,
      method,
    })
  }

  fn render_frame(
    mut self,
    _time_ms: f32,
    actions: impl Iterator<Item = InputAction>,
    context: &mut Context<impl Backend>,
  ) -> Result<LoopFeedback<Self>, Self::Err> {
    for action in actions {
      match action {
        InputAction::Quit => return Ok(LoopFeedback::Exit),

        InputAction::MainToggle => {
          self.method = self.method.toggle();
          log::info!("now rendering {:?}", self.method);
        }

        _ => (),
      }
    }

    context.with_framebuffer(
      &self.back_buffer,
      &PipelineState::default(),
      |mut with_framebuffer| {
        with_framebuffer.with_program(&self.program, |mut with_program| {
          with_program.with_render_state(
            &RenderState::default(),
            |mut with_render_state| match self.method {
              Method::Direct => {
                with_render_state.render_vertex_entity(self.direct_triangles.view())
              }
              Method::Indexed => {
                with_render_state.render_vertex_entity(self.indexed_triangles.view())
              }
              Method::DirectDeinterleaved => {
                with_render_state.render_vertex_entity(self.direct_deinterleaved_triangles.view())
              }
              Method::IndexedDeinterleaved => {
                with_render_state.render_vertex_entity(self.indexed_deinterleaved_triangles.view())
              }
            },
          )
        })
      },
    )?;

    // Finally, swap the backbuffer with the frontbuffer in order to render our triangles onto your
    // screen.
    Ok(LoopFeedback::Continue(self))
  }
}
