use crate::attrib::{get_field_attr_once, get_field_flag_once, AttrError};
use proc_macro::TokenStream;
use quote::quote;
use std::error;
use std::fmt;
use syn::{DataStruct, Fields, Field, Ident, Path, PathArguments, Type, TypePath};

// accepted sub keys for the "vertex" key
const KNOWN_SUBKEYS: &[&str] = &["name", "unbound"];

#[non_exhaustive]
#[derive(Debug)]
pub(crate) enum DeriveUniformInterfaceError {
  UnsupportedUnnamed,
  UnsupportedUnit,
  UnboundError(AttrError),
  NameError(AttrError),
  IncorrectlyWrappedType(Box<Type>),
}

impl DeriveUniformInterfaceError {
  pub(crate) fn unsupported_unnamed() -> Self {
    DeriveUniformInterfaceError::UnsupportedUnnamed
  }

  pub(crate) fn unsupported_unit() -> Self {
    DeriveUniformInterfaceError::UnsupportedUnit
  }

  pub(crate) fn unbound_error(e: AttrError) -> Self {
    DeriveUniformInterfaceError::UnboundError(e)
  }

  pub(crate) fn name_error(e: AttrError) -> Self {
    DeriveUniformInterfaceError::NameError(e)
  }

  pub(crate) fn incorrectly_wrapped_type(ty: &Type) -> Self {
    DeriveUniformInterfaceError::IncorrectlyWrappedType(Box::new(ty.clone()))
  }
}

impl fmt::Display for DeriveUniformInterfaceError {
  fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
    match *self {
      DeriveUniformInterfaceError::UnsupportedUnnamed => f.write_str("unsupported unnamed fields"),
      DeriveUniformInterfaceError::UnsupportedUnit => f.write_str("unsupported unit struct"),
      DeriveUniformInterfaceError::UnboundError(ref e) => write!(f, "unbound error: {}", e),
      DeriveUniformInterfaceError::NameError(ref e) => write!(f, "name error: {}", e),
      DeriveUniformInterfaceError::IncorrectlyWrappedType(ref t) => write!(
        f,
        "incorrectly wrapped uniform type: {:?} (should be Uniform<YourTypeHere>)",
        t
      ),
    }
  }
}

impl error::Error for DeriveUniformInterfaceError {
  fn source(&self) -> Option<&(dyn error::Error + 'static)> {
    match self {
      DeriveUniformInterfaceError::UnboundError(e) => Some(e),
      DeriveUniformInterfaceError::NameError(e) => Some(e),
      _ => None,
    }
  }
}

pub(crate) fn generate_uniform_interface_impl(
  ident: Ident,
  struct_: DataStruct,
) -> Result<TokenStream, DeriveUniformInterfaceError> {
  match struct_.fields {
    Fields::Named(named_fields) => {
      // field declarations; used to declare fields to be mapped while building the uniform
      // interface
      let mut field_decls = Vec::new();
      // collect field names to return the uniform interface with the shortcut syntax
      let mut field_names = Vec::new();
      // collect field types so that we can implement UniformInterface<S> where $t: Uniform<S>
      let mut field_where_clause = Vec::new();

      for Field { ident, ty, attrs, .. } in named_fields.named {
        let ident = ident.unwrap();
        let unbound = get_field_flag_once(
          &ident,
          attrs.iter(),
          "uniform",
          "unbound",
          KNOWN_SUBKEYS,
        )
        .map_err(DeriveUniformInterfaceError::unbound_error)?;
        let name =
          get_field_attr_once(&ident, attrs.iter(), "uniform", "name", KNOWN_SUBKEYS)
            .map(|ident: Ident| ident.to_string())
            .or_else(|e| match e {
              AttrError::CannotFindAttribute(..) => Ok(ident.to_string()),

              _ => Err(e),
            })
            .map_err(DeriveUniformInterfaceError::name_error)?;

        // the build call is the code that gets a uniform and possibly fails if bound; also handles
        // renaming
        let build_call = if unbound {
          quote! {
            builder.ask_or_unbound(#name)
          }
        } else {
          quote! {
            builder.ask(#name)?
          }
        };

        let field_ty = extract_uniform_type(&ty).ok_or_else(|| {
          DeriveUniformInterfaceError::incorrectly_wrapped_type(&ty)
        })?;
        field_names.push(ident.clone());
        field_decls.push(quote! {
          let #ident = #build_call;
        });
        field_where_clause.push(quote! {
          #field_ty: luminance::backend::shader::Uniformable<S>
        });
      }

      let output = quote! {
        impl<S> luminance::shader::UniformInterface<S> for #ident
        where
          S: ?Sized + luminance::backend::shader::Shader,
          #(#field_where_clause),*,
        {
          fn uniform_interface<'a>(
            builder: &mut luminance::shader::UniformBuilder<'a, S>,
            _: &mut ()
          ) -> Result<Self, luminance::shader::UniformWarning> {
            #(#field_decls)*

            let iface = #ident { #(#field_names,)* };
            Ok(iface)
          }
        }
      };

      Ok(output.into())
    }

    Fields::Unnamed(_) => Err(DeriveUniformInterfaceError::unsupported_unnamed()),
    Fields::Unit => Err(DeriveUniformInterfaceError::unsupported_unit()),
  }
}

// extract the type T in Uniform<T>
fn extract_uniform_type(ty: &Type) -> Option<proc_macro2::TokenStream> {
  if let Type::Path(TypePath {
    path: Path { ref segments, .. },
    ..
  }) = ty
  {
    let segment = segments.first()?;

    if let PathArguments::AngleBracketed(ref bracketed_args) = segment.arguments {
      let sub = bracketed_args.args.first()?;
      Some(quote! { #sub })
    } else {
      None
    }
  } else {
    None
  }
}
