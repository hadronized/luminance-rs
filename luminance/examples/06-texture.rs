//! This program is a showcase to demonstrate how you can use texture from an image loaded from the
//! disk.
//!
//! The texture path is read from the command line interface and is the sole argument.
//!
//! For the purpose of simplicity, the image is stretched to match your window resolution.
//!
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
use luminance_glfw::{Action, GlfwSurface, Key, Surface, WindowEvent, WindowDim, WindowOpt};
use std::env; // used to get the CLI arguments
use std::path::Path;

const VS: &'static str = include_str!("texture-vs.glsl");
const FS: &'static str = include_str!("texture-fs.glsl");

fn main() {
  if let Some(texture_path) = env::args().skip(1).next() {
    run(Path::new(&texture_path));
  } else {
    eprintln!("missing first argument (path to the texture to load)");
  }
}

// we also need a special uniform interface here to pass the texture to the shader
#[derive(UniformInterface)]
struct ShaderInterface {
  // the 'static lifetime acts as “anything” here
  tex: Uniform<&'static BoundTexture<'static, Flat, Dim2, NormUnsigned>>
}

fn run(texture_path: &Path) {
  let img = read_image(texture_path).expect("error while reading image on disk");
  let (width, height) = img.dimensions();

  let mut surface = GlfwSurface::new(
    WindowDim::Windowed(width, height),
    "Hello, world!",
    WindowOpt::default(),
  )
  .expect("GLFW surface creation");

  let tex = load_from_disk(&mut surface, img);

  // set the uniform interface to our type so that we can read textures from the shader
  let (program, _) =
    Program::<(), (), ShaderInterface>::from_strings(None, VS, None, FS).expect("program creation");

  // we’ll use an attributeless render here to display a quad on the screen (two triangles); there
  // are over ways to cover the whole screen but this is easier for you to understand; the
  // TriangleFan creates triangles by connecting the third (and next) vertex to the first one
  let tess = TessBuilder::new(&mut surface)
    .set_vertex_nb(4)
    .set_mode(Mode::TriangleFan)
    .build()
    .unwrap();

  let mut back_buffer = surface.back_buffer().unwrap();
  let render_st = RenderState::default().set_blending((Equation::Additive, Factor::SrcAlpha, Factor::Zero));
  let mut resize = false;

  println!("rendering!");

  'app: loop {
    for event in surface.poll_events() {
      match event {
        WindowEvent::Close | WindowEvent::Key(Key::Escape, _, Action::Release, _) => break 'app,

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

    // here, we need to bind the pipeline variable; it will enable us to bind the texture to the GPU
    // and use it in the shader
    surface
      .pipeline_builder()
      .pipeline(&back_buffer, [0., 0., 0., 0.], |pipeline, mut shd_gate| {
        // bind our fancy texture to the GPU: it gives us a bound texture we can use with the shader
        let bound_tex = pipeline.bind_texture(&tex);

        shd_gate.shade(&program, |iface, mut rdr_gate| {
          // update the texture; strictly speaking, this update doesn’t do much: it just tells the GPU
          // to use the texture passed as argument (no allocation or copy is performed)
          iface.tex.update(&bound_tex);

          rdr_gate.render(render_st, |mut tess_gate| {
            // render the tessellation to the surface the regular way and let the vertex shader’s
            // magic do the rest!
            tess_gate.render(&tess);
          });
        });
      });

    surface.swap_buffers();
  }
}

// read the texture into memory as a whole bloc (i.e. no streaming)
fn read_image(path: &Path) -> Option<image::RgbImage> {
  image::open(path).map(|img| img.flipv().to_rgb()).ok()
}

fn load_from_disk(surface: &mut GlfwSurface, img: image::RgbImage) -> Texture<Flat, Dim2, NormRGB8UI> {
  let (width, height) = img.dimensions();
  let texels = img.into_raw();

  // create the luminance texture; the third argument is the number of mipmaps we want (leave it
  // to 0 for now) and the latest is the sampler to use when sampling the texels in the
  // shader (we’ll just use the default one)
  let tex = Texture::new(surface, [width, height], 0, &Sampler::default())
    .expect("luminance texture creation");

  // the first argument disables mipmap generation (we don’t care so far)
  tex.upload_raw(GenMipmaps::No, &texels).unwrap();

  tex
}
