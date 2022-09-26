use crate::{
  backend::{VertexEntityBackend, VertexEntityError},
  primitive::Primitive,
  vertex::Vertex,
  vertex_storage::VertexStorage,
};

#[derive(Debug)]
pub struct VertexEntity<B, V, P, S>
where
  B: VertexEntityBackend,
{
  repr: B::VertexEntityRepr<V, P, S>,
}

impl<B, V, P, S> VertexEntity<B, V, P, S>
where
  B: VertexEntityBackend,
  V: Vertex,
  P: Primitive,
  S: Into<VertexStorage<V>>,
{
  pub unsafe fn new(repr: B::VertexEntityRepr<V, P, S>) -> Self {
    Self { repr }
  }

  pub fn handle(&self) -> usize {
    self.handle
  }

  pub fn vertex_count(&self) -> usize {
    unsafe { B::vertex_entity_vertex_count(&self.repr) }
  }

  pub fn index_count(&self) -> usize {
    unsafe { B::vertex_entity_index_count(&self.repr) }
  }

  pub fn primitive_restart(&self) -> bool {
    unsafe { B::vertex_entity_primitive_restart(&self.repr) }
  }

  pub fn view(&self) -> VertexEntityView<B, V, P, S> {
    VertexEntityView::new(self)
  }

  pub fn vertices(&self) -> Result<Vertices<B, V>, VertexEntityError> {
    unsafe { B::vertex_entity_vertices(&self.repr).map(|repr| Vertices::new(repr)) }
  }
}

#[derive(Debug)]
pub struct Vertices<'a, B, V>
where
  B: VertexEntityBackend,
{
  repr: B::VerticesRepr<'a, V>,
}

impl<'a, B, V> Vertices<'a, B, V>
where
  B: VertexEntityBackend,
{
  pub unsafe fn new(repr: B::VerticesRepr<'a, V>) -> Self {
    Self { repr }
  }
}

impl<'a, B, V> Drop for Vertices<'a, B, V>
where
  B: VertexEntityBackend,
  V: Vertex,
{
  fn drop(&mut self) {
    #[cfg(not(feature = "log"))]
    unsafe {
      let _ = B::vertex_entity_update_vertices(&self.repr);
    }

    #[cfg(feature = "log")]
    unsafe {
      if let Err(err) = B::vertex_entity_update_vertices(&self.repr) {
        log::err!("error while dropping Vertices: {}", e);
      }
    }
  }
}

#[derive(Debug)]
pub struct Indices<'a, B>
where
  B: VertexEntityBackend,
{
  repr: B::IndicesRepr<'a>,
}

impl<'a, B> Indices<'a, B>
where
  B: VertexEntityBackend,
{
  pub unsafe fn new(repr: B::IndicesRepr<'a>) -> Self {
    Self { repr }
  }
}

impl<'a, B> Drop for Indices<'a, B>
where
  B: VertexEntityBackend,
{
  fn drop(&mut self) {
    #[cfg(not(feature = "log"))]
    unsafe {
      let _ = B::vertex_entity_update_indices(&self.repr);
    }

    #[cfg(feature = "log")]
    unsafe {
      if let Err(err) = B::vvrtex_entity_update_indices(&self.repr) {
        log::err!("error while dropping Indices: {}", e);
      }
    }
  }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct VertexEntityView<'a, B, V, P, S>
where
  B: VertexEntityBackend,
{
  vertex_entity: &'a VertexEntity<B, V, P, S>,

  /// First vertex to start rendering from.
  start_vertex: usize,

  /// How many vertices to render.
  vertex_count: usize,

  /// How many instances to render.
  instance_count: usize,
}

impl<'a, B, V, P, S> VertexEntityView<'a, B, V, P, S> {
  pub fn new(vertex_entity: &'a VertexEntity<B, V, P, S>) -> Self {
    let vertex_count = vertex_entity.vertex_count;

    Self {
      vertex_entity,
      start_vertex: 0,
      vertex_count,
      instance_count: 1,
    }
  }

  pub fn start_vertex(mut self, start_vertex: usize) -> Self {
    self.start_vertex = start_vertex;
    self
  }

  pub fn vertex_count(mut self, count: usize) -> Self {
    self.vertex_count = count;
    self
  }

  pub fn instance_count(mut self, count: usize) -> Self {
    self.instance_count = count;
    self
  }
}
