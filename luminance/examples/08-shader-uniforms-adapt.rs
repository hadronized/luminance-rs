//! > This program is a sequel to 04-shader-uniforms. Be sure to have read it first.
//!
//! This example shows you how to change the type of a shader program’s interface on the fly without
//! changing the GPU object. This might be wanted whenever you need to use a different type which
//! fields overlap the former type you used, or to implement a form of dynamic introspection. By
//! readapting the uniform interface (to the same type), you can use a *value-driven* approach to
//! add new uniforms on the fly, which comes in very handy when writing UI systems for instance.
//!
//! The program should start black so press space and enjoy.
//!
//! Press the <a>, <s>, <d>, <z> or the arrow keys to move the triangle on the screen.
//! Press the <space> key to switch between uniform interfaces.
//! Press <escape> to quit or close the window.
//!
//! https://docs.rs/luminance

mod common;

use crate::common::{Semantics, Vertex, VertexPosition, VertexColor};
use luminance::context::GraphicsContext as _;
use luminance::render_state::RenderState;
use luminance::shader::program::{Program, Uniform};
use luminance::tess::{Mode, TessBuilder};
use luminance_derive::UniformInterface;
use luminance_glfw::{Action, GlfwSurface, Key, Surface, WindowEvent, WindowDim, WindowOpt};
use std::time::Instant;

const VS: &'static str = include_str!("adapt-vs.glsl");
const FS: &'static str = include_str!("displacement-fs.glsl");

// Only one triangle this time.
const TRI_VERTICES: [Vertex; 3] = [
  Vertex { pos: VertexPosition::new([0.5, -0.5]), rgb: VertexColor::new([1., 0., 0.]) },
  Vertex { pos: VertexPosition::new([0.0, 0.5]), rgb: VertexColor::new([0., 1., 0.]) },
  Vertex { pos: VertexPosition::new([-0.5, -0.5]), rgb: VertexColor::new([0., 0., 1.]) },
];

/// First uniform interface.
#[derive(Debug, UniformInterface)]
struct ShaderInterface1 {
  #[uniform(name = "t")]
  time: Uniform<f32>,
  triangle_size: Uniform<f32>
}

/// Second uniform interface.
#[derive(Debug, UniformInterface)]
struct ShaderInterface2 {
  #[uniform(name = "t")]
  time: Uniform<f32>,
  triangle_pos: Uniform<[f32; 2]>
}

// Which interface to use?
enum ProgramMode {
  First(Program<Semantics, (), ShaderInterface1>),
  Second(Program<Semantics, (), ShaderInterface2>),
}

impl ProgramMode {
  fn toggle(self) -> Self {
    match self {
      ProgramMode::First(p) => match p.adapt() {
        Ok((x, _)) => ProgramMode::Second(x),
        Err((e, y)) => {
          eprintln!("unable to switch to second uniform interface: {:?}", e);
          ProgramMode::First(y)
        }
      },

      ProgramMode::Second(p) => match p.adapt() {
        Ok((x, _)) => ProgramMode::First(x),
        Err((e, y)) => {
          eprintln!("unable to switch to first uniform interface: {:?}", e);
          ProgramMode::Second(y)
        }
      },
    }
  }
}

fn main() {
  let mut surface = GlfwSurface::new(
    WindowDim::Windowed(960, 540),
    "Hello, world!",
    WindowOpt::default(),
  )
  .expect("GLFW surface creation");

  let mut program = ProgramMode::First(
    Program::<Semantics, (), ShaderInterface1>::from_strings(None, VS, None, FS)
      .expect("program creation")
      .0,
  );

  let triangle = TessBuilder::new(&mut surface)
    .add_vertices(TRI_VERTICES)
    .set_mode(Mode::Triangle)
    .build()
    .unwrap();

  let mut back_buffer = surface.back_buffer().unwrap();
  let mut triangle_pos = [0., 0.];
  let start_t = Instant::now();
  let mut resize = false;

  'app: loop {
    for event in surface.poll_events() {
      match event {
        WindowEvent::Close | WindowEvent::Key(Key::Escape, _, Action::Release, _) => break 'app,

        WindowEvent::Key(Key::Space, _, Action::Release, _) => {
          program = program.toggle();
        }

        WindowEvent::Key(Key::A, _, action, _) | WindowEvent::Key(Key::Left, _, action, _)
          if action == Action::Press || action == Action::Repeat =>
        {
          triangle_pos[0] -= 0.1;
        }

        WindowEvent::Key(Key::D, _, action, _) | WindowEvent::Key(Key::Right, _, action, _)
          if action == Action::Press || action == Action::Repeat =>
        {
          triangle_pos[0] += 0.1;
        }

        WindowEvent::Key(Key::Z, _, action, _) | WindowEvent::Key(Key::Up, _, action, _)
          if action == Action::Press || action == Action::Repeat =>
        {
          triangle_pos[1] += 0.1;
        }

        WindowEvent::Key(Key::S, _, action, _) | WindowEvent::Key(Key::Down, _, action, _)
          if action == Action::Press || action == Action::Repeat =>
        {
          triangle_pos[1] -= 0.1;
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

    let elapsed = start_t.elapsed();
    let t64 = elapsed.as_secs() as f64 + (elapsed.subsec_millis() as f64 * 1e-3);
    let t = t64 as f32;

    surface
      .pipeline_builder()
      .pipeline(&back_buffer, [0., 0., 0., 0.], |_, shd_gate| {
        match program {
          // if we use the first interface, we just need to pass the time and the triangle position
          ProgramMode::First(ref program) => {
            shd_gate.shade(&program, |iface, rdr_gate| {
              iface.time.update(t);
              iface.triangle_size.update(t.cos().powf(2.));

              rdr_gate.render(RenderState::default(), |tess_gate| {
                tess_gate.render(&mut surface, &triangle);
              });
            });
          }

          // if we use the second interface, we just need to pass the time and we will make the size
          // grow by using the time
          ProgramMode::Second(ref program) => {
            shd_gate.shade(&program, |iface, rdr_gate| {
              iface.time.update(t);
              // iface.triangle_pos.update(triangle_pos); // uncomment this to see a nice error ;)
              iface.triangle_pos.update(triangle_pos);

              rdr_gate.render(RenderState::default(), |tess_gate| {
                tess_gate.render(&mut surface, &triangle);
              });
            });
          }
        }
      });

    surface.swap_buffers();
  }
}
