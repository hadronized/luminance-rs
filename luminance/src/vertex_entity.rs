use crate::{primitive::Primitive, vertex::Vertex, vertex_storage::AsVertexStorage};
use std::{
  marker::PhantomData,
  ops::{Range, RangeFrom, RangeFull, RangeTo, RangeToInclusive},
};

#[derive(Debug)]
pub struct VertexEntityBuilder<VS, WS> {
  pub vertices: VS,
  pub indices: Vec<u32>,
  pub instances: WS,
}

impl VertexEntityBuilder<(), ()> {
  pub fn new() -> Self {
    Self {
      vertices: (),
      indices: Vec::new(),
      instances: (),
    }
  }
}

impl<WS> VertexEntityBuilder<(), WS> {
  pub fn add_vertices<VS>(self, vertices: VS) -> VertexEntityBuilder<VS, WS> {
    VertexEntityBuilder {
      vertices,
      indices: self.indices,
      instances: self.instances,
    }
  }
}

impl<VS> VertexEntityBuilder<VS, ()> {
  pub fn add_instances<WS>(self, instances: WS) -> VertexEntityBuilder<VS, WS> {
    VertexEntityBuilder {
      vertices: self.vertices,
      indices: self.indices,
      instances,
    }
  }
}

impl<VS, WS> VertexEntityBuilder<VS, WS> {
  pub fn add_indices(self, indices: impl Into<Vec<u32>>) -> Self {
    Self {
      indices: indices.into(),
      ..self
    }
  }
}

pub struct VertexEntity<V, P, VS, W = (), WS = ()> {
  handle: usize,
  vertices: VS,
  instance_data: WS,
  indices: Vec<u32>,
  vertex_count: usize,
  dropper: Box<dyn FnMut(usize)>,
  _phantom: PhantomData<*const (V, P, W)>,
}

impl<V, P, VS, W, WS> VertexEntity<V, P, VS, W, WS>
where
  V: Vertex,
  P: Primitive,
  VS: AsVertexStorage<V>,
  W: Vertex,
  WS: AsVertexStorage<W>,
{
  pub unsafe fn new(
    handle: usize,
    vertices: VS,
    instance_data: WS,
    indices: Vec<u32>,
    vertex_count: usize,
    dropper: Box<dyn FnMut(usize)>,
  ) -> Self {
    Self {
      handle,
      vertices,
      instance_data,
      indices,
      vertex_count,
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

  pub fn vertices(&mut self) -> &mut VS {
    &mut self.vertices
  }

  pub fn instance_data(&mut self) -> &mut WS {
    &mut self.instance_data
  }

  pub fn indices(&mut self) -> &mut Vec<u32> {
    &mut self.indices
  }
}

impl<V, P, VS, W, WS> Drop for VertexEntity<V, P, VS, W, WS> {
  fn drop(&mut self) {
    (self.dropper)(self.handle);
  }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct VertexEntityView<V, W, P> {
  handle: usize,

  /// First vertex to start rendering from.
  start_vertex: usize,

  /// How many vertices to render.
  vertex_count: usize,

  /// How many instances to render.
  instance_count: usize,

  _phantom: PhantomData<*const (V, W, P)>,
}

impl<'a, V, W, P> VertexEntityView<V, W, P> {
  pub fn new<S, WS>(vertex_entity: &VertexEntity<V, P, S, W, WS>) -> Self {
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
}

pub trait View<R> {
  type Vertex: Vertex;
  type Instance: Vertex;
  type Primitive: Primitive;

  fn view(&self, range: R) -> VertexEntityView<Self::Vertex, Self::Instance, Self::Primitive>;
}

impl<V, P, VS, W, WS> View<RangeFull> for VertexEntity<V, P, VS, W, WS>
where
  V: Vertex,
  P: Primitive,
  VS: AsVertexStorage<V>,
  W: Vertex,
  WS: AsVertexStorage<W>,
{
  type Vertex = V;
  type Instance = W;
  type Primitive = P;

  fn view(&self, _: RangeFull) -> VertexEntityView<Self::Vertex, Self::Instance, Self::Primitive> {
    VertexEntityView::new(self)
  }
}

impl<V, P, VS, W, WS> View<Range<usize>> for VertexEntity<V, P, VS, W, WS>
where
  V: Vertex,
  P: Primitive,
  VS: AsVertexStorage<V>,
  W: Vertex,
  WS: AsVertexStorage<W>,
{
  type Vertex = V;
  type Instance = W;
  type Primitive = P;

  fn view(
    &self,
    range: Range<usize>,
  ) -> VertexEntityView<Self::Vertex, Self::Instance, Self::Primitive> {
    VertexEntityView {
      handle: self.handle(),
      start_vertex: range.start,
      vertex_count: range.end,
      instance_count: 1,
      _phantom: PhantomData,
    }
  }
}

impl<V, P, VS, W, WS> View<RangeFrom<usize>> for VertexEntity<V, P, VS, W, WS>
where
  V: Vertex,
  P: Primitive,
  VS: AsVertexStorage<V>,
  W: Vertex,
  WS: AsVertexStorage<W>,
{
  type Vertex = V;
  type Instance = W;
  type Primitive = P;

  fn view(
    &self,
    range: RangeFrom<usize>,
  ) -> VertexEntityView<Self::Vertex, Self::Instance, Self::Primitive> {
    VertexEntityView {
      handle: self.handle(),
      start_vertex: range.start,
      vertex_count: self.vertex_count,
      instance_count: 1,
      _phantom: PhantomData,
    }
  }
}

impl<V, P, VS, W, WS> View<RangeTo<usize>> for VertexEntity<V, P, VS, W, WS>
where
  V: Vertex,
  P: Primitive,
  VS: AsVertexStorage<V>,
  W: Vertex,
  WS: AsVertexStorage<W>,
{
  type Vertex = V;
  type Instance = W;
  type Primitive = P;

  fn view(
    &self,
    range: RangeTo<usize>,
  ) -> VertexEntityView<Self::Vertex, Self::Instance, Self::Primitive> {
    VertexEntityView {
      handle: self.handle(),
      start_vertex: 0,
      vertex_count: range.end,
      instance_count: 1,
      _phantom: PhantomData,
    }
  }
}

impl<V, P, VS, W, WS> View<RangeToInclusive<usize>> for VertexEntity<V, P, VS, W, WS>
where
  V: Vertex,
  P: Primitive,
  VS: AsVertexStorage<V>,
  W: Vertex,
  WS: AsVertexStorage<W>,
{
  type Vertex = V;
  type Instance = W;
  type Primitive = P;

  fn view(
    &self,
    range: RangeToInclusive<usize>,
  ) -> VertexEntityView<Self::Vertex, Self::Instance, Self::Primitive> {
    VertexEntityView {
      handle: self.handle(),
      start_vertex: 0,
      vertex_count: range.end + 1,
      instance_count: 1,
      _phantom: PhantomData,
    }
  }
}
