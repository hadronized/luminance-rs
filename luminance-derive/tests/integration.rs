use luminance::vertex::{
  HasSemantics, IndexedVertexAttribFmt, Vertex, VertexAttrib, VertexAttribSem, VertexInstancing
};
use luminance_derive::{Vertex, VertexAttribSem};

#[test]
fn derive_simple_semantics() {
  #[derive(Clone, Copy, Debug, Eq, PartialEq, VertexAttribSem)]
  pub enum Semantics {
    #[sem(name = "position", repr = "[f32; 3]", type_name = "VertexPosition")]
    Position,
    #[sem(name = "normal", repr = "[f32; 3]", type_name = "VertexNormal")]
    Normal,
    #[sem(name = "color", repr = "[f32; 4]", type_name = "VertexColor")]
    Color
  }

  #[derive(Clone, Copy, Debug, Vertex)]
  #[vertex(sem = "Semantics", instanced = "true")]
  struct Vertex {
    pos: VertexPosition,
    nor: VertexNormal,
    col: VertexColor
  }

  assert_eq!(Semantics::Position.index(), 0);
  assert_eq!(Semantics::Normal.index(), 1);
  assert_eq!(Semantics::Color.index(), 2);
  assert_eq!(<Semantics as VertexAttribSem>::parse("position"), Some(Semantics::Position));
  assert_eq!(<Semantics as VertexAttribSem>::parse("normal"), Some(Semantics::Normal));
  assert_eq!(<Semantics as VertexAttribSem>::parse("color"), Some(Semantics::Color));
  assert_eq!(<Semantics as VertexAttribSem>::parse("bidule"), None);
  assert_eq!(VertexPosition::VERTEX_ATTRIB_SEM, Semantics::Position);
  assert_eq!(VertexNormal::VERTEX_ATTRIB_SEM, Semantics::Normal);
  assert_eq!(VertexColor::VERTEX_ATTRIB_SEM, Semantics::Color);
  assert_eq!(VertexPosition::new([1., 2., 3.]).repr, [1., 2., 3.]);

  let expected_fmt = vec![
    IndexedVertexAttribFmt::new(Semantics::Position, VertexInstancing::On, <[f32; 3] as VertexAttrib>::VERTEX_ATTRIB_FMT),
    IndexedVertexAttribFmt::new(Semantics::Normal, VertexInstancing::On, <[f32; 3] as VertexAttrib>::VERTEX_ATTRIB_FMT),
    IndexedVertexAttribFmt::new(Semantics::Color, VertexInstancing::On, <[f32; 4] as VertexAttrib>::VERTEX_ATTRIB_FMT),
  ];

  assert_eq!(Vertex::vertex_fmt(), expected_fmt);
}
