use crate::vertex::Vertex;
use std::marker::PhantomData;

pub trait Primitive {
  type Vertex: Vertex;

  const CONNECTOR: Connector;
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Connector {
  Point,
  Line,
  LineStrip,
  Triangle,
  TriangleStrip,
  TriangleFan,
  Patch(usize),
}

macro_rules! impl_Primitive {
  ($t:ident) => {
    #[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
    pub struct $t<V> {
      _phantom: PhantomData<V>,
    }

    impl<V: Vertex> Primitive for $t<V> {
      type Vertex = V;

      const CONNECTOR: Connector = Connector::$t;
    }
  };
}

impl_Primitive!(Point);
impl_Primitive!(Line);
impl_Primitive!(LineStrip);
impl_Primitive!(Triangle);
impl_Primitive!(TriangleStrip);
impl_Primitive!(TriangleFan);

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct Patch<const SIZE: usize, V> {
  _phantom: PhantomData<V>,
}

impl<const SIZE: usize, V: Vertex> Primitive for Patch<SIZE, V> {
  type Vertex = V;

  const CONNECTOR: Connector = Connector::Patch(SIZE);
}
