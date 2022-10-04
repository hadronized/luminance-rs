use crate::attrib::{get_field_attr_once, get_field_flag_once, AttrError};
use proc_macro::TokenStream;
use quote::quote;
use std::error;
use std::fmt;
use syn::{DataStruct, Fields, Ident, Path, PathArguments, Type, TypePath};

const KNOWN_SUBKEYS: &[&str] = &["name", "unbound"];

#[non_exhaustive]
#[derive(Debug)]
pub(crate) enum UniformsError {
  UnsupportedUnnamed,
  UnsupportedUnit,
  UnboundError(AttrError),
  NameError(AttrError),
  IncorrectlyWrappedType(Type),
}

impl fmt::Display for UniformsError {
  fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
    match *self {
      UniformsError::UnsupportedUnnamed => f.write_str("unsupported unnamed fields"),
      UniformsError::UnsupportedUnit => f.write_str("unsupported unit struct"),
      UniformsError::UnboundError(ref e) => write!(f, "unbound error: {}", e),
      UniformsError::NameError(ref e) => write!(f, "name error: {}", e),
      UniformsError::IncorrectlyWrappedType(ref t) => write!(
        f,
        "incorrectly wrapped uniform type: {:?} (should be Uni<YourTypeHere>)",
        t
      ),
    }
  }
}

impl error::Error for UniformsError {
  fn source(&self) -> Option<&(dyn error::Error + 'static)> {
    match self {
      UniformsError::UnboundError(e) => Some(e),
      UniformsError::NameError(e) => Some(e),
      _ => None,
    }
  }
}

pub(crate) fn generate_uniforms_impl(
  ident: Ident,
  struct_: DataStruct,
) -> Result<TokenStream, UniformsError> {
  match struct_.fields {
    Fields::Named(named_fields) => {
      // field declarations; used to declare fields to be mapped while building the final type
      let mut field_decls = Vec::new();
      // collect field names to return the uniforms with the shortcut syntax
      let mut field_names = Vec::new();
      // collect field types so that we can implement UniformInterface<S> where $t: Uniform<S>
      let mut field_where_clause = Vec::new();

      for field in named_fields.named {
        let field_ident = field.ident.unwrap();
        let unbound = get_field_flag_once(
          &ident,
          field.attrs.iter(),
          "uniform",
          "unbound",
          KNOWN_SUBKEYS,
        )
        .map_err(UniformsError::UnboundError)?;
        let name =
          get_field_attr_once(&ident, field.attrs.iter(), "uniform", "name", KNOWN_SUBKEYS)
            .map(|ident: Ident| ident.to_string())
            .or_else(|e| match e {
              AttrError::CannotFindAttribute(..) => Ok(field_ident.to_string()),

              _ => Err(e),
            })
            .map_err(UniformsError::NameError)?;

        // the build call is the code that gets a uniform and possibly fails if bound; also handles
        // renaming
        let build_call = if unbound {
          quote! {
            backend.new_shader_uni(handle, #name).or_else(|| backend.new_shader_uni_unbound(handle))?
          }
        } else {
          quote! {
            backend.new_shader_uni(handle, #name)
          }
        };

        let field_ty =
          extract_uniform_type(&field.ty).ok_or(UniformsError::IncorrectlyWrappedType(field.ty))?;
        field_names.push(field_ident.clone());
        field_decls.push(quote! {
          let #field_ident = #build_call;
        });
        field_where_clause.push(quote! {
          B: luminance::backend::shader::Uniform<#field_ty>
        });
      }

      let output = quote! {
        impl luminance::shader::Uniforms for #ident
        where
          #(#field_where_clause),*,
        {
          fn build_uniforms<B>(backend: &mut B, program_hande: usize) -> Result<Self, luminance::shader::ShaderError>
          where B: luminance::shader::ShaderBackend {
            #(#field_decls)*

            Ok( #ident { #(#field_names,)* })
          }
        }
      };

      Ok(output.into())
    }

    Fields::Unnamed(_) => Err(UniformsError::UnsupportedUnnamed),
    Fields::Unit => Err(UniformsError::UnsupportedUnit),
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
