//! This program shows how to render two simple triangles and is the hello world of luminance.
//!
//! <https://docs.rs/luminance>

use crate::{Example, InputAction, LoopFeedback, PlatformServices};
use luminance::{
  backend::Backend,
  context::Context,
  dim::{Dim2, Size2},
  framebuffer::{Back, Framebuffer},
  namespace,
  pipeline::PipelineState,
  primitive::Triangle,
  render_state::RenderState,
  shader::{Program, ProgramBuilder, Stage},
  vertex_entity::{VertexEntity, View},
  vertex_storage::Interleaved,
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

/// Local example; this will be picked by the example runner.
pub struct LocalExample {
  back_buffer: Framebuffer<Dim2, Back<Slots>, Back<()>>,
  // the program will render by mapping our Vertex type as triangles to the color slot, containing a single color
  program: Program<Vertex, Triangle<Vertex>, Slots, ()>,
  triangles: VertexEntity<Vertex, Triangle<Vertex>, Interleaved<Vertex>>,
}

impl Example for LocalExample {
  type Err = luminance::backend::Error;

  const TITLE: &'static str = "Hello, world!";

  fn bootstrap(
    [width, height]: [u32; 2],
    _platform: &mut impl PlatformServices,
    context: &mut Context<impl Backend>,
  ) -> Result<Self, Self::Err> {
    // We need a program to “shade” our triangles
    let program = context
      .new_program(
        ProgramBuilder::new()
          .add_vertex_stage(Stage::<Vertex, Vertex, ()>::new(VS))
          .no_primitive_stage::<Triangle<Vertex>>()
          .add_shading_stage(Stage::<Vertex, Slots, ()>::new(FS)),
      )
      .unwrap();

    let triangles = context
      .new_vertex_entity(Interleaved::new().set_vertices(&TRI_VERTICES[..]), [])
      .unwrap();

    let back_buffer = context.back_buffer(Size2::new(width, height)).unwrap();

    Ok(Self {
      back_buffer,
      program,
      triangles,
    })
  }

  fn render_frame(
    self,
    _time_ms: f32,
    actions: impl Iterator<Item = InputAction>,
    context: &mut Context<impl Backend>,
  ) -> Result<LoopFeedback<Self>, Self::Err> {
    for action in actions {
      match action {
        InputAction::Quit => return Ok(LoopFeedback::Exit),

        _ => (),
      }
    }

    context.with_framebuffer(
      &self.back_buffer,
      &PipelineState::default(),
      |mut with_framebuffer| {
        with_framebuffer.with_program(&self.program, |mut with_program| {
          with_program.with_render_state(&RenderState::default(), |mut with_render_state| {
            with_render_state.render_vertex_entity(self.triangles.view(..))
          })
        })
      },
    )?;

    // Finally, swap the backbuffer with the frontbuffer in order to render our triangles onto your
    // screen.
    Ok(LoopFeedback::Continue(self))
  }
}
