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
