#![allow(incomplete_features)]
#![feature(adt_const_params)]
#![cfg(feature = "derive")]

use luminance::{
  has_field::HasField,
  namespace,
  vertex::{CompatibleVertex, Vertex as _, VertexAttrib, VertexComponent, VertexInstancing},
  Vertex,
};

namespace! {
  Namespace = { "pos", "nor", "col", "weight" }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Vertex)]
#[vertex(namespace = "Namespace")]
struct Vertex {
  pos: [f32; 3],
  nor: [f32; 3],
  col: [u8; 4],
}

#[test]
fn vertex_desc() {
  let expected_desc = vec![
    VertexComponent::new(
      0,
      "pos",
      VertexInstancing::Off,
      <[f32; 3] as VertexAttrib>::VERTEX_ATTRIB_DESC,
    ),
    VertexComponent::new(
      1,
      "nor",
      VertexInstancing::Off,
      <[f32; 3] as VertexAttrib>::VERTEX_ATTRIB_DESC,
    ),
    VertexComponent::new(
      2,
      "col",
      VertexInstancing::Off,
      <[u8; 4] as VertexAttrib>::VERTEX_ATTRIB_DESC,
    ),
  ];

  assert_eq!(Vertex::vertex_desc(), expected_desc);
}

#[test]
fn has_field() {
  fn must_have_field<const NAME: &'static str, V, F>()
  where
    V: HasField<NAME, FieldType = F>,
  {
  }

  must_have_field::<"pos", Vertex, [f32; 3]>();
  must_have_field::<"nor", Vertex, [f32; 3]>();
  must_have_field::<"col", Vertex, [u8; 4]>();
}

#[test]
fn compatible_vertex_types() {
  fn is_compatible<V, W>()
  where
    V: CompatibleVertex<W>,
  {
  }

  #[repr(C)]
  #[derive(Clone, Copy, Debug, Vertex)]
  #[vertex(namespace = "Namespace")]
  struct VertexSame {
    pos: [f32; 3],
    nor: [f32; 3],
    col: [u8; 4],
  }

  is_compatible::<Vertex, VertexSame>();

  #[repr(C)]
  #[derive(Clone, Copy, Debug, Vertex)]
  #[vertex(namespace = "Namespace")]
  struct VertexInclude {
    pos: [f32; 3],
    nor: [f32; 3],
    col: [u8; 4],
    weight: f32,
  }

  is_compatible::<Vertex, VertexInclude>();
}
