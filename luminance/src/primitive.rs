pub trait Primitive {
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
    pub struct $t;

    impl Primitive for $t {
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
pub struct Patch<const SIZE: usize>;

impl<const SIZE: usize> Primitive for Patch<SIZE> {
  const CONNECTOR: Connector = Connector::Patch(SIZE);
}
