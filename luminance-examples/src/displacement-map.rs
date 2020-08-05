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
//! Press <w> or <s> to increase or decrease the scale factor of the displacement, respectively.
//! Press <escape> to quit or close the window.
//!
//! > Note: for this example, it is recommended to compile with --release to speed up image loading.
//!
//! https://docs.rs/luminance

use glfw::{Action, Context as _, Key, WindowEvent};
use luminance::blending::{Blending, Equation, Factor};
use luminance::context::GraphicsContext;
use luminance::pipeline::{PipelineState, TextureBinding};
use luminance::pixel::{NormRGB8UI, NormUnsigned};
use luminance::render_state::RenderState;
use luminance::shader::Uniform;
use luminance::tess::Mode;
use luminance::texture::{Dim2, GenMipmaps, Sampler};
use luminance_derive::UniformInterface;
use luminance_glfw::GlfwSurface;
use luminance_windowing::{WindowDim, WindowOpt};
use std::env;
use std::time::Instant; // used to get the CLI arguments

const VS: &str = include_str!("./displacement-map-resources/displacement-map-vs.glsl");
const FS: &str = include_str!("./displacement-map-resources/displacement-map-fs.glsl");

#[derive(UniformInterface)]
struct ShaderInterface {
  image: Uniform<TextureBinding<Dim2, NormUnsigned>>,
  displacement_map_1: Uniform<TextureBinding<Dim2, NormUnsigned>>,
  displacement_map_2: Uniform<TextureBinding<Dim2, NormUnsigned>>,
  displacement_scale: Uniform<f32>,
  time: Uniform<f32>,
  window_dimensions: Uniform<[f32; 2]>,
}

fn main() {
  let texture_path = env::args()
    .nth(1)
    .expect("Please provide an image path for the core texture");
  let texture_image = image::open(&texture_path)
    .expect("Could not load image from path")
    .flipv()
    .to_rgb();
  let (width, height) = texture_image.dimensions();

  let displacement_map_1 = image::load_from_memory_with_format(
    include_bytes!("./displacement-map-resources/displacement_1.png"),
    image::ImageFormat::Png,
  )
  .expect("Could not load displacement map")
  .to_rgb();

  let displacement_map_2 = image::load_from_memory_with_format(
    include_bytes!("./displacement-map-resources/displacement_2.png"),
    image::ImageFormat::Png,
  )
  .expect("Could not load displacement map")
  .to_rgb();

  let dim = WindowDim::Windowed { width, height };
  let mut surface = GlfwSurface::new_gl33("Displacement Map", WindowOpt::default().set_dim(dim))
    .expect("Could not create GLFW surface");

  let texels = texture_image.into_raw();
  let mut tex = surface
    .new_texture::<Dim2, NormRGB8UI>([width, height], 0, Sampler::default())
    .expect("Could not create luminance texture");
  tex.upload_raw(GenMipmaps::No, &texels).unwrap();

  let texels = displacement_map_1.into_raw();
  let mut displacement_tex_1 = surface
    .new_texture::<Dim2, NormRGB8UI>([128, 128], 0, Sampler::default())
    .expect("Could not create luminance texture");
  displacement_tex_1
    .upload_raw(GenMipmaps::No, &texels)
    .unwrap();

  let texels = displacement_map_2.into_raw();
  let mut displacement_tex_2 = surface
    .new_texture::<Dim2, NormRGB8UI>([101, 101], 0, Sampler::default())
    .expect("Could not create luminance texture");
  displacement_tex_2
    .upload_raw(GenMipmaps::No, &texels)
    .unwrap();

  let mut program = surface
    .new_shader_program::<(), (), ShaderInterface>()
    .from_strings(VS, None, None, FS)
    .expect("Could not create shader program")
    .ignore_warnings();

  let tess = surface
    .new_tess()
    .set_vertex_nb(4)
    .set_mode(Mode::TriangleFan)
    .build()
    .unwrap();

  let mut back_buffer = surface.back_buffer().unwrap();
  let start_time = Instant::now();
  let render_state = RenderState::default().set_blending(Blending {
    equation: Equation::Additive,
    src: Factor::SrcAlpha,
    dst: Factor::Zero,
  });
  let mut resize = false;
  let mut displacement_scale: f32 = 0.010;

  'app: loop {
    surface.window.glfw.poll_events();
    for (_, event) in surface.events_rx.try_iter() {
      match event {
        WindowEvent::Close | WindowEvent::Key(Key::Escape, _, Action::Release, _) => break 'app,
        WindowEvent::Key(Key::W, _, Action::Press, _) => {
          displacement_scale = (displacement_scale + 0.005).min(1.0);
        }
        WindowEvent::Key(Key::S, _, Action::Press, _) => {
          displacement_scale = (displacement_scale - 0.005).max(0.0);
        }
        WindowEvent::FramebufferSize(..) => {
          resize = true;
        }
        _ => (),
      }
    }

    if resize {
      back_buffer = surface.back_buffer().unwrap();
      resize = false;
    }

    let elapsed = start_time.elapsed();
    let time = elapsed.as_secs() as f64 + (f64::from(elapsed.subsec_millis()) * 1e-3);

    let render = surface
      .new_pipeline_gate()
      .pipeline(
        &back_buffer,
        &PipelineState::default(),
        |pipeline, mut shading_gate| {
          let bound_texture = pipeline.bind_texture(&mut tex).unwrap();
          let bound_displacement_1 = pipeline.bind_texture(&mut displacement_tex_1)?;
          let bound_displacement_2 = pipeline.bind_texture(&mut displacement_tex_2)?;

          shading_gate.shade(&mut program, |mut interface, uni, mut render_gate| {
            let back_buffer_size = back_buffer.size();
            interface.set(&uni.image, bound_texture.binding());
            interface.set(&uni.displacement_map_1, bound_displacement_1.binding());
            interface.set(&uni.displacement_map_2, bound_displacement_2.binding());
            interface.set(&uni.displacement_scale, displacement_scale);
            interface.set(&uni.time, time as f32);
            interface.set(
              &uni.window_dimensions,
              [back_buffer_size[0] as f32, back_buffer_size[1] as f32],
            );

            render_gate.render(&render_state, |mut tess_gate| tess_gate.render(&tess))
          })
        },
      )
      .assume();

    if render.is_ok() {
      surface.window.swap_buffers();
    } else {
      break 'app;
    }
  }
}
