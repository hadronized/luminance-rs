//! This program shows how to render a single triangle into an offscreen framebuffer with two
//! target textures, and how to render the contents of these textures into the back
//! buffer (i.e. the screen), combining data from both.
//!
//! <https://docs.rs/luminance>

use crate::{
  shared::{FragSlot, Vertex},
  Example, InputAction, LoopFeedback, PlatformServices,
};
use luminance::{
  backend::{Backend, Error},
  context::Context,
  dim::{Dim2, Size2},
  framebuffer::{Back, Framebuffer},
  pipeline::PipelineState,
  pixel::{NormR8UI, NormRGB8UI, NormUnsigned},
  primitive::{Triangle, TriangleFan},
  render_state::RenderState,
  shader::{Program, ProgramBuilder, Uni},
  texture::{InUseTexture, Mipmaps, TextureSampling},
  vertex_entity::{VertexEntity, VertexEntityBuilder, View},
  vertex_storage::{Interleaved, Interleaving},
  RenderSlots, Uniforms,
};
use mint::{Vector2, Vector3};

// we get the shader at compile time from local files
const VS: &'static str = include_str!("simple-vs.glsl");
const FS: &'static str = include_str!("multi-fs.glsl");

// copy shader, at compile time as well
const COPY_VS: &'static str = include_str!("copy-vs.glsl");
const COPY_FS: &'static str = include_str!("copy-multi-fs.glsl");

// a single triangle is enough here
const TRI_VERTICES: [Vertex; 3] = [
  // triangle – an RGB one
  Vertex {
    co: Vector2 { x: 0.5, y: -0.5 },
    color: Vector3 {
      x: 0.,
      y: 1.,
      z: 0.,
    },
  },
  Vertex {
    co: Vector2 { x: 0.0, y: 0.5 },
    color: Vector3 {
      x: 0.,
      y: 0.,
      z: 1.,
    },
  },
  Vertex {
    co: Vector2 { x: -0.5, y: -0.5 },
    color: Vector3 {
      x: 1.,
      y: 0.,
      z: 0.,
    },
  },
];

// the shader uniform interface is defined there
#[derive(Uniforms)]
struct ShaderInterface {
  // we only need the source texture (from the framebuffer) to fetch from
  #[uniform(name = "source_texture_color")]
  texture_color: Uni<InUseTexture<Dim2, NormUnsigned>>,
  #[uniform(name = "source_texture_white")]
  texture_white: Uni<InUseTexture<Dim2, NormUnsigned>>,
}

// FIXME: use _frag_color and _frag_white and rename the slots with something like #[slot(rename = "…")]
#[derive(RenderSlots)]
struct OffscreenSlots {
  frag_color: NormRGB8UI,
  frag_white: NormR8UI,
}

pub struct LocalExample {
  program: Program<Vertex, (), Triangle, OffscreenSlots, ()>,
  copy_program: Program<(), (), TriangleFan, FragSlot, ShaderInterface>,
  triangle: VertexEntity<Vertex, Triangle, Interleaving>,
  quad: VertexEntity<(), TriangleFan, ()>,
  offscreen_buffer: Framebuffer<Dim2, OffscreenSlots, ()>,
  back_buffer: Framebuffer<Dim2, Back<FragSlot>, Back<()>>,
}

impl Example for LocalExample {
  type Err = Error;

  const TITLE: &'static str = "Multi Render Target";

  fn bootstrap(
    [width, height]: [u32; 2],
    _: &mut impl PlatformServices,
    ctx: &mut Context<impl Backend>,
  ) -> Result<Self, Self::Err> {
    let program = ctx.new_program(
      ProgramBuilder::new()
        .add_vertex_stage(VS)
        .no_primitive_stage()
        .add_shading_stage(FS),
    )?;

    let copy_program = ctx.new_program(
      ProgramBuilder::new()
        .add_vertex_stage(COPY_VS)
        .no_primitive_stage()
        .add_shading_stage(COPY_FS),
    )?;

    let triangle = ctx.new_vertex_entity(
      VertexEntityBuilder::new().add_vertices(Interleaved::new().set_vertices(TRI_VERTICES)),
    )?;

    // we’ll need an attributeless quad to fetch in full screen
    let quad = ctx.new_vertex_entity(VertexEntityBuilder::new())?;

    // the offscreen buffer; defined with a dummy 10×10 dimension
    let size = Size2::new(width, height);
    let offscreen_buffer = ctx.new_framebuffer(size, Mipmaps::No, &TextureSampling::default())?;

    let back_buffer = ctx.back_buffer(size)?;

    Ok(Self {
      program,
      copy_program,
      triangle,
      quad,
      offscreen_buffer,
      back_buffer,
    })
  }

  fn render_frame(
    mut self,
    _: f32,
    actions: impl Iterator<Item = InputAction>,
    ctx: &mut Context<impl Backend>,
  ) -> Result<LoopFeedback<Self>, Self::Err> {
    for action in actions {
      match action {
        InputAction::Quit => return Ok(LoopFeedback::Exit),

        InputAction::Resized { width, height } => {
          // simply ask another offscreen framebuffer at the right dimension (no allocation / reallocation)
          self.offscreen_buffer = ctx.new_framebuffer(
            Size2::new(width, height),
            Mipmaps::No,
            &TextureSampling::default(),
          )?;
        }

        _ => (),
      }
    }

    let program = &self.program;
    let copy_program = &self.copy_program;
    let triangle = &self.triangle;
    let quad = &self.quad;
    let offscreen_buffer = &self.offscreen_buffer;

    // render the triangle in the offscreen framebuffer first
    ctx.with_framebuffer(offscreen_buffer, &PipelineState::default(), |mut frame| {
      frame.with_program(program, |mut frame| {
        frame.with_render_state(&RenderState::default(), |mut frame| {
          frame.render_vertex_entity(triangle.view(..))
        })
      })
    })?;

    // read from the offscreen framebuffer and output it into the back buffer
    ctx.with_framebuffer(&self.back_buffer, &PipelineState::default(), |mut frame| {
      let layers = offscreen_buffer.layers();
      let bound_color = frame.use_texture(&layers.frag_color)?;
      let bound_white = frame.use_texture(&layers.frag_white)?;

      frame.with_program(copy_program, |mut frame| {
        frame.update(|mut update, unis| {
          update.set(&unis.texture_color, &bound_color)?;
          update.set(&unis.texture_white, &bound_white)
        })?;

        frame.with_render_state(&RenderState::default(), |mut frame| {
          frame.render_vertex_entity(quad.view(..4))
        })
      })
    })?;

    Ok(LoopFeedback::Continue(self))
  }
}
