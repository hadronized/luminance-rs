//! This program shows how to render a single triangle into an offscreen framebuffer and how to
//! render the content of this offscreen framebuffer into the back buffer (i.e. the screen).
//!
//! <https://docs.rs/luminance>

use luminance::{
  backend::Backend,
  context::Context,
  dim::{Dim2, Size2},
  framebuffer::{Back, Framebuffer},
  pipeline::PipelineState,
  pixel::Floating,
  primitive::{Triangle, TriangleFan},
  render_state::RenderState,
  shader::{Program, ProgramBuilder, Stage, Uni},
  texture::{InUseTexture, Mipmaps},
  vertex_entity::{VertexEntity, View},
  vertex_storage::Interleaved,
  Uniforms,
};

use crate::{
  shared::{FragSlot, Vertex},
  Example, InputAction, LoopFeedback, PlatformServices,
};

// we get the shader at compile time from local files
const VS: &'static str = include_str!("simple-vs.glsl");
const FS: &'static str = include_str!("simple-fs.glsl");

// copy shader, at compile time as well
const COPY_VS: &'static str = include_str!("copy-vs.glsl");
const COPY_FS: &'static str = include_str!("copy-fs.glsl");

// a single triangle is enough here
const TRI_VERTICES: [Vertex; 3] = [
  // triangle – an RGB one
  //
  Vertex::new(
    mint::Vector2 { x: 0.5, y: -0.5 },
    mint::Vector3 {
      x: 0.,
      y: 1.,
      z: 0.,
    },
  ),
  Vertex::new(
    mint::Vector2 { x: 0., y: 0.5 },
    mint::Vector3 {
      x: 0.,
      y: 0.,
      z: 1.,
    },
  ),
  Vertex::new(
    mint::Vector2 { x: -0.5, y: -0.5 },
    mint::Vector3 {
      x: 1.,
      y: 0.,
      z: 0.,
    },
  ),
];

// the shader uniform interface is defined there
#[derive(Uniforms)]
struct Uniforms {
  // we only need the source texture (from the framebuffer) to fetch from
  #[uniform(unbound, name = "source_texture")]
  texture: Uni<InUseTexture<Dim2, Floating>>,
}

pub struct LocalExample {
  program: Program<Vertex, Triangle<Vertex>, FragSlot, ()>,
  copy_program: Program<(), TriangleFan<()>, FragSlot, Uniforms>,
  triangle: VertexEntity<Vertex, Triangle<Vertex>, Interleaved<Vertex>>,
  quad: VertexEntity<(), TriangleFan<()>, Interleaved<()>>,
  offscreen_buffer: Framebuffer<Dim2, FragSlot, ()>,
  back_buffer: Framebuffer<Dim2, Back<FragSlot>, Back<()>>,
}

impl Example for LocalExample {
  type Err = luminance::backend::Error;

  const TITLE: &'static str = "Offscreen";

  fn bootstrap(
    [width, height]: [u32; 2],
    _platform: &mut impl PlatformServices,
    ctx: &mut Context<impl Backend>,
  ) -> Result<Self, Self::Err> {
    let program = ctx.new_program(
      ProgramBuilder::new()
        .add_vertex_stage(Stage::<Vertex, Vertex, ()>::new(VS))
        .no_primitive_stage()
        .add_shading_stage(Stage::<Vertex, FragSlot, ()>::new(FS)),
    )?;

    let copy_program = ctx.new_program(
      ProgramBuilder::new()
        .add_vertex_stage(Stage::<(), (), Uniforms>::new(COPY_VS))
        .no_primitive_stage::<TriangleFan<()>>()
        .add_shading_stage(Stage::<(), FragSlot, Uniforms>::new(COPY_FS)),
    )?;

    let triangle = ctx.new_vertex_entity(Interleaved::new().set_vertices(&TRI_VERTICES[..]), [])?;
    let quad = ctx.new_vertex_entity(Interleaved::new(), [])?;
    let fb_size = Size2::new(width, height);
    let offscreen_buffer = ctx.new_framebuffer(fb_size, Mipmaps::No)?;
    let back_buffer = ctx.back_buffer(fb_size)?;

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
    _time: f32,
    actions: impl Iterator<Item = InputAction>,
    ctx: &mut Context<impl Backend>,
  ) -> Result<LoopFeedback<Self>, Self::Err> {
    for action in actions {
      match action {
        InputAction::Quit => return Ok(LoopFeedback::Exit),
        InputAction::Resized { width, height } => {
          self.offscreen_buffer = ctx.new_framebuffer(Size2::new(width, height), Mipmaps::No)?;
        }
        _ => (),
      }
    }

    let program = &self.program;
    let copy_program = &self.copy_program;
    let triangle = &self.triangle;
    let quad = &self.quad;
    let offscreen_buffer = &self.offscreen_buffer;
    let back_buffer = &self.back_buffer;

    // render the triangle in the offscreen framebuffer first
    ctx.with_framebuffer(offscreen_buffer, &PipelineState::default(), |mut frame| {
      frame.with_program(program, |mut frame| {
        frame.with_render_state(&RenderState::default(), |mut frame| {
          frame.render_vertex_entity(triangle.view(..))
        })
      })
    })?;

    // read from the offscreen framebuffer and output it into the back buffer
    ctx.with_framebuffer(back_buffer, &PipelineState::default(), |mut frame| {
      frame.with_program(copy_program, |mut frame| {
        // we must bind the offscreen framebuffer color content so that we can pass it to a shader
        let used_render_layer = frame.use_render_layer(&offscreen_buffer.layers().frag)?;
        // we update the texture with the bound texture
        frame.update(|mut update, unis| update.set(&unis.texture, &used_render_layer))?;

        frame.with_render_state(&RenderState::default(), |mut frame| {
          // this will render the attributeless quad with the offscreen framebuffer color slot
          // bound for the shader to fetch from
          frame.render_vertex_entity(quad.view(..))
        })
      })
    })?;

    Ok(LoopFeedback::Continue(self))
  }
}
