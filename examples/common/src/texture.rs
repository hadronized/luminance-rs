//! This program is a showcase to demonstrate how you can use a texture from an image loaded from the disk.
//! For the purpose of simplicity, the image is stretched to match your window resolution.
//!
//! > Note: for this example, it is recommended to compile with --release to speed up image loading.
//!
//! <https://docs.rs/luminance>

use crate::{
  shared::{load_img, FragSlot},
  Example, InputAction, LoopFeedback, PlatformServices,
};
use luminance::{
  backend::{Backend, Error},
  blending::{Blending, BlendingMode, Equation, Factor},
  context::Context,
  dim::{Dim2, Size2},
  framebuffer::{Back, Framebuffer},
  pipeline::PipelineState,
  pixel::{NormRGB8UI, NormUnsigned},
  primitive::TriangleFan,
  render_state::RenderState,
  shader::{Program, ProgramBuilder, Uni},
  texture::{InUseTexture, Mipmaps, Texture, TextureSampling},
  vertex_entity::{VertexEntity, View},
  vertex_storage::Interleaved,
  Uniforms,
};

const VS: &'static str = include_str!("texture-vs.glsl");
const FS: &'static str = include_str!("texture-fs.glsl");

// we also need a special uniform interface here to pass the texture to the shader
#[derive(Uniforms)]
struct ShaderUniforms {
  tex: Uni<InUseTexture<Dim2, NormUnsigned>>,
}

pub struct LocalExample {
  texture: Texture<Dim2, NormRGB8UI>,
  program: Program<(), (), TriangleFan, FragSlot, ShaderUniforms>,
  vertex_entity: VertexEntity<(), TriangleFan, Interleaved<()>>,
  back_buffer: Framebuffer<Dim2, Back<FragSlot>, Back<()>>,
}

impl Example for LocalExample {
  type Err = Error;

  const TITLE: &'static str = "Texture";

  fn bootstrap(
    [width, height]: [u32; 2],
    platform: &mut impl PlatformServices,
    ctx: &mut Context<impl Backend>,
  ) -> Result<Self, Self::Err> {
    let (img_size, img_texels) = load_img(platform).expect("image to display");
    let texture = ctx.new_texture(
      img_size,
      Mipmaps::No,
      &TextureSampling::default(),
      &img_texels,
    )?;

    let program = ctx.new_program(
      ProgramBuilder::new()
        .add_vertex_stage(VS)
        .no_primitive_stage()
        .add_shading_stage(FS),
    )?;

    // we’ll use an attributeless render here to display a quad on the screen (two triangles); there
    // are over ways to cover the whole screen but this is easier for you to understand; the
    // TriangleFan creates triangles by connecting the third (and next) vertex to the first one
    let vertex_entity = ctx.new_vertex_entity(Interleaved::new(), [], Interleaved::new())?;

    let back_buffer = ctx.back_buffer(Size2::new(width, height))?;

    Ok(LocalExample {
      texture,
      program,
      vertex_entity,
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
          self.back_buffer = ctx.back_buffer(Size2::new(width, height))?;
        }

        _ => (),
      }
    }

    let tex = &self.texture;
    let program = &self.program;
    let vertex_entity = &self.vertex_entity;
    let render_state = &RenderState::default().set_blending(BlendingMode::Combined(Blending {
      equation: Equation::Additive,
      src: Factor::SrcAlpha,
      dst: Factor::Zero,
    }));

    let in_use_texture = ctx.use_texture(tex)?;
    ctx.with_framebuffer(&self.back_buffer, &PipelineState::default(), |mut frame| {
      frame.with_program(program, |mut frame| {
        frame.update(|mut program, unis| program.set(&unis.tex, &in_use_texture))?;

        frame.with_render_state(render_state, |mut frame| {
          frame.render_vertex_entity(vertex_entity.view(..4))
        })
      })
    })?;

    Ok(LoopFeedback::Continue(self))
  }
}
