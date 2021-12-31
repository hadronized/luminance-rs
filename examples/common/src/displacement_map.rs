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

use crate::{
  shared::{load_texture, RGBTexture},
  Example, InputAction, LoopFeedback, PlatformServices,
};
use luminance::UniformInterface;
use luminance_front::{
  blending::{Blending, Equation, Factor},
  context::GraphicsContext,
  framebuffer::Framebuffer,
  pipeline::{PipelineState, TextureBinding},
  pixel::NormUnsigned,
  render_state::RenderState,
  shader::{types::Vec2, Program, Uniform},
  tess::{Mode, Tess},
  texture::{Dim2, Sampler, TexelUpload},
  Backend,
};

const VS: &str = include_str!("./displacement-map-resources/displacement-map-vs.glsl");
const FS: &str = include_str!("./displacement-map-resources/displacement-map-fs.glsl");

#[derive(UniformInterface)]
struct ShaderInterface {
  image: Uniform<TextureBinding<Dim2, NormUnsigned>>,
  displacement_map_1: Uniform<TextureBinding<Dim2, NormUnsigned>>,
  displacement_map_2: Uniform<TextureBinding<Dim2, NormUnsigned>>,
  displacement_scale: Uniform<f32>,
  time: Uniform<f32>,
  window_dimensions: Uniform<Vec2<f32>>,
}

pub struct LocalExample {
  image: RGBTexture,
  displacement_maps: [RGBTexture; 2],
  program: Program<(), (), ShaderInterface>,
  tess: Tess<()>,
  displacement_scale: f32,
}

impl Example for LocalExample {
  fn bootstrap(
    platform: &mut impl PlatformServices,
    context: &mut impl GraphicsContext<Backend = Backend>,
  ) -> Self {
    let image = load_texture(context, platform).expect("texture to displace");
    let displacement_maps = [
      load_displacement_map(
        context,
        include_bytes!("./displacement-map-resources/displacement_1.png"),
      ),
      load_displacement_map(
        context,
        include_bytes!("./displacement-map-resources/displacement_2.png"),
      ),
    ];

    let program = context
      .new_shader_program::<(), (), ShaderInterface>()
      .from_strings(VS, None, None, FS)
      .expect("Could not create shader program")
      .ignore_warnings();

    let tess = context
      .new_tess()
      .set_render_vertex_nb(4)
      .set_mode(Mode::TriangleFan)
      .build()
      .unwrap();

    let displacement_scale = 0.01;

    Self {
      image,
      displacement_maps,
      program,
      tess,
      displacement_scale,
    }
  }

  fn render_frame(
    mut self,
    t: f32,
    back_buffer: Framebuffer<Dim2, (), ()>,
    actions: impl Iterator<Item = InputAction>,
    context: &mut impl GraphicsContext<Backend = Backend>,
  ) -> LoopFeedback<Self> {
    for action in actions {
      match action {
        InputAction::Quit => return LoopFeedback::Exit,
        InputAction::Forward => self.displacement_scale = (self.displacement_scale + 0.01).min(1.),
        InputAction::Backward => self.displacement_scale = (self.displacement_scale - 0.01).max(0.),
        _ => (),
      }
    }

    let image = &mut self.image;
    let [ref mut displacement_map_0, ref mut displacement_map_1] = self.displacement_maps;
    let displacement_scale = self.displacement_scale;
    let render_state = &RenderState::default().set_blending(Blending {
      equation: Equation::Additive,
      src: Factor::SrcAlpha,
      dst: Factor::Zero,
    });
    let tess = &self.tess;
    let program = &mut self.program;

    let render = context
      .new_pipeline_gate()
      .pipeline(
        &back_buffer,
        &PipelineState::default(),
        |pipeline, mut shading_gate| {
          let bound_texture = pipeline.bind_texture(image).unwrap();
          let bound_displacement_1 = pipeline.bind_texture(displacement_map_0)?;
          let bound_displacement_2 = pipeline.bind_texture(displacement_map_1)?;

          shading_gate.shade(program, |mut interface, uni, mut render_gate| {
            let back_buffer_size = back_buffer.size();
            interface.set(&uni.image, bound_texture.binding());
            interface.set(&uni.displacement_map_1, bound_displacement_1.binding());
            interface.set(&uni.displacement_map_2, bound_displacement_2.binding());
            interface.set(&uni.displacement_scale, displacement_scale);
            interface.set(&uni.time, t);
            interface.set(
              &uni.window_dimensions,
              Vec2::new(back_buffer_size[0] as f32, back_buffer_size[1] as f32),
            );

            render_gate.render(render_state, |mut tess_gate| tess_gate.render(tess))
          })
        },
      )
      .assume();

    if render.is_ok() {
      LoopFeedback::Continue(self)
    } else {
      LoopFeedback::Exit
    }
  }
}

fn load_displacement_map(
  context: &mut impl GraphicsContext<Backend = Backend>,
  bytes: &[u8],
) -> RGBTexture {
  let img = image::load_from_memory_with_format(bytes, image::ImageFormat::Png)
    .expect("Could not load displacement map")
    .to_rgb8();
  let (width, height) = img.dimensions();
  let texels = img.as_raw();

  context
    .new_texture_raw(
      [width, height],
      Sampler::default(),
      TexelUpload::base_level(texels, 0),
    )
    .map_err(|e| log::error!("error while creating texture: {}", e))
    .ok()
    .expect("load displacement map")
}
