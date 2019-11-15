use crate::attrib::{get_field_attr_once, AttrError};
use proc_macro::TokenStream;
use quote::{format_ident, quote};
use std::fmt;
use syn::{Attribute, DataStruct, Field, Fields, Ident, LitBool, Type};

// accepted sub keys for the "vertex" key
const KNOWN_SUBKEYS: &[&str] = &["sem", "instanced", "normalized"];

#[derive(Debug)]
pub(crate) enum StructImplError {
  SemanticsError(AttrError),
  FieldError(AttrError),
  UnsupportedUnit,
}

impl fmt::Display for StructImplError {
  fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
    match *self {
      StructImplError::SemanticsError(ref e) => write!(f, "error with semantics type; {}", e),
      StructImplError::FieldError(ref e) => write!(f, "error with vertex attribute field; {}", e),
      StructImplError::UnsupportedUnit => f.write_str("unsupported unit struct"),
    }
  }
}

/// Generate the Vertex impl for a struct.
pub(crate) fn generate_vertex_impl<'a, A>(
  ident: Ident,
  attrs: A,
  struct_: DataStruct,
) -> Result<TokenStream, StructImplError>
where
  A: Iterator<Item = &'a Attribute> + Clone,
{
  // search the semantics name
  let sem_type: Type = get_field_attr_once(&ident, attrs.clone(), "vertex", "sem", KNOWN_SUBKEYS)
    .map_err(StructImplError::SemanticsError)?;

  let instancing = get_instancing(&ident, attrs.clone())?;

  match struct_.fields {
    Fields::Unnamed(unnamed_fields) => {
      let mut indexed_vertex_attrib_descs = Vec::new();
      let mut fields_types = Vec::new();

      for (i, field) in unnamed_fields.unnamed.into_iter().enumerate() {
        let field_ident = format_ident!("field_{}", i);

        process_field(
          &field,
          field_ident,
          &sem_type,
          &instancing,
          &mut indexed_vertex_attrib_descs,
          &mut fields_types,
          None,
        )?;
      }

      let output = process_struct(ident, indexed_vertex_attrib_descs, Vec::new(), fields_types);
      Ok(output.into())
    }

    Fields::Named(named_fields) => {
      let mut indexed_vertex_attrib_descs = Vec::new();
      let mut fields_types = Vec::new();
      let mut fields_names = Vec::new();

      for field in named_fields.named {
        let field_ident = field.ident.clone().unwrap();

        process_field(
          &field,
          field_ident,
          &sem_type,
          &instancing,
          &mut indexed_vertex_attrib_descs,
          &mut fields_types,
          &mut fields_names,
        )?;
      }

      let output = process_struct(
        ident,
        indexed_vertex_attrib_descs,
        fields_names,
        fields_types,
      );
      Ok(output.into())
    }

    Fields::Unit => Err(StructImplError::UnsupportedUnit),
  }
}

fn process_field<'a, FN>(
  field: &Field,
  ident: Ident,
  sem_type: &Type,
  instancing: &proc_macro2::TokenStream,
  indexed_vertex_attrib_descs: &mut Vec<proc_macro2::TokenStream>,
  fields_types: &mut Vec<Type>,
  fields_names: FN,
) -> Result<(), StructImplError>
where
  FN: Into<Option<&'a mut Vec<Ident>>>,
{
  // search for the normalized argument; if not there, we don’t normalize anything
  let normalized = get_field_attr_once(&ident, &field.attrs, "vertex", "normalized", KNOWN_SUBKEYS)
    .map(|b: LitBool| b.value)
    .or_else(|e| match e {
      AttrError::CannotFindAttribute(..) => Ok(false),
      _ => Err(e),
    })
    .map_err(StructImplError::FieldError)?;

  let field_ty = &field.ty;
  let vertex_attrib_desc = if normalized {
    quote! { (<#field_ty as luminance::vertex::VertexAttrib>::VERTEX_ATTRIB_DESC).normalize() }
  } else {
    quote! { <#field_ty as luminance::vertex::VertexAttrib>::VERTEX_ATTRIB_DESC }
  };

  let indexed_vertex_attrib_desc_q = quote! {
    luminance::vertex::VertexBufferDesc::new::<#sem_type>(
      <#field_ty as luminance::vertex::HasSemantics>::SEMANTICS,
      #instancing,
      #vertex_attrib_desc,
    )
  };

  indexed_vertex_attrib_descs.push(indexed_vertex_attrib_desc_q);
  fields_types.push(field_ty.clone());

  if let Some(fields_names) = fields_names.into() {
    fields_names.push(ident);
  }

  Ok(())
}

/// Process the output struct.
///
/// If fields_names is empty, it is assumed to be a struct-tuple.
fn process_struct(
  struct_name: Ident,
  indexed_vertex_attrib_descs: Vec<proc_macro2::TokenStream>,
  fields_names: Vec<Ident>,
  fields_types: Vec<Type>,
) -> proc_macro2::TokenStream {
  let fn_new = if fields_names.is_empty() {
    // struct tuple
    let i: Vec<_> = (0..fields_types.len())
      .map(|i| format_ident!("field_{}", i))
      .collect();

    quote! {
      impl #struct_name {
        pub const fn new(#(#i : #fields_types),*) -> Self {
          #struct_name ( #(#i),* )
        }
      }
    }
  } else {
    quote! {
      impl #struct_name {
        pub const fn new(#(#fields_names : #fields_types),*) -> Self {
          #struct_name { #(#fields_names),* }
        }
      }
    }
  };

  quote! {
    // Vertex impl
    unsafe impl luminance::vertex::Vertex for #struct_name {
      fn vertex_desc() -> luminance::vertex::VertexDesc {
        vec![#(#indexed_vertex_attrib_descs),*]
      }
    }

    // helper function for the generate type
    #fn_new
  }
}

fn get_instancing<'a, A>(
  ident: &Ident,
  attrs: A,
) -> Result<proc_macro2::TokenStream, StructImplError>
where
  A: IntoIterator<Item = &'a Attribute>,
{
  // search for the instancing argument; if not there, we don’t use vertex instancing
  get_field_attr_once(&ident, attrs, "vertex", "instanced", KNOWN_SUBKEYS)
    .map(|b: LitBool| {
      if b.value {
        quote! { luminance::vertex::VertexInstancing::On }
      } else {
        quote! { luminance::vertex::VertexInstancing::Off }
      }
    })
    .or_else(|e| match e {
      AttrError::CannotFindAttribute(..) => Ok(quote! { luminance::vertex::VertexInstancing::Off }),

      _ => Err(e),
    })
    .map_err(StructImplError::FieldError)
}
