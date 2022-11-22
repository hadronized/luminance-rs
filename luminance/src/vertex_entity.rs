use crate::{primitive::Primitive, vertex::Vertex, vertex_storage::AsVertexStorage};
use std::{
  marker::PhantomData,
  ops::{Range, RangeFrom, RangeFull, RangeTo, RangeToInclusive},
};

pub struct VertexEntity<V, P, VS> {
  handle: usize,
  vertices: VS,
  indices: Vec<u32>,
  vertex_count: usize,
  primitive_restart: bool,
  dropper: Box<dyn FnMut(usize)>,
  _phantom: PhantomData<*const (V, P, VS)>,
}

impl<V, P, VS> VertexEntity<V, P, VS>
where
  V: Vertex,
  P: Primitive,
  VS: AsVertexStorage<V>,
{
  pub unsafe fn new(
    handle: usize,
    vertices: VS,
    indices: Vec<u32>,
    vertex_count: usize,
    primitive_restart: bool,
    dropper: Box<dyn FnMut(usize)>,
  ) -> Self {
    Self {
      handle,
      vertices,
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

  pub fn vertices(&mut self) -> &mut VS {
    &mut self.vertices
  }

  pub fn indices(&mut self) -> &mut Vec<u32> {
    &mut self.indices
  }
}

impl<V, P, VS> Drop for VertexEntity<V, P, VS> {
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

  primitive_restart: bool,

  _phantom: PhantomData<*const (V, P)>,
}

impl<'a, V, P> VertexEntityView<V, P> {
  pub fn new<S>(vertex_entity: &VertexEntity<V, P, S>) -> Self {
    let handle = vertex_entity.handle;
    let vertex_count = vertex_entity.vertex_count;
    let primitive_restart = vertex_entity.primitive_restart;

    Self {
      handle,
      start_vertex: 0,
      vertex_count,
      instance_count: 1,
      primitive_restart,
      _phantom: PhantomData,
    }
  }

  pub fn handle(&self) -> usize {
    self.handle
  }

  pub fn start_vertex(&self) -> usize {
    self.start_vertex
  }

  pub fn set_start_vertex(mut self, start_vertex: usize) -> Self {
    self.start_vertex = start_vertex;
    self
  }

  pub fn vertex_count(&self) -> usize {
    self.vertex_count
  }

  pub fn set_vertex_count(mut self, count: usize) -> Self {
    self.vertex_count = count;
    self
  }

  pub fn instance_count(&self) -> usize {
    self.instance_count
  }

  pub fn set_instance_count(mut self, count: usize) -> Self {
    self.instance_count = count;
    self
  }

  pub fn primitive_restart(&self) -> bool {
    self.primitive_restart
  }

  pub fn set_primitive_restart(mut self, primitive_restart: bool) -> Self {
    self.primitive_restart = primitive_restart;
    self
  }
}

pub trait View<R> {
  type Vertex: Vertex;
  type Primitive: Primitive<Vertex = Self::Vertex>;

  fn view(&self, range: R) -> VertexEntityView<Self::Vertex, Self::Primitive>;
}

impl<V, P, VS> View<RangeFull> for VertexEntity<V, P, VS>
where
  V: Vertex,
  P: Primitive<Vertex = V>,
  VS: AsVertexStorage<V>,
{
  type Vertex = V;
  type Primitive = P;

  fn view(&self, _: RangeFull) -> VertexEntityView<Self::Vertex, Self::Primitive> {
    VertexEntityView::new(self)
  }
}

impl<V, P, VS> View<Range<usize>> for VertexEntity<V, P, VS>
where
  V: Vertex,
  P: Primitive<Vertex = V>,
  VS: AsVertexStorage<V>,
{
  type Vertex = V;
  type Primitive = P;

  fn view(&self, range: Range<usize>) -> VertexEntityView<Self::Vertex, Self::Primitive> {
    VertexEntityView {
      handle: self.handle(),
      start_vertex: range.start,
      vertex_count: range.end,
      instance_count: 1,
      primitive_restart: false,
      _phantom: PhantomData,
    }
  }
}

impl<V, P, VS> View<RangeFrom<usize>> for VertexEntity<V, P, VS>
where
  V: Vertex,
  P: Primitive<Vertex = V>,
  VS: AsVertexStorage<V>,
{
  type Vertex = V;
  type Primitive = P;

  fn view(&self, range: RangeFrom<usize>) -> VertexEntityView<Self::Vertex, Self::Primitive> {
    VertexEntityView {
      handle: self.handle(),
      start_vertex: range.start,
      vertex_count: self.vertex_count,
      instance_count: 1,
      primitive_restart: false,
      _phantom: PhantomData,
    }
  }
}

impl<V, P, VS> View<RangeTo<usize>> for VertexEntity<V, P, VS>
where
  V: Vertex,
  P: Primitive<Vertex = V>,
  VS: AsVertexStorage<V>,
{
  type Vertex = V;
  type Primitive = P;

  fn view(&self, range: RangeTo<usize>) -> VertexEntityView<Self::Vertex, Self::Primitive> {
    VertexEntityView {
      handle: self.handle(),
      start_vertex: 0,
      vertex_count: range.end,
      instance_count: 1,
      primitive_restart: false,
      _phantom: PhantomData,
    }
  }
}

impl<V, P, VS> View<RangeToInclusive<usize>> for VertexEntity<V, P, VS>
where
  V: Vertex,
  P: Primitive<Vertex = V>,
  VS: AsVertexStorage<V>,
{
  type Vertex = V;
  type Primitive = P;

  fn view(
    &self,
    range: RangeToInclusive<usize>,
  ) -> VertexEntityView<Self::Vertex, Self::Primitive> {
    VertexEntityView {
      handle: self.handle(),
      start_vertex: 0,
      vertex_count: range.end + 1,
      instance_count: 1,
      primitive_restart: false,
      _phantom: PhantomData,
    }
  }
}
