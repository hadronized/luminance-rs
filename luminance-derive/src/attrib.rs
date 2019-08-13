use std::fmt;
use syn::{Attribute, Ident, Lit, Meta, NestedMeta};
use syn::parse::Parse;

#[derive(Debug)]
pub(crate) enum AttrError {
  Several(Ident, String, String),
  CannotFindAttribute(Ident, String, String),
  CannotParseAttribute(Ident, String, String),
  UnknownSubKey(Ident, String, String),
}

impl fmt::Display for AttrError {
  fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
    match *self {
      AttrError::Several(ref field, ref key, ref sub_key) =>
        write!(f, "expected one pair {}({}) for {}, got several", key, sub_key, field),
      AttrError::CannotFindAttribute(ref field, ref key, ref sub_key) =>
        write!(f, "no attribute found {}({}) for {}", key, sub_key, field),
      AttrError::CannotParseAttribute(ref field, ref key, ref sub_key) =>
        write!(f, "cannot parse attribute {}({}) for {}", key, sub_key, field),
      AttrError::UnknownSubKey(ref field, ref key, ref sub_key) =>
        write!(f, "unknown sub key “{}” in {}({}) for {}", sub_key, key, sub_key, field),
    }
  }
}

/// Get and parse an attribute on a field or a variant that must appear only once with the following
/// syntax:
///
///   #[key(sub_key = "lit")]
pub(crate) fn get_field_attr_once<'a, A, T>(
  field_ident: &Ident,
  attrs: A,
  key: &str,
  sub_key: &str,
  known_subkeys: &[&str]
) -> Result<T, AttrError> where A: IntoIterator<Item = &'a Attribute>, T: Parse {
  let mut lit = None;

  for attr in attrs.into_iter() {
    match attr.parse_meta() {
      Ok(Meta::List(ref ml)) if ml.path.is_ident(key) => {
        let nested = &ml.nested;

        for nested in nested.into_iter() {
          match nested {
            NestedMeta::Meta(Meta::NameValue(ref mnv)) => {
              if mnv.path.is_ident(sub_key) {
                if lit.is_some() {
                  return Err(AttrError::Several(field_ident.clone(), key.to_owned(), sub_key.to_owned()));
                }

                if let Lit::Str(ref strlit) = mnv.lit {
                  lit = Some(strlit.parse().map_err(|_| AttrError::CannotParseAttribute(field_ident.clone(), key.to_owned(), sub_key.to_owned()))?);
                }
              } else {
                let ident_str = mnv.path.segments.first().map(|seg| seg.ident.to_string()).unwrap_or_else(|| String::new());

                if !known_subkeys.contains(&ident_str.as_str()) {
                  return Err(AttrError::UnknownSubKey(field_ident.clone(), key.to_owned(), ident_str));
                }
              }
            }

            _ => ()
          }
        }
      }

      _ => () // ignore things that might not be ours
    }
  }

  lit.ok_or(AttrError::CannotFindAttribute(field_ident.clone(), key.to_owned(), sub_key.to_owned()))
}

/// Get and parse an attribute on a field or a variant that must appear only once with the following
/// syntax:
///
///   #[key(sub_key)]
pub(crate) fn get_field_flag_once<'a, A>(
  field_ident: &Ident,
  attrs: A,
  key: &str,
  sub_key: &str,
  known_subkeys: &[&str]
) -> Result<bool, AttrError> where A: IntoIterator<Item = &'a Attribute> {
  let mut flag = false;

  for attr in attrs.into_iter() {
    match attr.parse_meta() {
      Ok(Meta::List(ref ml)) if ml.path.is_ident(key) => {
        let nested = &ml.nested;

        for nested in nested.into_iter() {
          match nested {
            NestedMeta::Meta(Meta::Path(ref path)) => {
              if path.is_ident(sub_key) {
                if flag {
                  return Err(AttrError::Several(field_ident.clone(), key.to_owned(), sub_key.to_owned()));
                }

                flag = true;
              } else {
                let ident_str = path.segments.first().map(|seg| seg.ident.to_string()).unwrap_or_else(|| String::new());

                if !known_subkeys.contains(&ident_str.as_str()) {
                  return Err(AttrError::UnknownSubKey(field_ident.clone(), key.to_owned(), ident_str));
                }
              }
            }

            _ => ()
          }
        }
      }

      _ => () // ignore things that might not be ours
    }
  }

  Ok(flag)
}
