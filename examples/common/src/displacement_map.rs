//! This program is a showcase to demonstrate how you can use grayscale textures to displace the
//! lookup location of a texture. This is commonly referred to as "displacement mapping." Here we
//! demonstrate using multiple displacement maps to offset the lookup in different directions. The
//! displacement also uses time as an input, so the displacement changes according to a sine wave.
//!
//! The texture path is read from the command line interface and is the sole argument.
//!
//! The image is stretched to match the window size, but the displacement maps are tiled and true to
//! pixel size regardless of the window size.
//!
//! Press <Up> or <Down> actions to increase or decrease the scale factor of the
//! displacement, respectively.
//!
//! > Note: for this example, it is recommended to compile with --release to speed up image loading.
//!
//! <https://docs.rs/luminance>

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
  vertex_entity::{VertexEntity, VertexEntityBuilder, View},
  Uniforms,
};
use mint::Vector2;

use crate::{
  shared::{load_img, FragSlot},
  Example, InputAction, LoopFeedback, PlatformServices,
};

const VS: &str = include_str!("./displacement-map-resources/displacement-map-vs.glsl");
const FS: &str = include_str!("./displacement-map-resources/displacement-map-fs.glsl");

#[derive(Uniforms)]
struct ShaderInterface {
  image: Uni<InUseTexture<Dim2, NormUnsigned>>,
  displacement_map_1: Uni<InUseTexture<Dim2, NormUnsigned>>,
  displacement_map_2: Uni<InUseTexture<Dim2, NormUnsigned>>,
  displacement_scale: Uni<f32>,
  time: Uni<f32>,
  window_dimensions: Uni<Vector2<f32>>,
}

pub struct LocalExample {
  img_tex: Texture<Dim2, NormRGB8UI>,
  displacement_maps: [Texture<Dim2, NormRGB8UI>; 2],
  program: Program<(), (), TriangleFan, FragSlot, ShaderInterface>,
  quad: VertexEntity<(), TriangleFan, ()>,
  displacement_scale: f32,
  back_buffer: Framebuffer<Dim2, Back<FragSlot>, Back<()>>,
}

impl Example for LocalExample {
  type Err = Error;

  const TITLE: &'static str = "Displacement Map";

  fn bootstrap(
    [width, height]: [u32; 2],
    platform: &mut impl PlatformServices,
    ctx: &mut Context<impl Backend>,
  ) -> Result<Self, Self::Err> {
    let (img_size, img) = load_img(platform).expect("image to displace");
    let img_tex = ctx.new_texture(img_size, Mipmaps::No, &TextureSampling::default(), &img[..])?;

    let displacement_maps = [
      load_displacement_map(
        ctx,
        include_bytes!("./displacement-map-resources/displacement_1.png"),
      ),
      load_displacement_map(
        ctx,
        include_bytes!("./displacement-map-resources/displacement_2.png"),
      ),
    ];

    let program = ctx.new_program(
      ProgramBuilder::new()
        .add_vertex_stage(VS)
        .no_primitive_stage()
        .add_shading_stage(FS),
    )?;

    let quad = ctx.new_vertex_entity(VertexEntityBuilder::new())?;
    let displacement_scale = 0.01;

    let back_buffer = ctx.back_buffer(Size2::new(width, height))?;

    Ok(Self {
      img_tex,
      displacement_maps,
      program,
      quad,
      displacement_scale,
      back_buffer,
    })
  }

  fn render_frame(
    mut self,
    t: f32,
    actions: impl Iterator<Item = InputAction>,
    ctx: &mut Context<impl Backend>,
  ) -> Result<LoopFeedback<Self>, Self::Err> {
    for action in actions {
      match action {
        InputAction::Quit => return Ok(LoopFeedback::Exit),
        InputAction::Forward => {
          self.displacement_scale = (self.displacement_scale + 0.01).min(1.);
          log::info!("new displacement scale: {}", self.displacement_scale);
        }
        InputAction::Backward => {
          self.displacement_scale = (self.displacement_scale - 0.01).max(0.);
          log::info!("new displacement scale: {}", self.displacement_scale);
        }
        _ => (),
      }
    }

    let img_tex = &self.img_tex;
    let [ref displacement_map_1, ref displacement_map_2] = self.displacement_maps;
    let displacement_scale = self.displacement_scale;
    let render_state = &RenderState::default().set_blending(BlendingMode::Combined(Blending {
      equation: Equation::Additive,
      src: Factor::SrcAlpha,
      dst: Factor::Zero,
    }));
    let quad = &self.quad;
    let program = &self.program;
    let frame_size = self.back_buffer.size();

    ctx.with_framebuffer(&self.back_buffer, &PipelineState::default(), |mut frame| {
      let bound_tex = frame.use_texture(img_tex)?;
      let bound_displacement_1 = frame.use_texture(displacement_map_1)?;
      let bound_displacement_2 = frame.use_texture(displacement_map_2)?;
      frame.with_program(program, |mut frame| {
        frame.update(|mut p, unis| {
          p.set(&unis.image, &bound_tex)?;
          p.set(&unis.displacement_map_1, &bound_displacement_1)?;
          p.set(&unis.displacement_map_2, &bound_displacement_2)?;
          p.set(&unis.time, &t)?;
          p.set(&unis.displacement_scale, &displacement_scale)?;
          p.set(
            &unis.window_dimensions,
            &Vector2 {
              x: frame_size.width as f32,
              y: frame_size.height as f32,
            },
          )
        })?;

        frame.with_render_state(render_state, |mut frame| {
          frame.render_vertex_entity(quad.view(..4))
        })
      })
    })?;

    Ok(LoopFeedback::Continue(self))
  }
}

fn load_displacement_map(
  ctx: &mut Context<impl Backend>,
  bytes: &[u8],
) -> Texture<Dim2, NormRGB8UI> {
  let img = image::load_from_memory_with_format(bytes, image::ImageFormat::Png)
    .expect("Could not load displacement map")
    .to_rgb8();
  let (width, height) = img.dimensions();
  let texels = img.as_raw();

  ctx
    .new_texture(
      Size2::new(width, height),
      Mipmaps::No,
      &TextureSampling::default(),
      texels,
    )
    .map_err(|e| log::error!("error while creating texture: {}", e))
    .ok()
    .expect("load displacement map")
}
