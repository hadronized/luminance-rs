use crate::attrib::{AttrError, get_field_attr_once, get_field_flag_once};
use proc_macro::TokenStream;
use quote::quote;
use std::fmt;
use syn::{DataStruct, Fields, Ident};

// accepted sub keys for the "vertex" key
const KNOWN_SUBKEYS: &[&str] = &["name", "unbound"];

#[derive(Debug)]
pub(crate) enum DeriveUniformInterfaceError {
  UnsupportedUnnamed,
  UnsupportedUnit,
  UnboundError(AttrError),
  NameError(AttrError),
}

impl fmt::Display for DeriveUniformInterfaceError {
  fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
    match *self {
      DeriveUniformInterfaceError::UnsupportedUnnamed => f.write_str("unsupported unnamed fields"),
      DeriveUniformInterfaceError::UnsupportedUnit => f.write_str("unsupported unit struct"),
      DeriveUniformInterfaceError::UnboundError(ref e) => write!(f, "unbound error: {}", e),
      DeriveUniformInterfaceError::NameError(ref e) => write!(f, "name error: {}", e),
    }
  }
}

pub(crate) fn generate_uniform_interface_impl<'a>(
  ident: Ident,
  struct_: DataStruct
) -> Result<TokenStream, DeriveUniformInterfaceError> {
  match struct_.fields {
    Fields::Named(named_fields) => {
      // field declarations; used to declare fields to be mapped while building the uniform
      // interface
      let mut field_decls = Vec::new();
      // collect field names to return the uniform interface with the shortcut syntax
      let mut field_names = Vec::new();

      for field in named_fields.named {
        let field_ident = field.ident.unwrap();
        let unbound = get_field_flag_once(
          &ident,
          field.attrs.iter(),
          "uniform",
          "unbound",
          KNOWN_SUBKEYS
        ).map_err(DeriveUniformInterfaceError::UnboundError)?;
        let name = get_field_attr_once(
          &ident,
          field.attrs.iter(),
          "uniform",
          "name",
          KNOWN_SUBKEYS
        ).map(|ident: Ident| {
          ident.to_string()
        }).or_else(|e| match e {
          AttrError::CannotFindAttribute(..) => {
            Ok(field_ident.to_string())
          }

          _ => Err(e)
        }).map_err(DeriveUniformInterfaceError::NameError)?;

        // the build call is the code that gets a uniform and possibly fails if bound; also handles
        // renaming
        let build_call = if unbound {
          quote!{
            builder.ask_unbound(#name)
          }
        } else {
          quote!{
            builder.ask(#name).map_err(luminance::shader::program::ProgramError::UniformWarning)?
          }
        };

        field_names.push(field_ident.clone());
        field_decls.push(quote!{
          let #field_ident = #build_call;
        });
      }

      let output = quote!{
        impl luminance::shader::program::UniformInterface for #ident {
          fn uniform_interface(
            builder: &mut luminance::shader::program::UniformBuilder,
            _: ()
          ) -> Result<Self, luminance::shader::program::ProgramError> {
            #(#field_decls)*

            let iface = #ident { #(#field_names,)* };
            Ok(iface)
          }
        }
      };

      Ok(output.into())
    }

    Fields::Unnamed(_) => Err(DeriveUniformInterfaceError::UnsupportedUnnamed),
    Fields::Unit => Err(DeriveUniformInterfaceError::UnsupportedUnit),
  }
}
