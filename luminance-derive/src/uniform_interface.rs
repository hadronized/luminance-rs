//! Automatically derive `UniformInterface`.
//!
//! The procedural macro is very simple to use. You declare a struct as you would normally do:
//!
//! ```
//! # use luminance_derive::UniformInterface;
//!
//! #[derive(Debug, UniformInterface)]
//! struct MyIface {
//!   time: f32,
//!   resolution: [f32; 4]
//! }
//! ```
//!
//! The effect of this declaration is declaring the `MyIface` struct along with an effective
//! implementation of `UniformInterface` that will try to get the `"time"` and `"resolution"`
//! uniforms in the corresponding shader program. If any of the two uniforms fails to map (inactive
//! uniform, for instance), the whole struct cannot be generated, and an error is arisen (see
//! `UniformInterface::uniform_interface`’s documentation for further details).
//!
//! If you don’t use a parameter in your shader, you might not want the whole interface to fail
//! building if that parameter cannot be mapped. You can do that via the `#[unbound]` field
//! attribute:
//!
//! ```
//! struct MyIface {
//!   #[uniform(unbound)]
//!   time: f32, // if this field cannot be mapped, it’ll be ignored
//!   resolution: [f32; 4]
//! }
//! ```
//!
//! You can also change the default mapping with the `#[uniform(name = "string_mapping")]`
//! attribute. This changes the name that must be queried from the shader program for the mapping
//! to be complete:
//!
//! ```
//! struct MyIface {
//!   time: f32,
//!   #[uniform(name = "res")]
//!   resolution: [f32; 4] // must map "res" from the shader program
//! }
//! ```
//!
//! Finally, you can mix both attributes if you want to change the mapping and have an unbound
//! uniform if it cannot be mapped:
//!
//! ```
//! struct MyIface {
//!   time: f32,
//!   #[uniform(name = "res", unbound)]
//!   resolution: [f32; 4] // must map "res" from the shader program and ignored otherwise
//! }
//! ```

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
