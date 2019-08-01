use crate::attrib::{AttrError, get_field_attr_once};
use proc_macro::TokenStream;
use quote::quote;
use std::fmt;
use syn::{Attribute, DataEnum, Ident, Type};

const KNOWN_SUBKEYS: &[&str] = &["name", "repr", "wrapper"];

#[derive(Debug)]
pub(crate) enum SemanticsImplError {
  AttributeErrors(Vec<AttrError>),
}

impl fmt::Display for SemanticsImplError {
  fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
    match *self {
      SemanticsImplError::AttributeErrors(ref errs) => {
        for err in errs {
          err.fmt(f)?;
          writeln!(f, "").unwrap();
        }

        Ok(())
      }
    }
  }
}

/// Get vertex semantics attributes.
///
///   (name, repr, wrapper)
fn get_vertex_sem_attribs<'a, A>(
  var_name: &Ident,
  attrs: A
) -> Result<(Ident, Type, Type), AttrError>
where A: Iterator<Item = &'a Attribute> + Clone {
  let sem_name = get_field_attr_once::<_, Ident>(var_name, attrs.clone(), "sem", "name", KNOWN_SUBKEYS)?;
  let sem_repr = get_field_attr_once::<_, Type>(var_name, attrs.clone(), "sem", "repr", KNOWN_SUBKEYS)?;
  let sem_wrapper = get_field_attr_once::<_, Type>(var_name, attrs, "sem", "wrapper", KNOWN_SUBKEYS)?;

  Ok((sem_name, sem_repr, sem_wrapper))
}

pub(crate) fn generate_enum_semantics_impl(
  ident: Ident,
  enum_: DataEnum
) -> Result<TokenStream, SemanticsImplError> {
  let fields = enum_.variants.into_iter().map(|var| {
    get_vertex_sem_attribs(&var.ident, var.attrs.iter()).map(|attrs| {
      (var.ident, attrs.0, attrs.1, attrs.2)
    })
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

        // dynamic branch used for parsing the semantics from a string
        parse_branches.push(quote!{
          #sem_name => Ok(#ident::#sem_var)
        });

        // name of a semantics
        name_branches.push(quote!{
          #ident::#sem_var => #sem_name
        });

        semantics_set.push(quote!{
          luminance::vertex::SemanticsDesc {
            index: #index,
            name: #sem_name.to_owned()
          }
        });

        // field-based code generation
        let field_gen = quote!{
          // vertex attrib type
          #[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
          pub struct #ty_name {
            repr: #repr_ty_name
          }

          // convert from the repr type to the vertex attrib type
          impl From<#repr_ty_name> for #ty_name {
            fn from(repr: #repr_ty_name) -> Self {
              #ty_name::new(repr)
            }
          }

          // convert from the repr type to the vertex attrib type
          impl #ty_name {
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

      Err(e) => errors.push(e)
    }
  }

  if !errors.is_empty() {
    return Err(SemanticsImplError::AttributeErrors(errors));
  }

  // output generation
  let output_gen = quote!{
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

  let output = quote!{
    #output_gen
    #(#field_based_gen)*
  };

  Ok(output.into())
}

