use std::{
  marker::PhantomData,
  ops::{Deref, DerefMut},
};

#[derive(Debug)]
pub struct VertexEntity<V, S> {
  handle: usize,
  vertex_count: usize,
  index_count: usize,
  instance_count: usize,
  _phantom: PhantomData<*const (V, S)>,
}

impl<V, S> VertexEntity<V, S> {
  pub unsafe fn new(
    handle: usize,
    vertex_count: usize,
    index_count: usize,
    instance_count: usize,
  ) -> Self {
    Self {
      handle,
      vertex_count,
      index_count,
      instance_count,
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
    self.index_count
  }

  pub fn instance_count(&self) -> usize {
    self.instance_count
  }
}

#[derive(Debug)]
pub struct Vertices<'a, V, S> {
  storage: &'a mut S,
  _phantom: PhantomData<*const V>,
}

impl<'a, V, S> Deref for Vertices<'a, V, S> {
  type Target = S;

  fn deref(&self) -> &Self::Target {
    self.storage
  }
}

impl<'a, V, S> DerefMut for Vertices<'a, V, S> {
  fn deref_mut(&mut self) -> &mut Self::Target {
    self.storage
  }
}

#[derive(Debug)]
pub struct Indices<'a> {
  indices: &'a mut Vec<u32>,
}

impl<'a> Deref for Indices<'a> {
  type Target = Vec<u32>;

  fn deref(&self) -> &Self::Target {
    self.indices
  }
}

impl<'a> DerefMut for Indices<'a> {
  fn deref_mut(&mut self) -> &mut Self::Target {
    self.indices
  }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct VertexEntityView<V> {
  vertex_entity_handle: usize,

  /// First vertex to start rendering from.
  start_vertex: usize,

  /// How many vertices to render.
  vertex_count: usize,

  /// How many instances to render.
  instance_count: usize,

  _phantom: PhantomData<*const V>,
}

impl<V> VertexEntityView<V> {
  pub fn new<S>(entity: &VertexEntity<V, S>) -> Self {
    let vertex_entity_handle = entity.handle;
    let vertex_count = entity.vertex_count;
    let instance_count = entity.instance_count;

    Self {
      vertex_entity_handle,
      start_vertex: 0,
      vertex_count,
      instance_count,
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
