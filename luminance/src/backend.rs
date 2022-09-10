#![allow(missing_docs)]

use crate::{
  dim::Dimensionable,
  framebuffer::Framebuffer,
  render_channel::IsRenderChannelType,
  render_slots::{RenderLayer, RenderSlots},
  vertex::Vertex,
  vertex_entity::{Indices, VertexEntity, Vertices},
  vertex_storage::VertexStorage,
};

pub mod color_slot;
pub mod depth_stencil_slot;
pub mod framebuffer;
pub mod pipeline;
pub mod query;
pub mod render_gate;
pub mod shader;
pub mod shading_gate;
pub mod tess_gate;
pub mod texture;

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

  unsafe fn new_framebuffer<D, RS>(
    &mut self,
    size: D::Size,
  ) -> Result<Framebuffer<D, RS>, Self::Err>
  where
    D: Dimensionable,
    RS: RenderSlots;
}
