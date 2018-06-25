//! This program shows how to render a single triangle into an offscreen framebuffer and how to
//! render the content of this offscreen framebuffer into the back buffer (i.e. the screen).
//!
//! Press <escape> to quit or close the window.
//!
//! https://docs.rs/luminance

#[macro_use]
extern crate luminance;
extern crate luminance_glfw;

use luminance::framebuffer::Framebuffer;
use luminance::pipeline::BoundTexture;
use luminance::pixel::RGBA32F;
use luminance::shader::program::Program;
use luminance::tess::{Mode, Tess, TessSliceIndex};
use luminance::texture::{Dim2, Flat, Texture};
use luminance::render_state::RenderState;
use luminance_glfw::event::{Action, Key, WindowEvent};
use luminance_glfw::surface::{GlfwSurface, Surface, WindowDim, WindowOpt};
use luminance::context::GraphicsContext;

// we get the shader at compile time from ./vs.glsl and ./fs.glsl
const VS: &'static str = include_str!("vs.glsl");
const FS: &'static str = include_str!("fs.glsl");

// copy shader, at compile time as well
const COPY_VS: &'static str = include_str!("copy_vs.glsl");
const COPY_FS: &'static str = include_str!("copy_fs.glsl");

type Vertex = ([f32; 2], [f32; 3]);

// a single triangle is enough here
const TRI_VERTICES: [Vertex; 3] = [
  // triangle – an RGB one
  ([ 0.5, -0.5], [0., 1., 0.]),
  ([ 0.0,  0.5], [0., 0., 1.]),
  ([-0.5, -0.5], [1., 0., 0.]),
];

// the shader uniform interface is defined there
uniform_interface! {
  struct ShaderInterface {
    // we only need the source texture (from the framebuffer) to fetch from
    #[unbound, as("source_texture")]
    texture: &'static BoundTexture<'static, Texture<Flat, Dim2, RGBA32F>>
  }
}

fn main() {
  let mut surface = GlfwSurface::new(WindowDim::Windowed(960, 540), "Hello, world!", WindowOpt::default()).expect("GLFW surface creation");

  let (program, _) = Program::<Vertex, (), ()>::from_strings(None, VS, None, FS).expect("program creation");
  let (copy_program, warnings) = Program::<(), (), ShaderInterface>::from_strings(None, COPY_VS, None, COPY_FS).expect("copy program creation");

  for warning in &warnings {
    eprintln!("copy shader warning: {:?}", warning);
  }

  let triangle = Tess::new(&mut surface, Mode::Triangle, &TRI_VERTICES[..], None);
  // we’ll need an attributeless quad to fetch in full screen
  let quad = Tess::attributeless(&mut surface, Mode::TriangleFan, 4);

  let surf_size = surface.size();
  // “screen“ we want to render into our offscreen render
  let mut back_buffer = Framebuffer::default(surf_size); 
  // offscreen buffer that we will render in the first place
  let mut offscreen_buffer =
    Framebuffer::<Flat, Dim2, Texture<Flat, Dim2, RGBA32F>, ()>::new(&mut surface, surf_size, 0).expect("framebuffer creation");

  // hack to update the offscreen buffer if needed; this is needed because we cannot update the
  // offscreen buffer from within the event loop
  let mut update_offscreen_buffer = None;

  'app: loop {
    // for all the events on the surface
    for event in surface.poll_events() {
      match event {
        WindowEvent::Close | WindowEvent::Key(Key::Escape, _, Action::Release, _) => {
          break 'app
        }

        WindowEvent::FramebufferSize(width, height) => {
          update_offscreen_buffer = Some([width as u32, height as u32]);
        }

        _ => ()
      }
    }

    // if the window got resized
    if let Some(size) = update_offscreen_buffer {
      // simply ask another backbuffer at the right dimension (no allocation / reallocation)
      back_buffer = Framebuffer::default(size);
      // ditto for the offscreen framebuffer
      offscreen_buffer = Framebuffer::new(&mut surface, size, 0).expect("framebuffer recreation");

      update_offscreen_buffer = None;
    }

    // we get an object to create pipelines (we’ll need two)
    let builder = surface.pipeline_builder();

    // render the triangle in the offscreen framebuffer first
    builder.pipeline(&offscreen_buffer, [0., 0., 0., 0.], |_, shd_gate| {
      shd_gate.shade(&program, |rdr_gate, _| {
        rdr_gate.render(RenderState::default(), |tess_gate| {
          // we render the triangle here by asking for the whole triangle
          tess_gate.render(&mut surface, triangle.slice(..));
        });
      });
    });

    // read from the offscreen framebuffer and output it into the back buffer
    builder.pipeline(&back_buffer, [0., 0., 0., 0.], |pipeline, shd_gate| {
      // we must bind the offscreen framebuffer color content so that we can pass it to a shader
      let bound_texture = pipeline.bind_texture(offscreen_buffer.color_slot());

      shd_gate.shade(&copy_program, |rdr_gate, iface| {
        // we update the texture with the bound texture
        iface.texture.update(&bound_texture);

        rdr_gate.render(RenderState::default(), |tess_gate| {
          // this will render the attributeless quad with the offscreen framebuffer color slot
          // bound for the shader to fetch from
          tess_gate.render(&mut surface, quad.slice(..));
        });
      });
    });

    // finally, swap the backbuffer with the frontbuffer in order to render our triangles onto your
    // screen
    surface.swap_buffers();
  }
}
