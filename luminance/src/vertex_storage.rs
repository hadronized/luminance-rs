//! Vertex storage containers.

use crate::{
  backend::VertexEntityError,
  has_field::HasField,
  vertex::{Deinterleave, Vertex},
};
use std::{marker::PhantomData, mem};

/// Store vertices as an interleaved array.
#[derive(Debug)]
pub struct Interleaved<V> {
  vertices: Vec<V>,
}

impl<V> Interleaved<V> {
  /// Build a new interleaved storage.
  pub fn new() -> Self {
    Self {
      vertices: Vec::new(),
    }
  }

  /// Set vertices.
  pub fn set_vertices(mut self, vertices: impl Into<Vec<V>>) -> Self {
    self.vertices = vertices.into();
    self
  }

  /// Get access to the vertices as bytes.
  pub fn vertices_as_bytes(&self) -> &[u8] {
    let data = self.vertices.as_ptr();
    let len = self.vertices.len();

    unsafe { std::slice::from_raw_parts(data as _, len * mem::size_of::<V>()) }
  }
}

/// Store vertices as deinterleaved arrays.
#[derive(Debug)]
pub struct Deinterleaved<V> {
  components_list: Vec<Vec<u8>>,
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
}

pub trait VertexStorageVisitor<'a, V>
where
  V: Vertex,
{
  fn visit_interleaved(&mut self, _: &'a mut Interleaved<V>) -> Result<(), VertexEntityError> {
    Ok(())
  }

  fn visit_deinterleaved(&mut self, _: &'a mut Deinterleaved<V>) -> Result<(), VertexEntityError> {
    Ok(())
  }
}

pub struct InterleavedVisitor<F> {
  f: F,
}

impl<F> InterleavedVisitor<F> {
  pub fn new(f: F) -> Self {
    Self { f }
  }
}

impl<'a, F, V> VertexStorageVisitor<'a, V> for InterleavedVisitor<F>
where
  F: FnMut(&'a mut Interleaved<V>) -> Result<(), VertexEntityError>,
  V: 'a + Vertex,
{
  fn visit_interleaved(
    &mut self,
    storage: &'a mut Interleaved<V>,
  ) -> Result<(), VertexEntityError> {
    (self.f)(storage)
  }
}

pub struct DeinterleavedVisitor<F> {
  f: F,
}

impl<F> DeinterleavedVisitor<F> {
  pub fn new(f: F) -> Self {
    Self { f }
  }
}

impl<'a, F, V> VertexStorageVisitor<'a, V> for DeinterleavedVisitor<F>
where
  F: FnMut(&'a mut Deinterleaved<V>) -> Result<(), VertexEntityError>,
  V: 'a + Vertex,
{
  fn visit_deinterleaved(
    &mut self,
    storage: &'a mut Deinterleaved<V>,
  ) -> Result<(), VertexEntityError> {
    (self.f)(storage)
  }
}

pub trait VertexStorage<V>
where
  V: Vertex,
{
  fn visit<'a>(
    &'a mut self,
    visitor: &mut impl VertexStorageVisitor<'a, V>,
  ) -> Result<(), VertexEntityError>;
}

impl<V> VertexStorage<V> for Interleaved<V>
where
  V: Vertex,
{
  fn visit<'a>(
    &'a mut self,
    visitor: &mut impl VertexStorageVisitor<'a, V>,
  ) -> Result<(), VertexEntityError> {
    visitor.visit_interleaved(self)
  }
}

impl<V> VertexStorage<V> for Deinterleaved<V>
where
  V: Vertex,
{
  fn visit<'a>(
    &'a mut self,
    visitor: &mut impl VertexStorageVisitor<'a, V>,
  ) -> Result<(), VertexEntityError> {
    visitor.visit_deinterleaved(self)
  }
}
