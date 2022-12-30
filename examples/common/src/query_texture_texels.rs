//! This program shows how to render two simple triangles, query the texels from the rendered
//! framebuffer and output them in a texture.
//!
//! This example is requires a file system to run as it will write the output to it.
//!
//! <https://docs.rs/luminance>

use crate::{
  shared::{FragSlot, Vertex},
  Example, LoopFeedback, PlatformServices,
};
use image::{save_buffer, ColorType};
use luminance::{
  backend::{Backend, Error},
  context::Context,
  dim::{Dim2, Size2},
  framebuffer::Framebuffer,
  pipeline::PipelineState,
  primitive::Triangle,
  render_state::RenderState,
  shader::{Program, ProgramBuilder},
  texture::{Mipmaps, TextureSampling},
  vertex_entity::{VertexEntity, View},
  vertex_storage::Interleaved,
};
use mint::{Vector2, Vector3};

// We get the shader at compile time from local files
const VS: &'static str = include_str!("simple-vs.glsl");
const FS: &'static str = include_str!("simple-fs.glsl");

// The vertices. We define two triangles.
const TRI_VERTICES: [Vertex; 6] = [
  // first triangle – an RGB one
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
  // second triangle, a purple one, positioned differently
  Vertex {
    co: Vector2 { x: -0.5, y: 0.5 },
    color: Vector3 {
      x: 1.,
      y: 0.2,
      z: 1.,
    },
  },
  Vertex {
    co: Vector2 { x: 0.0, y: -0.5 },
    color: Vector3 {
      x: 0.2,
      y: 1.,
      z: 1.,
    },
  },
  Vertex {
    co: Vector2 { x: 0.5, y: 0.5 },
    color: Vector3 {
      x: 0.2,
      y: 0.2,
      z: 1.,
    },
  },
];

pub struct LocalExample {
  program: Program<Vertex, (), Triangle, FragSlot, ()>,
  triangles: VertexEntity<Vertex, Triangle, Interleaved<Vertex>>,
  framebuffer: Framebuffer<Dim2, FragSlot, ()>,
}

impl Example for LocalExample {
  type Err = Error;

  const TITLE: &'static str = "Query Texture Texels";

  fn bootstrap(
    [width, height]: [u32; 2],
    _: &mut impl PlatformServices,
    ctx: &mut Context<impl Backend>,
  ) -> Result<Self, Self::Err> {
    // we need a program to “shade” our triangles and to tell luminance which is the input vertex
    // type, and we’re not interested in the other two type variables for this sample
    let program = ctx.new_program(
      ProgramBuilder::new()
        .add_vertex_stage(VS)
        .no_primitive_stage()
        .add_shading_stage(FS),
    )?;

    // create tessellation for direct geometry; that is, tessellation that will render vertices by
    // taking one after another in the provided slice
    let triangles = ctx.new_vertex_entity(
      Interleaved::new().set_vertices(TRI_VERTICES),
      [],
      Interleaved::new(),
    )?;

    // the back buffer, which we will make our render into (we make it mutable so that we can change
    // it whenever the window dimensions change)
    let framebuffer = ctx.new_framebuffer(
      Size2::new(width, height),
      Mipmaps::No,
      &TextureSampling::default(),
    )?;

    Ok(Self {
      program,
      triangles,
      framebuffer,
    })
  }

  fn render_frame(
    mut self,
    _: f32,
    _: impl Iterator<Item = crate::InputAction>,
    ctx: &mut Context<impl Backend>,
  ) -> Result<LoopFeedback<Self>, Self::Err> {
    let program = &mut self.program;
    let triangles = &self.triangles;

    // create a new dynamic pipeline that will render to the back buffer and must clear it with
    // pitch black prior to do any render to it
    ctx.with_framebuffer(&self.framebuffer, &PipelineState::default(), |mut frame| {
      frame.with_program(program, |mut frame| {
        frame.with_render_state(&RenderState::default(), |mut frame| {
          frame.render_vertex_entity(triangles.view(..))
        })
      })
    })?;

    // the backbuffer contains our texels
    let texels = ctx.read_texture(&self.framebuffer.layers().frag)?;

    // create a .png file and output it
    save_buffer("./rendered.png", &texels[..], 960, 540, ColorType::Rgb32F).unwrap();

    Ok(LoopFeedback::Exit)
  }
}
