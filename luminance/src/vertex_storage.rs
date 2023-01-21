//! Vertex storage containers.

use crate::{
  has_field::HasField,
  vertex::{Deinterleave, Vertex},
};
use std::{marker::PhantomData, mem};

#[derive(Debug)]
pub enum VertexStorage<'a, V> {
  NoStorage,
  Interleaved(&'a mut Interleaved<V>),
  Deinterleaved(&'a mut Deinterleaved<V>),
}

pub trait AsVertexStorage<V> {
  fn as_vertex_storage(&mut self) -> VertexStorage<V>;
}

impl<V> AsVertexStorage<V> for () {
  fn as_vertex_storage(&mut self) -> VertexStorage<V> {
    VertexStorage::NoStorage
  }
}

impl<V> AsVertexStorage<V> for Interleaved<V> {
  fn as_vertex_storage(&mut self) -> VertexStorage<V> {
    VertexStorage::Interleaved(self)
  }
}

impl<V> AsVertexStorage<V> for Deinterleaved<V> {
  fn as_vertex_storage(&mut self) -> VertexStorage<V> {
    VertexStorage::Deinterleaved(self)
  }
}

/// Store vertices as an interleaved array.
#[derive(Debug)]
pub struct Interleaved<V> {
  vertices: Vec<V>,
  primitive_restart: bool,
}

impl<V> Interleaved<V> {
  /// Build a new interleaved storage.
  pub fn new() -> Self {
    Self {
      vertices: Vec::new(),
      primitive_restart: false,
    }
  }

  /// Set vertices.
  pub fn set_vertices(mut self, vertices: impl Into<Vec<V>>) -> Self {
    self.vertices = vertices.into();
    self
  }

  /// Get access to the vertices.
  pub fn vertices(&self) -> &Vec<V> {
    &self.vertices
  }

  /// Get access to the vertices.
  pub fn vertices_mut(&mut self) -> &mut Vec<V> {
    &mut self.vertices
  }

  /// Get access to the vertices as bytes.
  pub fn vertices_as_bytes(&self) -> &[u8] {
    let data = self.vertices.as_ptr();
    let len = self.vertices.len();

    unsafe { std::slice::from_raw_parts(data as _, len * mem::size_of::<V>()) }
  }

  pub fn primitive_restart(&self) -> bool {
    self.primitive_restart
  }

  pub fn set_primitive_restart(mut self, primitive_restart: bool) -> Self {
    self.primitive_restart = primitive_restart;
    self
  }
}

/// Store vertices as deinterleaved arrays.
#[derive(Debug)]
pub struct Deinterleaved<V> {
  components_list: Vec<Vec<u8>>,
  primitive_restart: bool,
  _phantom: PhantomData<V>,
}

impl<V> Deinterleaved<V>
where
  V: Vertex,
{
  /// Create a new empty deinterleaved storage.
  pub fn new() -> Self {
    let components_count = V::components_count();

    Self {
      components_list: vec![Vec::new(); components_count],
      primitive_restart: false,
      _phantom: PhantomData,
    }
  }

  /// Set named components.
  pub fn set_components<const NAME: &'static str>(
    mut self,
    components: impl Into<Vec<<V as HasField<NAME>>::FieldType>>,
  ) -> Self
  where
    V: Deinterleave<NAME>,
  {
    // turn the components into a raw vector (Vec<u8>)
    let boxed_slice = components.into().into_boxed_slice();
    let len = boxed_slice.len();
    let len_bytes = len * std::mem::size_of::<<V as HasField<NAME>>::FieldType>();
    let ptr = Box::into_raw(boxed_slice);
    let raw = unsafe { Vec::from_raw_parts(ptr as _, len_bytes, len_bytes) };

    self.components_list[<V as Deinterleave<NAME>>::RANK] = raw;
    self
  }

  /// Get all components
  pub fn components_list(&self) -> &Vec<Vec<u8>> {
    &self.components_list
  }

  pub fn primitive_restart(&self) -> bool {
    self.primitive_restart
  }

  pub fn set_primitive_restart(mut self, primitive_restart: bool) -> Self {
    self.primitive_restart = primitive_restart;
    self
  }
}
