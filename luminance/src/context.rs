use crate::{
  backend::Backend,
  vertex::Vertex,
  vertex_entity::{Indices, VertexEntity, Vertices},
  vertex_storage::VertexStorage,
};

#[derive(Debug)]
pub struct Context<B> {
  backend: B,
}

impl<B> Context<B>
where
  B: Backend,
{
  pub unsafe fn new(backend: B) -> Self {
    Self { backend }
  }

  pub fn new_vertex_entity<V, S, I>(
    &mut self,
    storage: S,
    indices: I,
  ) -> Result<VertexEntity<V, S>, B::Err>
  where
    V: Vertex,
    S: VertexStorage<V>,
    I: Into<Vec<u32>>,
  {
    unsafe { self.backend.new_vertex_entity(storage, indices) }
  }

  pub fn vertices<'a, V, S>(
    &'a mut self,
    entity: &VertexEntity<V, S>,
  ) -> Result<Vertices<'a, V, S>, B::Err>
  where
    V: Vertex,
    S: VertexStorage<V>,
  {
    unsafe { self.backend.vertex_entity_vertices(entity) }
  }

  pub fn update_vertices<'a, V, S>(
    &'a mut self,
    entity: &VertexEntity<V, S>,
    vertices: Vertices<'a, V, S>,
  ) -> Result<(), B::Err>
  where
    V: Vertex,
    S: VertexStorage<V>,
  {
    unsafe { self.backend.vertex_entity_update_vertices(entity, vertices) }
  }

  pub fn indices<'a, V, S>(&'a mut self, entity: &VertexEntity<V, S>) -> Result<Indices<'a>, B::Err>
  where
    V: Vertex,
    S: VertexStorage<V>,
  {
    unsafe { self.backend.vertex_entity_indices(entity) }
  }

  pub fn update_indices<'a, V, S>(
    &'a mut self,
    entity: &VertexEntity<V, S>,
    indices: Indices<'a>,
  ) -> Result<(), B::Err>
  where
    V: Vertex,
    S: VertexStorage<V>,
  {
    unsafe { self.backend.vertex_entity_update_indices(entity, indices) }
  }
}
