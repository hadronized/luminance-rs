#![feature(proc_macro_diagnostic)]

mod attrib;
mod render_slots;
mod uniform_interface;
mod vertex;

use crate::vertex::generate_vertex_impl;
use crate::{render_slots::impl_render_slots, uniform_interface::generate_uniform_interface_impl};
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

#[proc_macro_derive(RenderSlots)]
pub fn derive_render_slots(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
  let item: DeriveInput = parse_macro_input!(input);
  impl_render_slots(item).into()
}
