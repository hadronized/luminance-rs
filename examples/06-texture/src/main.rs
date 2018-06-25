//! This program is a showcase to demonstrate how you can use texture from an image loaded from the
//! disk.
//!
//! The texture path is read from the command line interface and is the sole argument.
//!
//! For the purpose of simplicity, the image is stretched to match your window resolution.
//!
//! Press <escape> to quit or close the window.
//!
//! https://docs.rs/luminance

extern crate image;
#[macro_use]
extern crate luminance;
extern crate luminance_glfw;

use luminance::framebuffer::Framebuffer;
use luminance::pipeline::BoundTexture;
use luminance::pixel::RGB32F;
use luminance::render_state::RenderState;
use luminance::shader::program::Program;
use luminance::tess::{Mode, Tess};
use luminance::texture::{Dim2, Flat, Sampler, Texture};
use luminance_glfw::event::{Action, Key, WindowEvent};
use luminance_glfw::surface::{GlfwSurface, Surface, WindowDim, WindowOpt};
use luminance::context::GraphicsContext;
use std::env; // used to get the CLI arguments
use std::path::Path;

const VS: &'static str = include_str!("vs.glsl");
const FS: &'static str = include_str!("fs.glsl");

fn main() {
  if let Some(texture_path) = env::args().skip(1).next() {
    run(Path::new(&texture_path));
  } else {
    eprintln!("missing first argument (path to the texture to load");
  }
}

// we also need a special uniform interface here to pass the texture to the shader
uniform_interface! {
  struct ShaderInterface {
    // the 'static lifetime acts as “anything” here
    #[unbound]
    tex: &'static BoundTexture<'static, Texture<Flat, Dim2, RGB32F>>
  }
}

fn run(texture_path: &Path) {
  let mut surface = GlfwSurface::new(WindowDim::Windowed(960, 540), "Hello, world!", WindowOpt::default()).expect("GLFW surface creation");

  let tex = load_from_disk(&mut surface, Path::new(&texture_path)).expect("texture loading");

  // set the uniform interface to our type so that we can read textures from the shader
  let (program, _) = Program::<(), (), ShaderInterface>::from_strings(None, VS, None, FS).expect("program creation");

  // we’ll use an attributeless render here to display a quad on the screen (two triangles); there
  // are over ways to cover the whole screen but this is easier for you to understand; the
  // TriangleFan creates triangles by connecting the third (and next) vertex to the first one
  let tess = Tess::attributeless(&mut surface, Mode::TriangleFan, 4);

  let mut back_buffer = Framebuffer::default(surface.size());

  println!("rendering!");

  'app: loop {
    for event in surface.poll_events() {
      match event {
        WindowEvent::Close | WindowEvent::Key(Key::Escape, _, Action::Release, _) => {
          break 'app
        }

        WindowEvent::FramebufferSize(width, height) => {
          back_buffer = Framebuffer::default([width as u32, height as u32]);
        }

        _ => ()
      }
    }

    // here, we need to bind the pipeline variable; it will enable us to bind the texture to the GPU
    // and use it in the shader
    surface.pipeline_builder().pipeline(&back_buffer, [0., 0., 0., 0.], |pipeline, shd_gate| {
      // bind our fancy texture to the GPU: it gives us a bound texture we can use with the shader
      let bound_tex = pipeline.bind_texture(&tex);

      shd_gate.shade(&program, |rdr_gate, iface| {
        // update the texture; strictly speaking, this update doesn’t do much: it just tells the GPU
        // to use the texture passed as argument (no allocation or copy is performed)
        iface.tex.update(&bound_tex);

        rdr_gate.render(RenderState::default(), |tess_gate| {
          // render the tessellation to the surface the regular way and let the vertex shader’s
          // magic do the rest!
          tess_gate.render(&mut surface, (&tess).into());
        });
      });
    });

    surface.swap_buffers();
  }
}

fn load_from_disk(surface: &mut GlfwSurface, path: &Path) -> Option<Texture<Flat, Dim2, RGB32F>> {
  // load the texture into memory as a whole bloc (i.e. no streaming)
  match image::open(&path) {
    Ok(img) => {
      // convert the image to a RGB colorspace (this allocates a new copy of the image
      let rgb_img = img.flipv().to_rgb();
      let (width, height) = rgb_img.dimensions();
      let texels = rgb_img.pixels().map(|rgb| {
        (rgb[0] as f32 / 255., rgb[1] as f32 / 255., rgb[2] as f32 / 255.)
      }).collect::<Vec<_>>();

      // create the luminance texture; the third argument is the number of mipmaps we want (leave it
      // to 0 for now) and the latest is a the sampler to use when sampling the texels in the 
      // shader (we’ll just use the default one)
      let tex = Texture::new(surface, [width, height], 0, &Sampler::default()).expect("luminance texture creation");

      // the first argument disables mipmap generation (we don’t care so far)
      tex.upload(false, &texels);

      Some(tex)
    }

    Err(e) => {
      eprintln!("cannot open image {}: {}", path.display(), e);
      None
    }
  }
}
