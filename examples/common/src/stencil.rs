//! This example shows how to use the stencil buffer to implement a glowing effect on a triangle.

use crate::{
  shared::{FragSlot, Vertex},
  Example, InputAction, LoopFeedback, PlatformServices,
};
use luminance::{
  backend::{Backend, Error},
  context::Context,
  depth_stencil::{Comparison, StencilOp, StencilTest},
  dim::{Dim2, Size2},
  framebuffer::{Back, Framebuffer},
  pipeline::PipelineState,
  pixel::{Depth32FStencil8, NormUnsigned},
  primitive::{Triangle, TriangleFan},
  render_state::RenderState,
  shader::{Program, ProgramBuilder, Uni},
  texture::{InUseTexture, Mipmaps, TextureSampling},
  vertex_entity::{VertexEntity, VertexEntityBuilder, View},
  vertex_storage::{Interleaved, Interleaving},
  Uniforms,
};

const VS: &str = r#"
in vec2 co;

uniform float scale;

void main() {
  gl_Position = vec4(co * scale, 0., 1.);
}
"#;

const FS: &str = r#"
out vec4 frag;

uniform vec3 color;

void main() {
  frag = vec4(color, 1.);
}
"#;

const COPY_VS: &str = include_str!("copy-vs.glsl");
const COPY_FS: &str = include_str!("copy-fs.glsl");

const VERTICES: [Vertex; 3] = [
  Vertex::new(
    mint::Vector2 { x: -0.5, y: -0.5 },
    mint::Vector3 {
      x: 1.,
      y: 1.,
      z: 1.,
    },
  ),
  Vertex::new(
    mint::Vector2 { x: 0.5, y: -0.5 },
    mint::Vector3 {
      x: 1.,
      y: 1.,
      z: 1.,
    },
  ),
  Vertex::new(
    mint::Vector2 { x: 0., y: 0.5 },
    mint::Vector3 {
      x: 1.,
      y: 1.,
      z: 1.,
    },
  ),
];

#[derive(Debug, Uniforms)]
struct StencilInterface {
  scale: Uni<f32>,
  color: Uni<mint::Vector3<f32>>,
}

#[derive(Uniforms)]
struct ShaderCopyInterface {
  source_texture: Uni<InUseTexture<Dim2, NormUnsigned>>,
}

pub struct LocalExample {
  program: Program<Vertex, (), Triangle, FragSlot, StencilInterface>,
  copy_program: Program<(), (), TriangleFan, FragSlot, ShaderCopyInterface>,
  framebuffer: Framebuffer<Dim2, FragSlot, Depth32FStencil8>,
  triangle: VertexEntity<Vertex, Triangle, Interleaving>,
  attributeless: VertexEntity<(), TriangleFan, ()>,
  back_buffer: Framebuffer<Dim2, Back<FragSlot>, Back<()>>,
}

impl Example for LocalExample {
  type Err = Error;

  const TITLE: &'static str = "Stencil";

  fn bootstrap(
    [width, height]: [u32; 2],
    _platform: &mut impl PlatformServices,
    ctx: &mut Context<impl Backend>,
  ) -> Result<Self, Self::Err> {
    let program = ctx.new_program(
      ProgramBuilder::new()
        .add_vertex_stage(VS)
        .no_primitive_stage()
        .add_shading_stage(FS),
    )?;

    let copy_program = ctx.new_program(
      ProgramBuilder::new()
        .add_vertex_stage(COPY_VS)
        .no_primitive_stage()
        .add_shading_stage(COPY_FS),
    )?;

    let framebuffer = ctx.new_framebuffer(
      Size2::new(width, height),
      Mipmaps::No,
      &TextureSampling::default(),
    )?;

    let triangle = ctx.new_vertex_entity(
      VertexEntityBuilder::new().add_vertices(Interleaved::new().set_vertices(VERTICES)),
    )?;

    let attributeless = ctx.new_vertex_entity(VertexEntityBuilder::new())?;

    let back_buffer = ctx.back_buffer(Size2::new(width, height))?;

    Ok(LocalExample {
      program,
      framebuffer,
      copy_program,
      triangle,
      attributeless,
      back_buffer,
    })
  }

  fn render_frame(
    mut self,
    time: f32,
    actions: impl Iterator<Item = InputAction>,
    ctx: &mut Context<impl Backend>,
  ) -> Result<LoopFeedback<Self>, Self::Err> {
    for action in actions {
      match action {
        InputAction::Quit => return Ok(LoopFeedback::Exit),

        InputAction::Resized { width, height } => {
          let size = Size2::new(width, height);
          self.framebuffer = ctx.new_framebuffer(size, Mipmaps::No, &TextureSampling::default())?;
          self.back_buffer = ctx.back_buffer(size)?;
        }

        _ => (),
      }
    }

    let framebuffer = &self.framebuffer;
    let program = &self.program;
    let copy_program = &self.copy_program;
    let triangle = &self.triangle;
    let attributeless = &self.attributeless;

    ctx.with_framebuffer(framebuffer, &PipelineState::default(), |mut frame| {
      frame.with_program(program, |mut frame| {
        frame.update(|mut update, unis| {
          // first we do a regular render in the framebuffer; we will write stencil bits to 1
          update.set(&unis.scale, &1.)?;
          update.set(
            &unis.color,
            &mint::Vector3 {
              x: 1.,
              y: 1.,
              z: 1.,
            },
          )
        })?;

        // we pass the stencil test if the value is < 1
        let stencil_test = StencilTest::On {
          comparison: Comparison::Less,
          reference: 1,
          mask: 0xFF,
          depth_passes_stencil_fails: StencilOp::Keep,
          depth_fails_stencil_passes: StencilOp::Keep,
          depth_stencil_pass: StencilOp::Replace,
        };
        frame.with_render_state(
          &RenderState::default().set_stencil_test(stencil_test),
          |mut frame| frame.render_vertex_entity(triangle.view(..)),
        )?;

        // then, render again but slightly upscaled
        frame.update(|mut update, unis| {
          update.set(&unis.scale, &(1. + (time * 3.).cos().abs() * 0.1))?;
          update.set(
            &unis.color,
            &mint::Vector3 {
              x: 0.,
              y: 1.,
              z: 0.,
            },
          )
        })?;

        // we pass the stencil test if the value is == 0
        let stencil_test = StencilTest::On {
          comparison: Comparison::Equal,
          reference: 0,
          mask: 0xFF,
          depth_passes_stencil_fails: StencilOp::Keep,
          depth_fails_stencil_passes: StencilOp::Keep,
          depth_stencil_pass: StencilOp::Replace,
        };

        frame.with_render_state(
          &RenderState::default().set_stencil_test(stencil_test),
          |mut frame| frame.render_vertex_entity(triangle.view(..)),
        )
      })
    })?;

    // copy the result the back buffer
    ctx.with_framebuffer(&self.back_buffer, &PipelineState::default(), |mut frame| {
      let source = frame.use_texture(&framebuffer.layers().frag)?;

      frame.with_program(copy_program, |mut frame| {
        frame.update(|mut update, unis| update.set(&unis.source_texture, &source))?;
        frame.with_render_state(&RenderState::default(), |mut frame| {
          frame.render_vertex_entity(attributeless.view(..4))
        })
      })
    })?;

    Ok(LoopFeedback::Continue(self))
  }
}
