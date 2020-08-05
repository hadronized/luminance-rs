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

use glfw::{Action, Context as _, Key, WindowEvent};
use luminance::backend::texture::Texture as TextureBackend;
use luminance::blending::{Blending, Equation, Factor};
use luminance::context::GraphicsContext;
use luminance::pipeline::{PipelineState, TextureBinding};
use luminance::pixel::{NormRGB8UI, NormUnsigned};
use luminance::render_state::RenderState;
use luminance::shader::Uniform;
use luminance::tess::Mode;
use luminance::texture::{Dim2, GenMipmaps, Sampler, Texture};
use luminance_derive::UniformInterface;
use luminance_glfw::GlfwSurface;
use luminance_windowing::{WindowDim, WindowOpt};
use std::env; // used to get the CLI arguments
use std::path::Path;

const VS: &str = include_str!("texture-vs.glsl");
const FS: &str = include_str!("texture-fs.glsl");

fn main() {
  if let Some(texture_path) = env::args().nth(1) {
    run(Path::new(&texture_path));
  } else {
    eprintln!("missing first argument (path to the texture to load)");
  }
}

// we also need a special uniform interface here to pass the texture to the shader
#[derive(UniformInterface)]
struct ShaderInterface {
  tex: Uniform<TextureBinding<Dim2, NormUnsigned>>,
}

fn run(texture_path: &Path) {
  let img = read_image(texture_path).expect("error while reading image on disk");
  let (width, height) = img.dimensions();

  let dim = WindowDim::Windowed { width, height };
  let mut surface = GlfwSurface::new_gl33("Hello, world!", WindowOpt::default().set_dim(dim))
    .expect("GLFW surface creation");

  let mut tex = load_from_disk(&mut surface, img);

  // set the uniform interface to our type so that we can read textures from the shader
  let mut program = surface
    .new_shader_program::<(), (), ShaderInterface>()
    .from_strings(VS, None, None, FS)
    .expect("program creation")
    .ignore_warnings();

  // we’ll use an attributeless render here to display a quad on the screen (two triangles); there
  // are over ways to cover the whole screen but this is easier for you to understand; the
  // TriangleFan creates triangles by connecting the third (and next) vertex to the first one
  let tess = surface
    .new_tess()
    .set_vertex_nb(4)
    .set_mode(Mode::TriangleFan)
    .build()
    .unwrap();

  let mut back_buffer = surface.back_buffer().unwrap();
  let render_st = &RenderState::default().set_blending(Blending {
    equation: Equation::Additive,
    src: Factor::SrcAlpha,
    dst: Factor::Zero,
  });
  let mut resize = false;

  println!("rendering!");

  'app: loop {
    surface.window.glfw.poll_events();
    for (_, event) in surface.events_rx.try_iter() {
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
    let render = surface
      .new_pipeline_gate()
      .pipeline(
        &back_buffer,
        &PipelineState::default(),
        |pipeline, mut shd_gate| {
          // bind our fancy texture to the GPU: it gives us a bound texture we can use with the shader
          let bound_tex = pipeline.bind_texture(&mut tex)?;

          shd_gate.shade(&mut program, |mut iface, uni, mut rdr_gate| {
            // update the texture; strictly speaking, this update doesn’t do much: it just tells the GPU
            // to use the texture passed as argument (no allocation or copy is performed)
            iface.set(&uni.tex, bound_tex.binding());

            rdr_gate.render(render_st, |mut tess_gate| {
              // render the tessellation to the surface the regular way and let the vertex shader’s
              // magic do the rest!
              tess_gate.render(&tess)
            })
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

// read the texture into memory as a whole bloc (i.e. no streaming)
fn read_image(path: &Path) -> Option<image::RgbImage> {
  image::open(path).map(|img| img.flipv().to_rgb()).ok()
}

fn load_from_disk<B>(surface: &mut B, img: image::RgbImage) -> Texture<B::Backend, Dim2, NormRGB8UI>
where
  B: GraphicsContext,
  B::Backend: TextureBackend<Dim2, NormRGB8UI>,
{
  let (width, height) = img.dimensions();
  let texels = img.into_raw();

  // create the luminance texture; the third argument is the number of mipmaps we want (leave it
  // to 0 for now) and the latest is the sampler to use when sampling the texels in the
  // shader (we’ll just use the default one)
  let mut tex = Texture::new(surface, [width, height], 0, Sampler::default())
    .expect("luminance texture creation");

  // the first argument disables mipmap generation (we don’t care so far)
  tex.upload_raw(GenMipmaps::No, &texels).unwrap();

  tex
}
