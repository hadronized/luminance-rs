#![allow(incomplete_features)]
#![feature(adt_const_params)]
#![cfg(feature = "derive")]

use luminance::{
  namespace,
  vertex::{Vertex as _, VertexAttrib, VertexBufferDesc, VertexInstancing},
  Vertex,
};

namespace! {
  Namespace = { "pos", "nor", "col" }
}

#[derive(Clone, Copy, Debug, Vertex)]
#[repr(C)]
#[vertex(namespace = "Namespace", instanced = "true")]
struct Vertex {
  pos: [f32; 3],
  nor: [f32; 3],
  col: [f32; 4],
}

#[test]
fn derive_vertex() {
  let expected_desc = vec![
    VertexBufferDesc::new(
      0,
      "pos",
      VertexInstancing::On,
      <[f32; 3] as VertexAttrib>::VERTEX_ATTRIB_DESC,
    ),
    VertexBufferDesc::new(
      1,
      "nor",
      VertexInstancing::On,
      <[f32; 3] as VertexAttrib>::VERTEX_ATTRIB_DESC,
    ),
    VertexBufferDesc::new(
      2,
      "col",
      VertexInstancing::On,
      <[f32; 4] as VertexAttrib>::VERTEX_ATTRIB_DESC,
    ),
  ];

  assert_eq!(Vertex::vertex_desc(), expected_desc);
}
