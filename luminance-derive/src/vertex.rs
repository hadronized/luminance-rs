//! The `Vertex` derive proc-macro.
//!
//! That proc-macro allows you to create custom vertex types easily without having to care about
//! implementing the required traits for your types to be usable with the rest of the crate.
//!
//! # The `Vertex` trait
//!
//! The [`Vertex`] trait must be implemented if you want to use a type as vertex (passed-in via
//! slices to [`Tess`]). Either you can decide to implement it on your own, or you could just let
//! this crate do the job for you.
//!
//! > Important: the [`Vertex`] trait is `unsafe`, which means that all of its implementors must be
//! > as well. This is due to the fact that vertex formats include information about raw-level
//! > GPU memory and a bad implementation can have undefined behaviors.
//!
//! # Deriving `Vertex`
//!
//! You can derive the [`Vertex`] trait if your type follows these conditions:
//!
//!   - It must be a `struct` with named fields. This is just a temporary limitation that will get
//!     dropped as soon as the crate is stable enough.
//!   - Its fields must have a type that implements [`VertexAttrib`]. This is mandatory so that the
//!     backend knows enough about the types used in the structure to correctly align memory, pick
//!     the right types, etc.
//!   - Its fields must have a type that implements [`HasSemantics`] as well. This trait is just a
//!     type family that associates a single constant (i.e. the semantics) that the vertex attribute
//!     uses.
//!
//! Once all those requirements are met, you can derive [`Vertex`] pretty easily.
//!
//! > Note: feel free to look at the [`Semantics`] proc-macro as well, that provides a way
//! > to generate semantics types in order to completely both implement [`Semantics`] for an
//! > `enum` of your choice, but also generate *field* types you can use when defining your vertex
//! > type.
//!
//! ## Syntax
//!
//! The syntax is the following:
//!
//! ```rust
//! # use luminance_derive::{Vertex, Semantics};
//!
//! // visit the Semantics proc-macro documentation for further details
//! #[derive(Clone, Copy, Debug, PartialEq, Semantics)]
//! pub enum Semantics {
//!   #[sem(name = "position", repr = "[f32; 3]", type_name = "VertexPosition")]
//!   Position,
//!   #[sem(name = "color", repr = "[f32; 4]", type_name = "VertexColor")]
//!   Color
//! }
//!
//! #[derive(Clone, Copy, Debug, PartialEq, Vertex)] // just add Vertex to the list of derived traits
//! #[vertex(sem = "Semantics")] // specify the semantics to use for this type
//! struct MyVertex {
//!   position: VertexPosition,
//!   color: VertexColor
//! }
//! ```
//!
//! > Note: the `Semantics` enum must be public because of the implementation of [`HasSemantics`]
//! > trait.
//!
//! Besides the `Semantics`-related code, this will:
//!
//!   - Create a type called `MyVertex`, a struct that will hold a single vertex.
//!   - Implement `Vertex for MyVertex`.
//!
//! The proc-macro also supports an optional `#[vertex(instanced = "<bool>")]` struct attribute.
//! This attribute allows you to specify whether the fields are to be instanced or not. For more
//! about that, have a look at [`VertexInstancing`].

use crate::attrib::{AttrError, get_field_attr_once};
use proc_macro::TokenStream;
use quote::quote;
use std::fmt;
use syn::{Attribute, DataStruct, Fields, Ident, LitBool, Type};

// accepted sub keys for the "vertex" key
const KNOWN_SUBKEYS: &[&str] = &["sem", "instanced"];

#[derive(Debug)]
pub(crate) enum StructImplError {
  SemanticsError(AttrError),
  FieldError(AttrError),
  UnsupportedUnnamed,
  UnsupportedUnit,
}

impl fmt::Display for StructImplError {
  fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
    match *self {
      StructImplError::SemanticsError(ref e) =>
        write!(f, "error with semantics type; {}", e),
      StructImplError::FieldError(ref e) =>
        write!(f, "error with vertex attribute field; {}", e),
      StructImplError::UnsupportedUnnamed => f.write_str("unsupported unnamed fields"),
      StructImplError::UnsupportedUnit => f.write_str("unsupported unit struct"),
    }
  }
}

/// Generate the Vertex impl for a struct.
pub(crate) fn generate_vertex_impl<'a, A>(
  ident: Ident,
  attrs: A,
  struct_: DataStruct
) -> Result<TokenStream, StructImplError>
where A: Iterator<Item = &'a Attribute> + Clone {
  // search the semantics name
  let sem_type: Type = get_field_attr_once(
    &ident,
    attrs.clone(),
    "vertex",
    "sem",
    KNOWN_SUBKEYS
  ).map_err(StructImplError::SemanticsError)?;

  // search for the instancing argument; if not there, we don’t use vertex instancing
  let instancing = get_field_attr_once(
    &ident,
    attrs,
    "vertex",
    "instanced",
    KNOWN_SUBKEYS
  ).map(|b: LitBool| {
      if b.value {
        quote! { luminance::vertex::VertexInstancing::On }
      } else {
        quote! { luminance::vertex::VertexInstancing::Off }
      }
  }).or_else(|e| match e {
    AttrError::CannotFindAttribute(..) => {
      Ok(quote! { luminance::vertex::VertexInstancing::Off })
    }

    _ => Err(e)
  }).map_err(StructImplError::FieldError)?;

  match struct_.fields {
    Fields::Named(named_fields) => {
      let mut indexed_vertex_attrib_fmts = Vec::new();
      let mut fields_tys = Vec::new();

      // partition and generate VertexBufferDesc
      for field in named_fields.named {
        let field_ty = field.ty;
        let indexed_vertex_attrib_fmt_q = quote!{
          luminance::vertex::VertexBufferDesc::new::<#sem_type>(
            <#field_ty as luminance::vertex::HasSemantics>::SEMANTICS,
            #instancing,
            <#field_ty as luminance::vertex::VertexAttrib>::VERTEX_ATTRIB_DESC
          )
        };

        indexed_vertex_attrib_fmts.push(indexed_vertex_attrib_fmt_q);
        fields_tys.push(field_ty);
      }

      // indexed_vertex_attrib_fmts contains the exhaustive list of the indexed vertex attribute
      // formats needed to implement the Vertex trait
      let struct_name = ident;
      let impl_ = quote! {
        unsafe impl luminance::vertex::Vertex for #struct_name {
          fn vertex_fmt() -> luminance::vertex::VertexDesc {
            vec![#(#indexed_vertex_attrib_fmts),*]
          }
        }
      };

      Ok(impl_.into())
    }

    Fields::Unnamed(_) => Err(StructImplError::UnsupportedUnnamed),
    Fields::Unit => Err(StructImplError::UnsupportedUnit)
  }
}
