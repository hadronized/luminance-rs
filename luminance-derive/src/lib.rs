//! Derive procedural macros for [luminance].
//!
//! This crate exports several macros used to ease development with [luminance]. You are
//! strongly advised to read the documentation of [luminance] in the first place.
//!
//! # `Vertex`
//!
//! This macro allows to derive the [`Vertex`] trait for a custom `struct` type.
//!
//! [See the full documentation here](https://docs.rs/luminance/latest/luminance/#vertex)
//!
//! # `UniformInterface`
//!
//! This macro allows to derive the [`UniformInterface`] trait for a custom `struct` type.
//!
//! [See the full documentation here](https://docs.rs/luminance/latest/luminance/#uniform-interface)
//!
//! [luminance]: https://crates.io/crates/luminance
//! [`Vertex`]: https://docs.rs/luminance/latest/luminance/vertex/trait.Vertex.html

mod attrib;
mod uniform_interface;
mod vertex;

use crate::uniform_interface::generate_uniform_interface_impl;
use crate::vertex::generate_vertex_impl;
use proc_macro::TokenStream;
use syn::{self, parse_macro_input, Data, DeriveInput};

#[proc_macro_derive(Vertex, attributes(vertex))]
pub fn derive_vertex(input: TokenStream) -> TokenStream {
  let di: DeriveInput = parse_macro_input!(input);

  match di.data {
    // for now, we only handle structs
    Data::Struct(struct_) => match generate_vertex_impl(di.ident, di.attrs.iter(), struct_) {
      Ok(impl_) => impl_,
      Err(e) => panic!("{}", e),
    },

    _ => panic!("only structs are currently supported for deriving Vertex"),
  }
}

#[proc_macro_derive(UniformInterface, attributes(uniform))]
pub fn derive_uniform_interface(input: TokenStream) -> TokenStream {
  let di: DeriveInput = parse_macro_input!(input);

  match di.data {
    // for now, we only handle structs
    Data::Struct(struct_) => match generate_uniform_interface_impl(di.ident, struct_) {
      Ok(impl_) => impl_,
      Err(e) => panic!("{}", e),
    },

    _ => panic!("only structs are currently supported for deriving UniformInterface"),
  }
}
