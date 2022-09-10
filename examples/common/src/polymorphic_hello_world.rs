//! This program shows how to render two simple triangles and is the hello world of luminance. This version
//! is slightly different from the hello_world.rs program as this one uses the polymorphic interface. You
//! will not find any usage of luminance-front. All the comments were removed to focus only on the
//! polymorphic interface. You are advised to look at hello_world.rs to understand the rest of the code.
//!
//! The direct / indexed methods just show you how you’re supposed to use them (don’t try and find
//! any differences in the rendered images, because there’s none!).
//!
//! Press the <main action> to switch between direct tessellation and indexed tessellation.
//!
//! <https://docs.rs/luminance>

use crate::{Example, InputAction, LoopFeedback, PlatformServices};
use luminance::{
  backend::{
    framebuffer::FramebufferBackBuffer, pipeline::Pipeline, render_gate::RenderGate,
    vertex_entity::Tess as TessBackend, tess_gate::TessGate,
  },
  context::GraphicsContext,
  framebuffer::Framebuffer,
  pipeline::PipelineState,
  render_state::RenderState,
  shader::Program,
  vertex_entity::{Deinterleaved, Interleaved, Mode, Tess},
  texture::Dim2,
  Semantics, Vertex,
};

const VS: &'static str = include_str!("simple-vs.glsl");
const FS: &'static str = include_str!("simple-fs.glsl");

#[derive(Clone, Copy, Debug, Eq, PartialEq, Semantics)]
pub enum Semantics {
  #[sem(name = "co", repr = "[f32; 2]", wrapper = "VertexPosition")]
  Position,
  #[sem(name = "color", repr = "[u8; 3]", wrapper = "VertexColor")]
  Color,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Vertex)]
#[vertex(sem = "Semantics")]
pub struct Vertex {
  pos: VertexPosition,
  #[vertex(normalized = "true")]
  rgb: VertexColor,
}

const TRI_VERTICES: [Vertex; 6] = [
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

const TRI_INDICES: [u8; 6] = [0, 1, 2, 3, 4, 5];

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

// This is the main important difference. Because luminance uses a lot of traits, we are going to create a local trait
// that alias all of them. This is not a real trait alias, but a type system trick that will bring a combination of
// trait when this trait is used.
pub trait Luminance:
  Pipeline<Dim2> // < this is so wrong…
  + FramebufferBackBuffer
  + RenderGate
  + TessBackend<(), (), (), Interleaved>
  + TessGate<Vertex, (), (), Interleaved>
  + TessGate<Vertex, u8, (), Interleaved>
  + TessGate<Vertex, (), (), Deinterleaved>
  + TessGate<Vertex, u8, (), Deinterleaved>
{
}

impl<B> Luminance for B where
  B: Pipeline<Dim2> // < this is so wrong…
    + FramebufferBackBuffer
    + RenderGate
    + TessBackend<(), (), (), Interleaved>
    + TessGate<Vertex, (), (), Interleaved>
    + TessGate<Vertex, u8, (), Interleaved>
    + TessGate<Vertex, (), (), Deinterleaved>
    + TessGate<Vertex, u8, (), Deinterleaved>
{
}

// The other difference is that not using luminance-front implies that all types have the B type variable (B = backend),
// so we have to annotate it everywhere, or use dyn trait.
pub struct LocalExample<B>
where
  B: Luminance,
{
  program: Program<B, Semantics, (), ()>,
  direct_triangles: Tess<B, Vertex>,
  indexed_triangles: Tess<B, Vertex, u8>,
  direct_deinterleaved_triangles: Tess<B, Vertex, (), (), Deinterleaved>,
  indexed_deinterleaved_triangles: Tess<B, Vertex, u8, (), Deinterleaved>,
  tess_method: TessMethod,
}

impl<B> Example<B> for LocalExample<B>
where
  B: Luminance,
{
  fn bootstrap(
    _platform: &mut impl PlatformServices,
    context: &mut impl GraphicsContext<Backend = B>,
  ) -> Self {
    // We need a program to “shade” our triangles and to tell luminance which is the input vertex
    // type, and we’re not interested in the other two type variables for this sample.
    let program = context
      .new_shader_program::<Semantics, (), ()>()
      .from_strings(VS, None, None, FS)
      .expect("program creation")
      .ignore_warnings();

    // Create tessellation for direct geometry; that is, tessellation that will render vertices by
    // taking one after another in the provided slice.
    let direct_triangles = context
      .new_tess()
      .set_vertices(&TRI_VERTICES[..])
      .set_mode(Mode::Triangle)
      .build()
      .unwrap();

    // Create indexed tessellation; that is, the vertices will be picked by using the indexes provided
    // by the second slice and this indexes will reference the first slice (useful not to duplicate
    // vertices on more complex objects than just two triangles).
    let indexed_triangles = context
      .new_tess()
      .set_vertices(&TRI_VERTICES[..])
      .set_indices(&TRI_INDICES[..])
      .set_mode(Mode::Triangle)
      .build()
      .unwrap();

    // Create direct, deinterleaved tesselations; such tessellations allow to separate vertex
    // attributes in several contiguous regions of memory.
    let direct_deinterleaved_triangles = context
      .new_deinterleaved_tess::<Vertex, ()>()
      .set_attributes(&TRI_DEINT_POS_VERTICES[..])
      .set_attributes(&TRI_DEINT_COLOR_VERTICES[..])
      .set_mode(Mode::Triangle)
      .build()
      .unwrap();

    // Create indexed, deinterleaved tessellations; have your cake and fucking eat it, now.
    let indexed_deinterleaved_triangles = context
      .new_deinterleaved_tess::<Vertex, ()>()
      .set_attributes(&TRI_DEINT_POS_VERTICES[..])
      .set_attributes(&TRI_DEINT_COLOR_VERTICES[..])
      .set_indices(&TRI_INDICES[..])
      .set_mode(Mode::Triangle)
      .build()
      .unwrap();

    let tess_method = TessMethod::Direct;

    Self {
      program,
      direct_triangles,
      indexed_triangles,
      direct_deinterleaved_triangles,
      indexed_deinterleaved_triangles,
      tess_method,
    }
  }

  fn render_frame(
    mut self,
    _time_ms: f32,
    back_buffer: Framebuffer<B, Dim2, (), ()>,
    actions: impl Iterator<Item = InputAction>,
    context: &mut impl GraphicsContext<Backend = B>,
  ) -> LoopFeedback<Self> {
    for action in actions {
      match action {
        InputAction::Quit => return LoopFeedback::Exit,

        InputAction::MainToggle => {
          self.tess_method = self.tess_method.toggle();
          log::info!("now rendering {:?}", self.tess_method);
        }

        _ => (),
      }
    }

    let program = &mut self.program;
    let direct_triangles = &self.direct_triangles;
    let indexed_triangles = &self.indexed_triangles;
    let direct_deinterleaved_triangles = &self.direct_deinterleaved_triangles;
    let indexed_deinterleaved_triangles = &self.indexed_deinterleaved_triangles;
    let tess_method = &self.tess_method;

    // Create a new dynamic pipeline that will render to the back buffer and must clear it with
    // pitch black prior to do any render to it.
    let render = context
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
                TessMethod::IndexedDeinterleaved => {
                  tess_gate.render(indexed_deinterleaved_triangles)
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
      LoopFeedback::Continue(self)
    } else {
      LoopFeedback::Exit
    }
  }
}
