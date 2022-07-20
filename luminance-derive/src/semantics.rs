use crate::attrib::{get_field_attr_many, get_field_attr_once, AttrError};
use proc_macro::TokenStream;
use quote::quote;
use std::{error, fmt};
use syn::{Attribute, DataEnum, Ident, Type};

const KNOWN_SUBKEYS: &[&str] = &["name", "repr", "wrapper", "wrapper_attr"];

#[derive(Debug)]
pub(crate) enum SemanticsImplError {
  AttributeErrors(Vec<AttrError>),
  NoField,
}

impl SemanticsImplError {
  pub(crate) fn attribute_errors(errors: impl IntoIterator<Item = AttrError>) -> Self {
    SemanticsImplError::AttributeErrors(errors.into_iter().collect())
  }

  pub(crate) fn no_field() -> Self {
    SemanticsImplError::NoField
  }
}

impl fmt::Display for SemanticsImplError {
  fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
    match *self {
      SemanticsImplError::AttributeErrors(ref errs) => {
        for err in errs {
          err.fmt(f)?;
          writeln!(f).unwrap();
        }

        Ok(())
      }

      SemanticsImplError::NoField => f.write_str("semantics cannot be empty sets"),
    }
  }
}

impl error::Error for SemanticsImplError {
  fn source(&self) -> Option<&(dyn error::Error + 'static)> {
    match self {
      SemanticsImplError::AttributeErrors(ref ve) => ve.first().map(|x| x as &dyn error::Error),
      _ => None,
    }
  }
}

/// Get vertex semantics attributes.
///
///   (name, repr, wrapper)
fn get_vertex_sem_attribs<'a, A>(
  var_name: &Ident,
  attrs: A,
) -> Result<(Ident, Type, Type, Vec<syn::MetaList>), AttrError>
where
  A: Iterator<Item = &'a Attribute> + Clone,
{
  let sem_name =
    get_field_attr_once::<_, Ident>(var_name, attrs.clone(), "sem", "name", KNOWN_SUBKEYS)?;
  let sem_repr =
    get_field_attr_once::<_, Type>(var_name, attrs.clone(), "sem", "repr", KNOWN_SUBKEYS)?;
  let sem_wrapper =
    get_field_attr_once::<_, Type>(var_name, attrs.clone(), "sem", "wrapper", KNOWN_SUBKEYS)?;
  let sem_wrapper_attrs =
    get_field_attr_many::<_, syn::MetaList>(var_name, attrs, "sem", "wrapper_attr", KNOWN_SUBKEYS)?;

  Ok((sem_name, sem_repr, sem_wrapper, sem_wrapper_attrs))
}

pub(crate) fn generate_enum_semantics_impl(
  ident: Ident,
  enum_: DataEnum,
) -> Result<TokenStream, SemanticsImplError> {
  let fields = enum_.variants.into_iter().map(|var| {
    get_vertex_sem_attribs(&var.ident, var.attrs.iter())
      .map(|attrs| (var.ident, attrs.0, attrs.1, attrs.2, attrs.3))
  });

  let mut parse_branches = Vec::new();
  let mut name_branches = Vec::new();
  let mut field_based_gen = Vec::new();
  let mut semantics_set = Vec::new();

  let mut errors = Vec::new();

  for (index, field) in fields.enumerate() {
    match field {
      Ok(field) => {
        // parse branches
        let sem_var = field.0;
        let sem_name = field.1.to_string();
        let repr_ty_name = field.2;
        let ty_name = field.3;
        let ty_attrs = field.4;

        // dynamic branch used for parsing the semantics from a string
        parse_branches.push(quote! {
          #sem_name => Ok(#ident::#sem_var)
        });

        // name of a semantics
        name_branches.push(quote! {
          #ident::#sem_var => #sem_name
        });

        semantics_set.push(quote! {
          luminance::vertex::SemanticsDesc {
            index: #index,
            name: #sem_name.to_owned()
          }
        });

        // field-based code generation
        let field_gen = quote! {
          /// Vertex attribute type (representing #repr_ty_name).
          #[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
          #[repr(transparent)]
          #(#[#ty_attrs])*
          pub struct #ty_name {
            /// Internal representation.
            pub repr: #repr_ty_name
          }

          // give access to the underlying field
          impl std::ops::Deref for #ty_name {
            type Target = #repr_ty_name;

            fn deref(&self) -> &Self::Target {
              &self.repr
            }
          }

          impl std::ops::DerefMut for #ty_name {
            fn deref_mut(&mut self) -> &mut Self::Target {
              &mut self.repr
            }
          }

          // convert from the repr type to the vertex attrib type
          impl From<#repr_ty_name> for #ty_name {
            fn from(repr: #repr_ty_name) -> Self {
              #ty_name::new(repr)
            }
          }

          // convert from the repr type to the vertex attrib type
          impl #ty_name {
            /// Create a new vertex attribute based on its inner representation.
            pub const fn new(repr: #repr_ty_name) -> Self {
             #ty_name {
               repr
             }
            }
          }

          // get the associated semantics
          impl luminance::vertex::HasSemantics for #ty_name {
            type Sem = #ident;

            const SEMANTICS: Self::Sem = #ident::#sem_var;
          }

          // make the vertex attrib impl VertexAttrib by forwarding implementation to the repr type
          unsafe impl luminance::vertex::VertexAttrib for #ty_name {
            const VERTEX_ATTRIB_DESC: luminance::vertex::VertexAttribDesc =
              <#repr_ty_name as luminance::vertex::VertexAttrib>::VERTEX_ATTRIB_DESC;
          }
        };

        field_based_gen.push(field_gen);
      }

      Err(e) => errors.push(e),
    }
  }

  if !errors.is_empty() {
    return Err(SemanticsImplError::attribute_errors(errors));
  }

  if semantics_set.is_empty() {
    return Err(SemanticsImplError::no_field());
  }

  // output generation
  let output_gen = quote! {
    impl luminance::vertex::Semantics for #ident {
      fn index(&self) -> usize {
        *self as usize
      }

      fn name(&self) -> &'static str {
        match *self {
          #(#name_branches,)*
        }
      }

      fn semantics_set() -> Vec<luminance::vertex::SemanticsDesc> {
        vec![#(#semantics_set,)*]
      }
    }

    // easy parsing
    impl std::str::FromStr for #ident {
      type Err = ();

      fn from_str(name: &str) -> Result<Self, Self::Err> {
        match name {
          #(#parse_branches,)*
          _ => Err(())
        }
      }
    }
  };

  let output = quote! {
    #output_gen
    #(#field_based_gen)*
  };

  Ok(output.into())
}
