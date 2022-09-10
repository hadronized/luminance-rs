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
