use luminance_derive::{Semantics, Vertex};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Semantics)]
pub enum Semantics {
  // reference vertex positions with the co variable in vertex shaders
  #[sem(name = "co", repr = "[f32; 2]", type_name = "VertexPosition")]
  Position,
  // reference vertex colors with the color variable in vertex shaders
  #[sem(name = "color", repr = "[f32; 3]", type_name = "VertexColor")]
  Color,
  // reference verteex instance’s position on screen
  #[sem(name = "position", repr = "[f32; 2]", type_name = "VertexInstancePosition")]
  InstancePosition,
  // reference vertex size in vertex shaders (used for vertex instancing)
  #[sem(name = "weight", repr = "f32", type_name = "VertexWeight")]
  Weight,
}

#[derive(Clone, Copy, Debug, PartialEq, Vertex)]
#[vertex(sem = "Semantics")]
pub struct Vertex {
  pub pos: VertexPosition,
  pub rgb: VertexColor,
}

// definition of a single instance
#[derive(Clone, Copy, Debug, PartialEq, Vertex)]
#[vertex(sem = "Semantics", instanced = "true")]
pub struct Instance {
  pub pos: VertexInstancePosition,
  pub w: VertexWeight
}
