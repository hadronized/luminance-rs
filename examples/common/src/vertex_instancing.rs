//! This program shows you how to do *vertex instancing*, the easy way.
//!
//! <https://docs.rs/luminance>

use luminance::{
  dim::Dim2,
  framebuffer::{Back, Framebuffer},
  primitive::Triangle,
  shader::Program,
  vertex_entity::VertexEntity,
  vertex_storage::Interleaved,
};

use crate::shared::{FragSlot, Instance, Vertex};

const VS: &'static str = include_str!("instancing-vs.glsl");
const FS: &'static str = include_str!("instancing-fs.glsl");

// Only one triangle this time.
const TRI_VERTICES: [Vertex; 3] = [
  // triangle – an RGB one
  //
  Vertex::new(
    mint::Vector2 { x: 0.5, y: -0.5 },
    mint::Vector3 {
      x: 0.,
      y: 1.,
      z: 0.,
    },
  ),
  Vertex::new(
    mint::Vector2 { x: 0., y: 0.5 },
    mint::Vector3 {
      x: 0.,
      y: 0.,
      z: 1.,
    },
  ),
  Vertex::new(
    mint::Vector2 { x: -0.5, y: -0.5 },
    mint::Vector3 {
      x: 1.,
      y: 0.,
      z: 0.,
    },
  ),
];

// Instances. We’ll be using five triangles.
const INSTANCES: [Instance; 5] = [
  Instance {
    position: mint::Vector2 { x: 0., y: 0. },
    weight: 1.,
  },
  Instance {
    position: mint::Vector2 { x: -0.5, y: 0.5 },
    weight: 1.,
  },
  Instance {
    position: mint::Vector2 { x: -0.25, y: -0.1 },
    weight: 1.,
  },
  Instance {
    position: mint::Vector2 { x: 0.45, y: 0.25 },
    weight: 1.,
  },
  Instance {
    position: mint::Vector2 { x: 0.6, y: -0.3 },
    weight: 1.,
  },
];

pub struct LocalExample {
  program: Program<Vertex, Triangle<Vertex>, FragSlot, ()>,
  triangle: VertexEntity<Vertex, Triangle<Vertex>, Interleaved<Vertex>>,
  back_buffer: Framebuffer<Dim2, Back<FragSlot>, Back<()>>,
}

impl Example for LocalExample {
  fn bootstrap(
    _: &mut impl PlatformServices,
    context: &mut impl GraphicsContext<Backend = Backend>,
  ) -> Self {
    // notice that we don’t set a uniform interface here: we’re going to look it up on the fly
    let program = context
      .new_shader_program::<Semantics, (), ()>()
      .from_strings(VS, None, None, FS)
      .expect("program creation")
      .ignore_warnings();

    let triangle = context
      .new_tess()
      .set_vertices(&TRI_VERTICES[..])
      .set_instances(&INSTANCES[..])
      .set_mode(Mode::Triangle)
      .build()
      .unwrap();

    Self { program, triangle }
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

        _ => (),
      }
    }

    // make instances go boop boop by changing their weight dynamically
    {
      let mut instances = self.triangle.instances_mut().expect("instances");

      for (i, instance) in instances.iter_mut().enumerate() {
        let tcos = (t * (i + 1) as f32 * 0.5).cos().powf(2.);
        instance.w = VertexWeight::new(tcos);
      }
    }

    let program = &mut self.program;
    let triangle = &self.triangle;

    let render = context
      .new_pipeline_gate()
      .pipeline(
        &back_buffer,
        &PipelineState::default(),
        |_, mut shd_gate| {
          shd_gate.shade(program, |mut iface, _, mut rdr_gate| {
            if let Ok(ref time_u) = iface.query().unwrap().ask::<f32>("t") {
              iface.set(time_u, t);
            }

            rdr_gate.render(&RenderState::default(), |mut tess_gate| {
              tess_gate.render(triangle)
            })
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
