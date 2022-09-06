use crate::attrib::{get_field_attr_once, AttrError};
use proc_macro::TokenStream;
use quote::quote;
use std::error;
use std::fmt;
use syn::{Attribute, DataStruct, Field, Fields, Ident, LitBool};

// accepted sub keys for the "vertex" key
const KNOWN_SUBKEYS: &[&str] = &["instanced", "normalized", "namespace"];

#[derive(Debug)]
pub(crate) enum StructImplError {
  FieldError(AttrError),
  UnsupportedUnnamed,
  UnsupportedUnit,
}

impl fmt::Display for StructImplError {
  fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
    match self {
      StructImplError::FieldError(ref e) => write!(f, "error with vertex attribute field; {}", e),
      StructImplError::UnsupportedUnnamed => f.write_str("unsupported unnamed fields in struct"),
      StructImplError::UnsupportedUnit => f.write_str("unsupported unit struct"),
    }
  }
}

impl error::Error for StructImplError {
  fn source(&self) -> Option<&(dyn error::Error + 'static)> {
    match self {
      StructImplError::FieldError(e) => Some(e),
      _ => None,
    }
  }
}

/// Generate the Vertex impl for a struct.
pub(crate) fn generate_vertex_impl<'a, A>(
  struct_ident: Ident,
  attrs: A,
  struct_: DataStruct,
) -> Result<TokenStream, StructImplError>
where
  A: Iterator<Item = &'a Attribute> + Clone,
{
  let instancing = get_instancing(&struct_ident, attrs.clone())?;
  let namespace = get_namespace(&struct_ident, attrs.clone())?;

  match struct_.fields {
    Fields::Named(named_fields) => {
      let per_field = named_fields
        .named
        .iter()
        .enumerate()
        .map(|(rank, field)| {
          let field_ident = field.ident.as_ref().unwrap().to_string();
          let field_ty = &field.ty;

          let vertex_attrib_desc = field_vertex_attrib_desc(
            &field,
            field.ident.as_ref().unwrap(),
            &instancing,
            &namespace,
          )?;

          let deinterleave_impl = quote! {
            impl luminance::vertex::HasField<#field_ident> for #struct_ident {
              type FieldType = #field_ty;
            }

            impl luminance::vertex::Deinterleave<#field_ident> for #struct_ident {
              const RANK: usize = #rank;
            }
          };

          let has_field_trait_bound = quote! {
            luminance::vertex::HasField<#field_ident, FieldType = #field_ty>
          };

          Ok((vertex_attrib_desc, deinterleave_impl, has_field_trait_bound))
        })
        .collect::<Result<Vec<_>, _>>()?;

      let vertex_attrib_descs = per_field.iter().map(|pf| &pf.0);
      let deinterleave_impls = per_field.iter().map(|pf| &pf.1);
      let has_field_trait_bounds = per_field.iter().map(|pf| &pf.2);
      let output = quote! {
        // Vertex impl
        unsafe impl luminance::vertex::Vertex for #struct_ident {
          fn vertex_desc() -> luminance::vertex::VertexDesc {
            vec![#(#vertex_attrib_descs),*]
          }
        }

        #(#deinterleave_impls)*

        impl<V> luminance::vertex::CompatibleVertex<V> for #struct_ident
        where V: luminance::vertex::Vertex + #(#has_field_trait_bounds)+*
        {
        }
      };

      Ok(output.into())
    }

    Fields::Unnamed(..) => Err(StructImplError::UnsupportedUnnamed),

    Fields::Unit => Err(StructImplError::UnsupportedUnit),
  }
}

fn field_vertex_attrib_desc(
  field: &Field,
  ident: &Ident,
  instancing: &proc_macro2::TokenStream,
  namespace: &proc_macro2::TokenStream,
) -> Result<proc_macro2::TokenStream, StructImplError> {
  // search for the normalized argument; if not there, we don’t normalize anything
  let normalized = get_field_attr_once(&ident, &field.attrs, "vertex", "normalized", KNOWN_SUBKEYS)
    .map(|b: LitBool| b.value)
    .or_else(|e| match e {
      AttrError::CannotFindAttribute(..) => Ok(false),
      _ => Err(e),
    })
    .map_err(StructImplError::FieldError)?;

  let field_ty = &field.ty;
  let field_name = field.ident.as_ref().unwrap().to_string();

  let vertex_attrib_desc = if normalized {
    quote! { (<#field_ty as luminance::vertex::VertexAttrib>::VERTEX_ATTRIB_DESC).normalize() }
  } else {
    quote! { <#field_ty as luminance::vertex::VertexAttrib>::VERTEX_ATTRIB_DESC }
  };

  let q = quote! {
    luminance::vertex::VertexBufferDesc::new(
      <#namespace as luminance::named_index::NamedIndex<#field_name>>::INDEX,
      #field_name,
      #instancing,
      #vertex_attrib_desc,
    )
  };

  Ok(q)
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

fn get_namespace<'a, A>(
  ident: &Ident,
  attrs: A,
) -> Result<proc_macro2::TokenStream, StructImplError>
where
  A: IntoIterator<Item = &'a Attribute>,
{
  get_field_attr_once(ident, attrs, "vertex", "namespace", KNOWN_SUBKEYS)
    .map(|namespace: Ident| {
      quote! { #namespace }
    })
    .map_err(StructImplError::FieldError)
}
