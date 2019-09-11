//! This program shows how to render a single triangle into an offscreen framebuffer and how to
//! render the content of this offscreen framebuffer into the back buffer (i.e. the screen).
//!
//! Press <escape> to quit or close the window.
//!
//! https://docs.rs/luminance

mod common;

use crate::common::{Semantics, Vertex, VertexPosition, VertexColor};
use luminance::context::GraphicsContext as _;
use luminance::framebuffer::Framebuffer;
use luminance::pipeline::BoundTexture;
use luminance::pixel::{RGBA32F, Floating};
use luminance::render_state::RenderState;
use luminance::shader::program::{Program, Uniform};
use luminance::tess::{Mode, TessBuilder, TessSliceIndex};
use luminance::texture::{Dim2, Flat};
use luminance_derive::UniformInterface;
use luminance_glfw::{Action, GlfwSurface, Key, Surface, WindowEvent, WindowDim, WindowOpt};

// we get the shader at compile time from local files
const VS: &'static str = include_str!("simple-vs.glsl");
const FS: &'static str = include_str!("simple-fs.glsl");

// copy shader, at compile time as well
const COPY_VS: &'static str = include_str!("copy-vs.glsl");
const COPY_FS: &'static str = include_str!("copy-fs.glsl");

// a single triangle is enough here
const TRI_VERTICES: [Vertex; 3] = [
  // triangle – an RGB one
  Vertex { pos: VertexPosition::new([0.5, -0.5]), rgb: VertexColor::new([0., 1., 0.]) },
  Vertex { pos: VertexPosition::new([0.0, 0.5]), rgb: VertexColor::new([0., 0., 1.]) },
  Vertex { pos: VertexPosition::new([-0.5, -0.5]), rgb: VertexColor::new([1., 0., 0.]) },
];

// the shader uniform interface is defined there
#[derive(UniformInterface)]
struct ShaderInterface {
  // we only need the source texture (from the framebuffer) to fetch from
  #[uniform(unbound, name = "source_texture")]
  texture: Uniform<&'static BoundTexture<'static, Flat, Dim2, Floating>>
}

fn main() {
  let mut surface = GlfwSurface::new(
    WindowDim::Windowed(960, 540),
    "Hello, world!",
    WindowOpt::default(),
  )
  .expect("GLFW surface creation");

  let (program, _) = Program::<Semantics, (), ()>::from_strings(None, VS, None, FS).expect("program creation");
  let (copy_program, warnings) =
    Program::<(), (), ShaderInterface>::from_strings(None, COPY_VS, None, COPY_FS)
      .expect("copy program creation");

  for warning in &warnings {
    eprintln!("copy shader warning: {:?}", warning);
  }

  let triangle = TessBuilder::new(&mut surface)
    .add_vertices(TRI_VERTICES)
    .set_mode(Mode::Triangle)
    .build()
    .unwrap();

  // we’ll need an attributeless quad to fetch in full screen
  let quad = TessBuilder::new(&mut surface)
    .set_vertex_nb(4)
    .set_mode(Mode::TriangleFan)
    .build()
    .unwrap();

  // “screen“ we want to render into our offscreen render
  let mut back_buffer = surface.back_buffer().unwrap();
  // offscreen buffer that we will render in the first place
  let size = surface.size();
  let mut offscreen_buffer =
    Framebuffer::<Flat, Dim2, RGBA32F, ()>::new(&mut surface, size, 0).expect("framebuffer creation");

  // hack to update the offscreen buffer if needed; this is needed because we cannot update the
  // offscreen buffer from within the event loop
  let mut resize = false;

  'app: loop {
    // for all the events on the surface
    for event in surface.poll_events() {
      match event {
        WindowEvent::Close | WindowEvent::Key(Key::Escape, _, Action::Release, _) => break 'app,

        WindowEvent::FramebufferSize(..) => {
          resize = true;
        }

        _ => (),
      }
    }

    // if the window got resized
    if resize {
      // simply ask another backbuffer at the right dimension (no allocation / reallocation)
      back_buffer = surface.back_buffer().unwrap();
      // ditto for the offscreen framebuffer
      let size = surface.size();
      offscreen_buffer = Framebuffer::new(&mut surface, size, 0).expect("framebuffer recreation");

      resize = false;
    }

    // we get an object to create pipelines (we’ll need two)
    let builder = surface.pipeline_builder();

    // render the triangle in the offscreen framebuffer first
    builder.pipeline(&offscreen_buffer, [0., 0., 0., 0.], |_, shd_gate| {
      shd_gate.shade(&program, |_, rdr_gate| {
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

      shd_gate.shade(&copy_program, |iface, rdr_gate| {
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
