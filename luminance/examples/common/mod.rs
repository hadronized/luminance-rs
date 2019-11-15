use luminance_derive::{Semantics, Vertex};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Semantics)]
pub enum Semantics {
  // reference vertex positions with the co variable in vertex shaders
  #[sem(name = "co", repr = "[f32; 2]", wrapper = "VertexPosition")]
  Position,
  // reference vertex colors with the color variable in vertex shaders
  #[sem(name = "color", repr = "[f32; 3]", wrapper = "VertexColor")]
  Color,
  // reference vertex instance’s position on screen
  #[sem(
    name = "position",
    repr = "[f32; 2]",
    wrapper = "VertexInstancePosition"
  )]
  InstancePosition,
  // reference vertex size in vertex shaders (used for vertex instancing)
  #[sem(name = "weight", repr = "f32", wrapper = "VertexWeight")]
  Weight,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Vertex)]
#[vertex(sem = "Semantics")]
pub struct Vertex {
  pub pos: VertexPosition,
  pub rgb: VertexColor,
}

// definition of a single instance
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Vertex)]
#[vertex(sem = "Semantics", instanced = "true")]
pub struct Instance {
  pub pos: VertexInstancePosition,
  pub w: VertexWeight,
}
