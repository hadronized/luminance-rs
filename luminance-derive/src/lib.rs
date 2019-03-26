extern crate proc_macro;

mod attrib;
mod semantics;
mod uniform_interface;
mod vertex;

use crate::semantics::generate_enum_semantics_impl;
use crate::uniform_interface::generate_uniform_interface_impl;
use crate::vertex::generate_vertex_impl;
use proc_macro::TokenStream;
use syn::{self, Data, DeriveInput, parse_macro_input};

#[proc_macro_derive(Vertex, attributes(vertex))]
pub fn derive_vertex(input: TokenStream) -> TokenStream {
  let di: DeriveInput = parse_macro_input!(input);

  match di.data {
    // for now, we only handle structs
    Data::Struct(struct_) => {
      match generate_vertex_impl(di.ident, di.attrs.iter(), struct_) {
        Ok(impl_) => impl_,
        Err(e) => panic!("{}", e)
      }
    }

    _ => panic!("only structs are currently supported for deriving Vertex")
  }
}

#[proc_macro_derive(Semantics, attributes(sem))]
pub fn derive_semantics(input: TokenStream) -> TokenStream {
  let di: DeriveInput = parse_macro_input!(input);

  match di.data {
    // for now, we only handle enums
    Data::Enum(enum_) => {
      match generate_enum_semantics_impl(di.ident, enum_) {
        Ok(impl_) => impl_,
        Err(e) => panic!("{}", e)
      }
    }

    _ => panic!("only enums are currently supported for deriving VertexAttribSem")
  }
}

#[proc_macro_derive(UniformInterface, attributes(uniform))]
pub fn derive_uniform_interface(input: TokenStream) -> TokenStream {
  let di: DeriveInput = parse_macro_input!(input);

  match di.data {
    // for now, we only handle structs
    Data::Struct(struct_) => {
      match generate_uniform_interface_impl(di.ident, struct_) {
        Ok(impl_) => impl_,
        Err(e) => panic!("{}", e)
      }
    }

    _ => panic!("only structs are currently supported for deriving UniformInterface")
  }
}
