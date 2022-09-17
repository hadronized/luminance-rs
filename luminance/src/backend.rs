#![allow(missing_docs)]

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

pub trait Backend:
  BackendErr + VertexEntityBackend + FramebufferBackend + ShaderBackend + PipelineBackend
{
}

impl<B> Backend for B where
  B: BackendErr + VertexEntityBackend + FramebufferBackend + ShaderBackend + PipelineBackend
{
}

pub unsafe trait BackendErr {
  type Err;
}

pub unsafe trait VertexEntityBackend: BackendErr {
  unsafe fn new_vertex_entity<V, S, I>(
    &mut self,
    storage: S,
    indices: I,
  ) -> Result<VertexEntity<V, S>, Self::Err>
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
  ) -> Result<(), Self::Err>
  where
    V: Vertex,
    S: VertexStorage<V>;

  unsafe fn vertex_entity_vertices<'a, V, S>(
    &'a mut self,
    entity: &VertexEntity<V, S>,
  ) -> Result<Vertices<'a, V, S>, Self::Err>
  where
    V: Vertex,
    S: VertexStorage<V>;

  unsafe fn vertex_entity_update_vertices<'a, V, S>(
    &'a mut self,
    entity: &VertexEntity<V, S>,
    vertices: Vertices<'a, V, S>,
  ) -> Result<(), Self::Err>
  where
    V: Vertex,
    S: VertexStorage<V>;

  unsafe fn vertex_entity_indices<'a, V, S>(
    &'a mut self,
    entity: &VertexEntity<V, S>,
  ) -> Result<Indices<'a>, Self::Err>
  where
    V: Vertex,
    S: VertexStorage<V>;

  unsafe fn vertex_entity_update_indices<'a, V, S>(
    &'a mut self,
    entity: &VertexEntity<V, S>,
    indices: Indices<'a>,
  ) -> Result<(), Self::Err>
  where
    V: Vertex,
    S: VertexStorage<V>;
}

pub unsafe trait FramebufferBackend: BackendErr {
  unsafe fn new_render_layer<D, RC>(&mut self, size: D::Size) -> Result<RenderLayer<RC>, Self::Err>
  where
    D: Dimensionable,
    RC: IsRenderChannelType;

  unsafe fn new_depth_render_layer<D, DC>(
    &mut self,
    size: D::Size,
  ) -> Result<RenderLayer<DC>, Self::Err>
  where
    D: Dimensionable,
    DC: IsDepthChannelType;

  unsafe fn new_framebuffer<D, RS, DS>(
    &mut self,
    size: D::Size,
  ) -> Result<Framebuffer<D, RS, DS>, Self::Err>
  where
    D: Dimensionable,
    RS: RenderSlots,
    DS: DepthRenderSlot;

  unsafe fn back_buffer<D, RS, DS>(
    &mut self,
    size: D::Size,
  ) -> Result<Framebuffer<D, RS, DS>, Self::Err>
  where
    D: Dimensionable,
    RS: RenderSlots,
    DS: DepthRenderSlot;
}

pub unsafe trait ShaderBackend: BackendErr {
  unsafe fn new_program<V, P, S, E>(
    &mut self,
    vertex_code: String,
    primitive_code: String,
    shading_code: String,
  ) -> Result<Program<V, P, S, E>, Self::Err>
  where
    V: Vertex,
    P: Primitive,
    S: RenderSlots,
    E: FromEnv;

  unsafe fn new_shader_env<T>(
    &mut self,
    program_handle: usize,
    name: &str,
  ) -> Result<Env<T>, Self::Err>
  where
    T: IsEnv;

  unsafe fn set_program_env<T>(
    &mut self,
    program_handle: usize,
    env_handle: usize,
    value: T,
  ) -> Result<(), Self::Err>
  where
    T: IsEnv;

  unsafe fn new_shader_shared_env<T>(
    &mut self,
    program_handle: usize,
    name: &str,
  ) -> Result<SharedEnv<T>, Self::Err>
  where
    T: IsSharedEnv;
}

pub unsafe trait PipelineBackend: BackendErr {
  unsafe fn with_framebuffer<'a, D, CS, DS, Err>(
    &'a mut self,
    framebuffer: &Framebuffer<D, CS, DS>,
    state: &PipelineState,
    f: impl FnOnce(WithFramebuffer<'a, Self, CS>) -> Result<(), Err>,
  ) -> Result<(), Self::Err>
  where
    D: Dimensionable,
    CS: RenderSlots,
    DS: DepthRenderSlot,
    Err: From<Self::Err>;

  unsafe fn with_program<'a, V, P, S, E, Err>(
    &'a mut self,
    program: &Program<V, P, S, E>,
    f: impl FnOnce(WithProgram<'a, Self, V, P, S, E>) -> Result<(), Err>,
  ) -> Result<(), Self::Err>
  where
    V: Vertex,
    P: Primitive,
    S: RenderSlots,
    E: FromEnv,
    Err: From<Self::Err>;

  unsafe fn with_render_state<'a, V, Err>(
    &'a mut self,
    render_state: &RenderState,
    f: impl FnOnce(WithRenderState<'a, Self, V>) -> Result<(), Err>,
  ) -> Result<(), Self::Err>
  where
    V: Vertex,
    Err: From<Self::Err>;

  unsafe fn render_vertex_entity<V>(&mut self, view: VertexEntityView<V>) -> Result<(), Self::Err>
  where
    V: Vertex;
}
