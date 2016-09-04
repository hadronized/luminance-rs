//! Vertex formats and associated types and functions.
//!
//! A vertex is a type representing a point. It’s common to find vertex position, normals, colors or
//! even texture coordinates. However, you’re free to use whichever type you want.
//! Nevertheless, you’re limited to a range of types and dimensions. See `Type` and
//! `Dim` for further details.
//!
//! # `Vertex`
//!
//! ## Rules
//!
//! To be able to use a type as a vertex, you have to implement the `Vertex` trait. That trait
//! represents a mapping between your type and `VertexFormat`. A `VertexFormat` gives runtime hints
//! about your type and restricts the supported type. If you cannot map your type to `VertexFormat`,
//! that means you cannot use it as a `Vertex`.
//!
//! The rule is that your type should have a static size greater than 0 and less than or equal to 4.
//! It should also be either integral, unsigned, floating or boolean. If your type is a complex one
//! – for instance a `struct` – you have to recursively apply that rule to all its fields.
//! For instance, the tuple `(i32, bool)` implements `Vertex` by providing an implementation using
//! the ones of `i32` and `bool`.
//!
//! ## Components list
//!
//! As mentionned above, you can use tuples and structs as `Vertex`. If you look at the definition
//! of `VertexFormat`, you’ll notice that it’s a `Vec<VertexComponentFormat>`. That means simple
//! and primary types map to unit vectors – i.e. their size is 1 – but tuples and structs need
//! several `VertexComponentFormat`s to be represented, hence vectors with sizes greater than 1.
//!
//! # Generic implementation
//!
//! You have `Vertex` implementations for all the primary types that can be mapped to
//! `VertexFormat`. However, as it’s not possible to automatically implement `Vertex` for your
//! structure (yet?), a type is provided to help you design your vertex type so that you’re
//! automatically provided with a `Vertex` implementation if you use `Chain`.
//!
//! `Chain` is a special type used to represent static list of types. With that in hand, you can
//! easily create `Vertex` types and start using them without even implementing `Vertex`, as long as
//! you use `Vertex` types. Feel free to dig in the `Chain` documentation for further details.

use chain::Chain;
use std::vec::Vec;

/// A `VertexFormat` is a list of `VertexComponentFormat`s.
pub type VertexFormat = Vec<VertexComponentFormat>;

/// Retrieve the number of components in a `VertexFormat`.
pub fn vertex_format_size(vf: &[VertexComponentFormat]) -> usize {
  vf.len()
}

/// A `VertexComponentFormat` gives hints about:
///
/// - the type of the component (`Type`);
/// - the dimension of the component (`Dim`).
#[derive(Clone, Copy, Debug)]
pub struct VertexComponentFormat {
  pub component_type: Type,
  pub dim: Dim
}

/// Possible type of vertex components.
#[derive(Clone, Copy, Debug)]
pub enum Type {
  Integral,
  Unsigned,
  Floating,
  Boolean
}

/// Possible dimension of vertex components.
#[derive(Clone, Copy, Debug)]
pub enum Dim {
  Dim1,
  Dim2,
  Dim3,
  Dim4
}

/// A type that can be used as a `Vertex` has to implement that trait – it must provide a mapping
/// to `VertexFormat`.
///
/// If you’re not sure on how to implement that or if you want to use automatic types, feel free
/// to use the primary supported types and `Chain` or tuples.
pub trait Vertex {
  fn vertex_format() -> VertexFormat;
}

impl Vertex for () {
  fn vertex_format() -> VertexFormat {
    Vec::new()
  }
}

impl Vertex for i32 {
  fn vertex_format() -> VertexFormat {
    vec![ VertexComponentFormat { component_type: Type::Integral, dim: Dim::Dim1 } ]
  }
}

impl Vertex for [i32; 1] {
  fn vertex_format() -> VertexFormat {
    vec![ VertexComponentFormat { component_type: Type::Integral, dim: Dim::Dim1 } ]
  }
}

impl Vertex for [i32; 2] {
  fn vertex_format() -> VertexFormat {
    vec![ VertexComponentFormat { component_type: Type::Integral, dim: Dim::Dim2 } ]
  }
}

impl Vertex for [i32; 3] {
  fn vertex_format() -> VertexFormat {
    vec![ VertexComponentFormat { component_type: Type::Integral, dim: Dim::Dim3 } ]
  }
}

impl Vertex for [i32; 4] {
  fn vertex_format() -> VertexFormat {
    vec![ VertexComponentFormat { component_type: Type::Integral, dim: Dim::Dim4 } ]
  }
}

impl Vertex for u32 {
  fn vertex_format() -> VertexFormat {
    vec![ VertexComponentFormat { component_type: Type::Unsigned, dim: Dim::Dim1 } ]
  }
}

impl Vertex for [u32; 1] {
  fn vertex_format() -> VertexFormat {
    vec![ VertexComponentFormat { component_type: Type::Unsigned, dim: Dim::Dim1 } ]
  }
}

impl Vertex for [u32; 2] {
  fn vertex_format() -> VertexFormat {
    vec![ VertexComponentFormat { component_type: Type::Unsigned, dim: Dim::Dim2 } ]
  }
}

impl Vertex for [u32; 3] {
  fn vertex_format() -> VertexFormat {
    vec![ VertexComponentFormat { component_type: Type::Unsigned, dim: Dim::Dim3 } ]
  }
}

impl Vertex for [u32; 4] {
  fn vertex_format() -> VertexFormat {
    vec![ VertexComponentFormat { component_type: Type::Unsigned, dim: Dim::Dim4 } ]
  }
}

impl Vertex for f32 {
  fn vertex_format() -> VertexFormat {
    vec![ VertexComponentFormat { component_type: Type::Floating, dim: Dim::Dim1 } ]
  }
}

impl Vertex for [f32; 1] {
  fn vertex_format() -> VertexFormat {
    vec![ VertexComponentFormat { component_type: Type::Floating, dim: Dim::Dim1 } ]
  }
}

impl Vertex for [f32; 2] {
  fn vertex_format() -> VertexFormat {
    vec![ VertexComponentFormat { component_type: Type::Floating, dim: Dim::Dim2 } ]
  }
}

impl Vertex for [f32; 3] {
  fn vertex_format() -> VertexFormat {
    vec![ VertexComponentFormat { component_type: Type::Floating, dim: Dim::Dim3 } ]
  }
}

impl Vertex for [f32; 4] {
  fn vertex_format() -> VertexFormat {
    vec![ VertexComponentFormat { component_type: Type::Floating, dim: Dim::Dim4 } ]
  }
}

impl Vertex for bool {
  fn vertex_format() -> VertexFormat {
    vec![ VertexComponentFormat { component_type: Type::Boolean, dim: Dim::Dim1 } ]
  }
}

impl Vertex for [bool; 1] {
  fn vertex_format() -> VertexFormat {
    vec![ VertexComponentFormat { component_type: Type::Boolean, dim: Dim::Dim1 } ]
  }
}

impl Vertex for [bool; 2] {
  fn vertex_format() -> VertexFormat {
    vec![ VertexComponentFormat { component_type: Type::Boolean, dim: Dim::Dim2 } ]
  }
}

impl Vertex for [bool; 3] {
  fn vertex_format() -> VertexFormat {
    vec![ VertexComponentFormat { component_type: Type::Boolean, dim: Dim::Dim3 } ]
  }
}

impl Vertex for [bool; 4] {
  fn vertex_format() -> VertexFormat {
    vec![ VertexComponentFormat { component_type: Type::Boolean, dim: Dim::Dim4 } ]
  }
}

impl<A, B> Vertex for Chain<A, B> where A: Vertex, B: Vertex {
  fn vertex_format() -> VertexFormat {
    let mut t = A::vertex_format();
    t.extend(B::vertex_format());
    t
  }
}

impl<A, B> Vertex for (A, B) where A: Vertex, B: Vertex {
  fn vertex_format() -> VertexFormat {
    Chain::<A, B>::vertex_format()
  }
}

impl<A, B, C> Vertex for (A, B, C) where A: Vertex, B: Vertex, C: Vertex {
  fn vertex_format() -> VertexFormat {
    Chain::<A, Chain<B, C>>::vertex_format()
  }
}

impl<A, B, C, D> Vertex for (A, B, C, D) where A: Vertex, B: Vertex, C: Vertex, D: Vertex {
  fn vertex_format() -> VertexFormat {
    Chain::<A, Chain<B, Chain<C, D>>>::vertex_format()
  }
}

impl<A, B, C, D, E> Vertex for (A, B, C, D, E) where A: Vertex, B: Vertex, C: Vertex, D: Vertex, E: Vertex {
  fn vertex_format() -> VertexFormat {
    Chain::<A, Chain<B, Chain<C, Chain<D, E>>>>::vertex_format()
  }
}

impl<A, B, C, D, E, F> Vertex for (A, B, C, D, E, F) where A: Vertex, B: Vertex, C: Vertex, D: Vertex, E: Vertex, F: Vertex {
  fn vertex_format() -> VertexFormat {
    Chain::<A, Chain<B, Chain<C, Chain<D, Chain<E, F>>>>>::vertex_format()
  }
}

impl<A, B, C, D, E, F, G> Vertex for (A, B, C, D, E, F, G) where A: Vertex, B: Vertex, C: Vertex, D: Vertex, E: Vertex, F: Vertex, G: Vertex {
  fn vertex_format() -> VertexFormat {
    Chain::<A, Chain<B, Chain<C, Chain<D, Chain<E, Chain<F, G>>>>>>::vertex_format()
  }
}

impl<A, B, C, D, E, F, G, H> Vertex for (A, B, C, D, E, F, G, H) where A: Vertex, B: Vertex, C: Vertex, D: Vertex, E: Vertex, F: Vertex, G: Vertex, H: Vertex {
  fn vertex_format() -> VertexFormat {
    Chain::<A, Chain<B, Chain<C, Chain<D, Chain<E, Chain<F, Chain<G, H>>>>>>>::vertex_format()
  }
}

impl<A, B, C, D, E, F, G, H, I> Vertex for (A, B, C, D, E, F, G, H, I) where A: Vertex, B: Vertex, C: Vertex, D: Vertex, E: Vertex, F: Vertex, G: Vertex, H: Vertex, I: Vertex {
  fn vertex_format() -> VertexFormat {
    Chain::<A, Chain<B, Chain<C, Chain<D, Chain<E, Chain<F, Chain<G, Chain<H, I>>>>>>>>::vertex_format()
  }
}

impl<A, B, C, D, E, F, G, H, I, J> Vertex for (A, B, C, D, E, F, G, H, I, J) where A: Vertex, B: Vertex, C: Vertex, D: Vertex, E: Vertex, F: Vertex, G: Vertex, H: Vertex, I: Vertex, J: Vertex {
  fn vertex_format() -> VertexFormat {
    Chain::<A, Chain<B, Chain<C, Chain<D, Chain<E, Chain<F, Chain<G, Chain<H, Chain<I, J>>>>>>>>>::vertex_format()
  }
}
