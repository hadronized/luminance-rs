#![allow(missing_docs)]

use crate::{
  dim::Dimensionable,
  framebuffer::Framebuffer,
  primitive::Primitive,
  render_channel::{IsDepthChannelType, IsRenderChannelType},
  render_slots::{DepthRenderSlot, RenderLayer, RenderSlots},
  shader::{Env, FromEnv, IsEnv, IsSharedEnv, Program, SharedEnv},
  vertex::Vertex,
  vertex_entity::{Indices, VertexEntity, Vertices},
  vertex_storage::VertexStorage,
};

pub unsafe trait Backend {
  type Err;

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
