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

use luminance::blending::{Equation, Factor};
use luminance::context::GraphicsContext as _;
use luminance::pipeline::BoundTexture;
use luminance::pixel::{NormRGB8UI, NormUnsigned};
use luminance::render_state::RenderState;
use luminance::shader::program::{Program, Uniform};
use luminance::tess::{Mode, TessBuilder};
use luminance::texture::{Dim2, Flat, GenMipmaps, Sampler, Texture};
use luminance_derive::UniformInterface;
use luminance_glfw::{Action, GlfwSurface, Key, Surface, WindowDim, WindowEvent, WindowOpt};
use std::env;
use std::time::Instant;

const VS: &str = include_str!("./12-displacement-map-resources/displacement-map-vs.glsl");
const FS: &str = include_str!("./12-displacement-map-resources/displacement-map-fs.glsl");

#[derive(UniformInterface)]
struct ShaderInterface {
  image: Uniform<&'static BoundTexture<'static, Flat, Dim2, NormUnsigned>>,
  displacement_map_1: Uniform<&'static BoundTexture<'static, Flat, Dim2, NormUnsigned>>,
  displacement_map_2: Uniform<&'static BoundTexture<'static, Flat, Dim2, NormUnsigned>>,
  displacement_scale: Uniform<f32>,
  time: Uniform<f32>,
  window_dimensions: Uniform<[f32; 2]>,
}

fn main() {
  let texture_path = env::args()
    .skip(1)
    .next()
    .expect("Please provide an image path for the core texture");
  let texture_image = image::open(&texture_path)
    .expect("Could not load image from path")
    .flipv()
    .to_rgb();
  let (width, height) = texture_image.dimensions();

  let displacement_map_1 = image::load_from_memory_with_format(
    include_bytes!("./12-displacement-map-resources/displacement_1.png"),
    image::ImageFormat::PNG,
  )
  .expect("Could not load displacement map")
  .to_rgb();

  let displacement_map_2 = image::load_from_memory_with_format(
    include_bytes!("./12-displacement-map-resources/displacement_2.png"),
    image::ImageFormat::PNG,
  )
  .expect("Could not load displacement map")
  .to_rgb();

  let mut surface = GlfwSurface::new(
    WindowDim::Windowed(width, height),
    "texture_demo",
    WindowOpt::default(),
  )
  .expect("Could not create GLFW surface");

  let texels = texture_image.into_raw();
  let tex =
    Texture::<Flat, Dim2, NormRGB8UI>::new(&mut surface, [width, height], 0, Sampler::default())
      .expect("Could not create luminance texture");
  tex.upload_raw(GenMipmaps::No, &texels).unwrap();

  let texels = displacement_map_1.into_raw();
  let displacement_tex_1 =
    Texture::<Flat, Dim2, NormRGB8UI>::new(&mut surface, [128, 128], 0, Sampler::default())
      .expect("Could not create luminance texture");
  displacement_tex_1
    .upload_raw(GenMipmaps::No, &texels)
    .unwrap();

  let texels = displacement_map_2.into_raw();
  let displacement_tex_2 =
    Texture::<Flat, Dim2, NormRGB8UI>::new(&mut surface, [101, 101], 0, Sampler::default())
      .expect("Could not create luminance texture");
  displacement_tex_2
    .upload_raw(GenMipmaps::No, &texels)
    .unwrap();

  let program = Program::<(), (), ShaderInterface>::from_strings(None, VS, None, FS)
    .expect("Could not create shader program")
    .ignore_warnings();

  let tess = TessBuilder::new(&mut surface)
    .set_vertex_nb(4)
    .set_mode(Mode::TriangleFan)
    .build()
    .unwrap();

  let mut back_buffer = surface.back_buffer().unwrap();
  let start_time = Instant::now();
  let render_state =
    RenderState::default().set_blending((Equation::Additive, Factor::SrcAlpha, Factor::Zero));
  let mut resize = false;
  let mut displacement_scale: f32 = 0.010;

  'app: loop {
    for event in surface.poll_events() {
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
        _ => {}
      }
    }

    if resize {
      back_buffer = surface.back_buffer().unwrap();
      resize = false;
    }

    let elapsed = start_time.elapsed();
    let time = elapsed.as_secs() as f64 + (f64::from(elapsed.subsec_millis()) * 1e-3);

    surface.pipeline_builder().pipeline(
      &back_buffer,
      [0.0, 0.0, 0.0, 0.0],
      |pipeline, mut shading_gate| {
        let bound_texture = pipeline.bind_texture(&tex);
        let bound_displacement_1 = pipeline.bind_texture(&displacement_tex_1);
        let bound_displacement_2 = pipeline.bind_texture(&displacement_tex_2);

        shading_gate.shade(&program, |interface, mut render_gate| {
          interface.image.update(&bound_texture);
          interface.displacement_map_1.update(&bound_displacement_1);
          interface.displacement_map_2.update(&bound_displacement_2);
          interface.displacement_scale.update(displacement_scale);
          interface.time.update(time as f32);
          interface
            .window_dimensions
            .update([back_buffer.width() as f32, back_buffer.height() as f32]);

          render_gate.render(render_state, |mut tess_gate| {
            tess_gate.render(&tess);
          })
        });
      },
    );

    surface.swap_buffers();
  }
}
