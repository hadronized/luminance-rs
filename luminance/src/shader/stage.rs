//! _Shader stages_ and their related features.
//!
//! A shader stage is a part of a _shader program_. Typically, _shader programs_ are comprised of
//! _several_ shader stages. The minimal configuration implies at least a _vertex shader_ and a
//! _fragment shader_.

use std::fmt;

/// A shader stage type.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Type {
  /// Tessellation control shader.
  TessellationControlShader,
  /// Tessellation evaluation shader.
  TessellationEvaluationShader,
  /// Vertex shader.
  VertexShader,
  /// Geometry shader.
  GeometryShader,
  /// Fragment shader.
  FragmentShader,
}

impl fmt::Display for Type {
  fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
    match *self {
      Type::TessellationControlShader => f.write_str("tessellation control shader"),
      Type::TessellationEvaluationShader => f.write_str("tessellation evaluation shader"),
      Type::VertexShader => f.write_str("vertex shader"),
      Type::GeometryShader => f.write_str("geometry shader"),
      Type::FragmentShader => f.write_str("fragment shader"),
    }
  }
}

/// A shader stage.
pub trait Stage<C>: Sized {
  type Err;

  /// Create a new shader stage.
  fn new<S>(ty: Type, src: S) -> Result<Self, Self::Err>
  where
    S: AsRef<str>;
}
