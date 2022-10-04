#![feature(proc_macro_diagnostic)]

mod attrib;
mod render_slots;
mod uniforms;
mod vertex;

use crate::vertex::generate_vertex_impl;
use crate::{render_slots::impl_render_slots, uniforms::generate_uniforms_impl};
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

#[proc_macro_derive(Environment, attributes(env))]
pub fn derive_uniforms(input: TokenStream) -> TokenStream {
  let di: DeriveInput = parse_macro_input!(input);

  match di.data {
    // for now, we only handle structs
    Data::Struct(struct_) => match generate_uniforms_impl(di.ident, struct_) {
      Ok(impl_) => impl_,
      Err(e) => panic!("{}", e),
    },

    _ => panic!("only structs are currently supported for deriving Uniforms"),
  }
}

#[proc_macro_derive(RenderSlots, attributes(slot))]
pub fn derive_render_slots(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
  let item: DeriveInput = parse_macro_input!(input);
  // panic!("{}", impl_render_slots(item).to_string());
  impl_render_slots(item).into()
}
