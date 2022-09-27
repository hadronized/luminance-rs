use std::marker::PhantomData;

use crate::{primitive::Primitive, vertex::Vertex, vertex_storage::AsVertexStorage};

pub struct VertexEntity<V, P, S> {
  handle: usize,
  storage: S,
  indices: Vec<u32>,
  vertex_count: usize,
  primitive_restart: bool,
  dropper: Box<dyn FnMut(usize)>,
  _phantom: PhantomData<*const (V, P, S)>,
}

impl<V, P, S> VertexEntity<V, P, S>
where
  V: Vertex,
  P: Primitive,
  S: AsVertexStorage<V>,
{
  pub unsafe fn new(
    handle: usize,
    storage: S,
    indices: Vec<u32>,
    vertex_count: usize,
    primitive_restart: bool,
    dropper: Box<dyn FnMut(usize)>,
  ) -> Self {
    Self {
      handle,
      storage,
      indices,
      vertex_count,
      primitive_restart,
      dropper,
      _phantom: PhantomData,
    }
  }

  pub fn handle(&self) -> usize {
    self.handle
  }

  pub fn vertex_count(&self) -> usize {
    self.vertex_count
  }

  pub fn index_count(&self) -> usize {
    self.indices.len()
  }

  pub fn primitive_restart(&self) -> bool {
    self.primitive_restart
  }

  pub fn storage(&mut self) -> &mut S {
    &mut self.storage
  }

  pub fn indices(&mut self) -> &mut Vec<u32> {
    &mut self.indices
  }

  pub fn view(&self) -> VertexEntityView<V, P> {
    VertexEntityView::new(self)
  }
}

impl<V, P, S> Drop for VertexEntity<V, P, S> {
  fn drop(&mut self) {
    (self.dropper)(self.handle);
  }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct VertexEntityView<V, P> {
  handle: usize,

  /// First vertex to start rendering from.
  start_vertex: usize,

  /// How many vertices to render.
  vertex_count: usize,

  /// How many instances to render.
  instance_count: usize,

  _phantom: PhantomData<*const (V, P)>,
}

impl<'a, V, P> VertexEntityView<V, P> {
  pub fn new<S>(vertex_entity: &VertexEntity<V, P, S>) -> Self {
    let handle = vertex_entity.handle;
    let vertex_count = vertex_entity.vertex_count;

    Self {
      handle,
      start_vertex: 0,
      vertex_count,
      instance_count: 1,
      _phantom: PhantomData,
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
