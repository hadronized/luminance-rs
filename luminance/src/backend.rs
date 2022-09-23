use crate::{
  dim::Dimensionable,
  framebuffer::Framebuffer,
  pipeline::{PipelineState, WithFramebuffer, WithProgram, WithRenderState},
  primitive::Primitive,
  render_channel::{IsDepthChannelType, IsRenderChannelType},
  render_slots::{DepthRenderSlot, RenderLayer, RenderSlots},
  render_state::RenderState,
  shader::{Env, FromEnv, IsEnv, IsSharedEnv, Program, SharedEnv},
  vertex::Vertex,
  vertex_entity::{Indices, VertexEntity, VertexEntityView, Vertices},
  vertex_storage::VertexStorage,
};
use std::error::Error as ErrorTrait;

#[derive(Debug)]
pub enum VertexEntityError {
  Creation { cause: Option<Box<dyn ErrorTrait>> },

  Render { cause: Option<Box<dyn ErrorTrait>> },

  RetrieveVertexStorage { cause: Option<Box<dyn ErrorTrait>> },

  UpdateVertexStorage { cause: Option<Box<dyn ErrorTrait>> },

  RetrieveIndices { cause: Option<Box<dyn ErrorTrait>> },

  UpdateIndices { cause: Option<Box<dyn ErrorTrait>> },
}

#[derive(Debug)]
pub enum FramebufferError {
  Creation { cause: Option<Box<dyn ErrorTrait>> },

  RenderLayerCreation { cause: Option<Box<dyn ErrorTrait>> },

  DepthRenderLayerCreation { cause: Option<Box<dyn ErrorTrait>> },

  RetrieveBackBuffer { cause: Option<Box<dyn ErrorTrait>> },
}

#[derive(Debug)]
pub enum ShaderError {
  Creation {
    vertex_code: String,
    primitive_code: String,
    shading_code: String,
    cause: Option<Box<dyn ErrorTrait>>,
  },

  UniCreation {
    name: String,
    cause: Option<Box<dyn ErrorTrait>>,
  },

  UniSet {
    cause: Option<Box<dyn ErrorTrait>>,
  },

  UniBufferCreation {
    name: String,
    cause: Option<Box<dyn ErrorTrait>>,
  },
}

#[derive(Debug)]
pub enum PipelineError {
  WithFramebuffer {
    pipeline_state: PipelineState,
    cause: Option<Box<dyn ErrorTrait>>,
  },

  WithProgram {
    cause: Option<Box<dyn ErrorTrait>>,
  },

  WithRenderState {
    render_state: RenderState,
    cause: Option<Box<dyn ErrorTrait>>,
  },

  RenderVertexEntity {
    start_vertex: usize,
    vertex_count: usize,
    instance_count: usize,
    cause: Option<Box<dyn ErrorTrait>>,
  },
}

#[derive(Debug)]
pub enum Error {
  VertexEntity(VertexEntityError),
  Framebuffer(FramebufferError),
  Shader(ShaderError),
  Pipeline(PipelineError),
}

impl From<VertexEntityError> for Error {
  fn from(e: VertexEntityError) -> Self {
    Error::VertexEntity(e)
  }
}

impl From<FramebufferError> for Error {
  fn from(e: FramebufferError) -> Self {
    Error::Framebuffer(e)
  }
}

impl From<ShaderError> for Error {
  fn from(e: ShaderError) -> Self {
    Error::Shader(e)
  }
}

impl From<PipelineError> for Error {
  fn from(e: PipelineError) -> Self {
    Error::Pipeline(e)
  }
}

pub trait Backend:
  VertexEntityBackend + FramebufferBackend + ShaderBackend + PipelineBackend
{
}

impl<B> Backend for B where
  B: VertexEntityBackend + FramebufferBackend + ShaderBackend + PipelineBackend
{
}

pub unsafe trait VertexEntityBackend {
  unsafe fn new_vertex_entity<V, S, I>(
    &mut self,
    storage: S,
    indices: I,
  ) -> Result<VertexEntity<V, S>, VertexEntityError>
  where
    V: Vertex,
    S: VertexStorage<V>,
    I: Into<Vec<u32>>;

  unsafe fn vertex_entity_render<V, S>(
    &self,
    entity: &VertexEntity<V, S>,
    start_index: usize,
    vert_count: usize,
    inst_count: usize,
  ) -> Result<(), VertexEntityError>
  where
    V: Vertex,
    S: VertexStorage<V>;

  unsafe fn vertex_entity_vertices<'a, V, S>(
    &'a mut self,
    entity: &VertexEntity<V, S>,
  ) -> Result<Vertices<'a, V, S>, VertexEntityError>
  where
    V: Vertex,
    S: VertexStorage<V>;

  unsafe fn vertex_entity_update_vertices<'a, V, S>(
    &'a mut self,
    entity: &VertexEntity<V, S>,
    vertices: Vertices<'a, V, S>,
  ) -> Result<(), VertexEntityError>
  where
    V: Vertex,
    S: VertexStorage<V>;

  unsafe fn vertex_entity_indices<'a, V, S>(
    &'a mut self,
    entity: &VertexEntity<V, S>,
  ) -> Result<Indices<'a>, VertexEntityError>
  where
    V: Vertex,
    S: VertexStorage<V>;

  unsafe fn vertex_entity_update_indices<'a, V, S>(
    &'a mut self,
    entity: &VertexEntity<V, S>,
    indices: Indices<'a>,
  ) -> Result<(), VertexEntityError>
  where
    V: Vertex,
    S: VertexStorage<V>;
}

pub unsafe trait FramebufferBackend {
  unsafe fn new_render_layer<D, RC>(
    &mut self,
    size: D::Size,
  ) -> Result<RenderLayer<RC>, FramebufferError>
  where
    D: Dimensionable,
    RC: IsRenderChannelType;

  unsafe fn new_depth_render_layer<D, DC>(
    &mut self,
    size: D::Size,
  ) -> Result<RenderLayer<DC>, FramebufferError>
  where
    D: Dimensionable,
    DC: IsDepthChannelType;

  unsafe fn new_framebuffer<D, RS, DS>(
    &mut self,
    size: D::Size,
  ) -> Result<Framebuffer<D, RS, DS>, FramebufferError>
  where
    D: Dimensionable,
    RS: RenderSlots,
    DS: DepthRenderSlot;

  unsafe fn back_buffer<D, RS, DS>(
    &mut self,
    size: D::Size,
  ) -> Result<Framebuffer<D, RS, DS>, FramebufferError>
  where
    D: Dimensionable,
    RS: RenderSlots,
    DS: DepthRenderSlot;
}

pub unsafe trait ShaderBackend {
  unsafe fn new_program<V, P, S, E>(
    &mut self,
    vertex_code: String,
    primitive_code: String,
    shading_code: String,
  ) -> Result<Program<V, P, S, E>, ShaderError>
  where
    V: Vertex,
    P: Primitive,
    S: RenderSlots,
    E: FromEnv;

  unsafe fn new_shader_env<T>(
    &mut self,
    program_handle: usize,
    name: &str,
  ) -> Result<Env<T>, ShaderError>
  where
    T: IsEnv;

  unsafe fn set_program_env<T>(
    &mut self,
    program_handle: usize,
    env_handle: usize,
    value: T,
  ) -> Result<(), ShaderError>
  where
    T: IsEnv;

  unsafe fn new_shader_shared_env<T>(
    &mut self,
    program_handle: usize,
    name: &str,
  ) -> Result<SharedEnv<T>, ShaderError>
  where
    T: IsSharedEnv;
}

pub unsafe trait PipelineBackend {
  unsafe fn with_framebuffer<'a, D, CS, DS, Err>(
    &mut self,
    framebuffer: &Framebuffer<D, CS, DS>,
    state: &PipelineState,
    f: impl FnOnce(WithFramebuffer<'a, Self, CS>) -> Result<(), Err>,
  ) -> Result<(), Err>
  where
    Self: 'a,
    D: Dimensionable,
    CS: RenderSlots,
    DS: DepthRenderSlot,
    Err: From<PipelineError>;

  unsafe fn with_program<'a, V, P, S, E, Err>(
    &mut self,
    program: &Program<V, P, S, E>,
    f: impl FnOnce(WithProgram<'a, Self, V, P, S, E>) -> Result<(), Err>,
  ) -> Result<(), Err>
  where
    Self: 'a,
    V: Vertex,
    P: Primitive,
    S: RenderSlots,
    E: FromEnv,
    Err: From<PipelineError>;

  unsafe fn with_render_state<'a, V, Err>(
    &mut self,
    render_state: &RenderState,
    f: impl FnOnce(WithRenderState<'a, Self, V>) -> Result<(), Err>,
  ) -> Result<(), Err>
  where
    Self: 'a,
    V: Vertex,
    Err: From<PipelineError>;

  unsafe fn render_vertex_entity<V>(
    &mut self,
    view: VertexEntityView<V>,
  ) -> Result<(), PipelineError>
  where
    V: Vertex;
}
