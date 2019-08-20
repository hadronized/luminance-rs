use crate::attrib::{AttrError, get_field_attr_once};
use proc_macro::TokenStream;
use quote::quote;
use std::fmt;
use syn::{Attribute, DataStruct, Fields, Ident, LitBool, Type};

// accepted sub keys for the "vertex" key
const KNOWN_SUBKEYS: &[&str] = &["sem", "instanced", "normalized"];

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
    attrs.clone(),
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
      let mut indexed_vertex_attrib_descs = Vec::new();
      let mut fields_tys = Vec::new();

      // partition and generate VertexBufferDesc
      for field in named_fields.named {
        let field_ty = field.ty;
        let ident = field.ident.unwrap();

        // search for the normalized argument; if not there, we don’t normalize anything
        let normalized = get_field_attr_once(
          &ident,
          &field.attrs,
          "vertex",
          "normalized",
          KNOWN_SUBKEYS
        )
          .map(|b: LitBool| b.value)
          .or_else(|e| match e {
            AttrError::CannotFindAttribute(..) => Ok(false),
            _ => Err(e)
          })
          .map_err(StructImplError::FieldError)?;

        let vertex_attrib_desc = if normalized {
          quote!{ (<#field_ty as luminance::vertex::VertexAttrib>::VERTEX_ATTRIB_DESC).normalize() }
        } else {
          quote!{ <#field_ty as luminance::vertex::VertexAttrib>::VERTEX_ATTRIB_DESC }
        };

        let indexed_vertex_attrib_desc_q = quote!{
          luminance::vertex::VertexBufferDesc::new::<#sem_type>(
            <#field_ty as luminance::vertex::HasSemantics>::SEMANTICS,
            #instancing,
            #vertex_attrib_desc,
          )
        };

        indexed_vertex_attrib_descs.push(indexed_vertex_attrib_desc_q);
        fields_tys.push(field_ty);
      }

      let struct_name = ident;
      let impl_ = quote! {
        unsafe impl luminance::vertex::Vertex for #struct_name {
          fn vertex_desc() -> luminance::vertex::VertexDesc {
            vec![#(#indexed_vertex_attrib_descs),*]
          }
        }
      };

      Ok(impl_.into())
    }

    Fields::Unnamed(_) => Err(StructImplError::UnsupportedUnnamed),
    Fields::Unit => Err(StructImplError::UnsupportedUnit)
  }
}
