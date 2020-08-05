use glfw::{Action, Context as _, Key, WindowEvent};
use luminance::{pipeline::PipelineState, render_state::RenderState};
use luminance_derive::UniformInterface;
use luminance_front::context::GraphicsContext;
use luminance_front::shader::Uniform;
use luminance_front::tess::Mode;
use luminance_glfw::GlfwSurface;
use luminance_windowing::WindowOpt;
use std::time::Instant;

const VS: &str = "
const vec2[4] POSITIONS = vec2[](
  vec2(-1., -1.),
  vec2( 1., -1.),
  vec2( 1.,  1.),
  vec2(-1.,  1.)
);

void main() {
  gl_Position = vec4(POSITIONS[gl_VertexID], 0., 1.);
}";

const FS: &str = "
out vec3 frag;

uniform dvec3 color;

void main() {
  frag = vec3(color);
}";

#[derive(Debug, UniformInterface)]
struct ShaderInterface {
  color: Uniform<[f64; 3]>,
}

fn main() {
  let mut surface =
    GlfwSurface::new_gl33("GL_ARB_gpu_shader_fp64 test", WindowOpt::default()).unwrap();

  let mut program = surface
    .new_shader_program::<(), (), ShaderInterface>()
    .from_strings(VS, None, None, FS)
    .unwrap()
    .ignore_warnings();

  let tess = surface
    .new_tess()
    .set_mode(Mode::TriangleFan)
    .set_vertex_nb(4)
    .build()
    .unwrap();

  let timer = Instant::now();

  'app: loop {
    surface.window.glfw.poll_events();
    for (_, event) in surface.events_rx.try_iter() {
      match event {
        WindowEvent::Close | WindowEvent::Key(Key::Escape, _, Action::Release, _) => break 'app,
        _ => (),
      }
    }

    let back_buffer = surface.back_buffer().unwrap();
    let t = timer.elapsed().as_secs_f64();
    let color = [t.cos(), 0.3, t.sin()];

    let render = surface
      .new_pipeline_gate()
      .pipeline(
        &back_buffer,
        &PipelineState::default(),
        |_, mut shd_gate| {
          shd_gate.shade(&mut program, |mut iface, uni, mut rdr_gate| {
            iface.set(&uni.color, color);

            rdr_gate.render(&RenderState::default(), |mut tess_gate| {
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
