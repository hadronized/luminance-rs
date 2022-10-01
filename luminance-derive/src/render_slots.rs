//! Implementation of the derive proc-macro for `RenderSlots`.

use proc_macro::{Diagnostic, Level};
use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{DeriveInput, Ident};

use crate::attrib::get_field_attr_once;

// accepted sub keys for the "vertex" key
const KNOWN_SUBKEYS: &[&str] = &["namespace"];

pub fn impl_render_slots(item: DeriveInput) -> TokenStream {
  let type_ident = &item.ident;
  let attrs = &item.attrs;

  let namespace = get_field_attr_once(type_ident, attrs, "slot", "namespace", KNOWN_SUBKEYS)
    .map(|namespace: Ident| {
      quote! { #namespace }
    })
    .unwrap_or_else(|e| {
      Diagnostic::new(Level::Error, format!("cannot find namespace: {}", e)).emit();
      proc_macro2::TokenStream::new()
    });

  match item.data {
    syn::Data::Struct(data) => {
      let per_channel = data
        .fields
        .iter()
        .map(|field| {
          let field_ident = field.ident.as_ref().expect("field ident");
          let field_name = field_ident.to_string();
          let field_ty = &field.ty;

          let impl_has_field = quote! {
            impl luminance::has_field::HasField<#field_name> for #type_ident {
              type FieldType = #field_ty;
            }
          };

          let has_field_trait_bound = quote! {
            luminance::has_field::HasField<#field_name, FieldType = #field_ty>
          };

          let render_layer_field = quote! {
            pub #field_ident: luminance::render_slots::RenderLayer<#field_ty>
          };

          let render_layer_decl = quote! {
            #field_ident: backend.new_render_layer::<D, _>(
              framebuffer_handle,
              size,
              mipmaps,
              <#namespace as luminance::named_index::NamedIndex<#field_name>>::INDEX
            )?
          };

          (
            impl_has_field,
            has_field_trait_bound,
            render_layer_field,
            render_layer_decl,
          )
        })
        .collect::<Vec<_>>();
      let has_field_impls = per_channel.iter().map(|f| &f.0);
      let has_field_trait_bounds = per_channel.iter().map(|f| &f.1);
      let render_layer_fields = per_channel.iter().map(|f| &f.2);
      let render_layer_decls = per_channel.iter().map(|f| &f.3);

      let render_layers_ty = Ident::new(&format!("{}RenderLayers", type_ident), Span::call_site());

      quote! {
        // implement HasField for all the fields
        #(#has_field_impls)*

        // implement CompatibleRenderSlots
        impl<S> luminance::render_slots::CompatibleRenderSlots<S> for #type_ident
        where S: luminance::render_slots::RenderSlots + #(#has_field_trait_bounds)+*
        {
        }

        // generate a type that will act as RenderSlots::RenderLayers
        #[derive(Debug)]
        pub struct #render_layers_ty {
          #(#render_layer_fields),*
        }

        // implement RenderSlots
        impl luminance::render_slots::RenderSlots for #type_ident {
          type RenderLayers = #render_layers_ty;

          unsafe fn new_render_layers<B, D>(
            backend: &mut B,
            framebuffer_handle: usize,
            size: D::Size,
            mipmaps: usize,
          ) -> Result<Self::RenderLayers, luminance::backend::FramebufferError>
          where
            B: luminance::backend::FramebufferBackend,
            D: luminance::dim::Dimensionable,
          {
            Ok(
              #render_layers_ty { #( #render_layer_decls),* }
            )
          }
        }
      }
    }

    syn::Data::Enum(_) => {
      Diagnostic::new(Level::Error, "cannot implement RenderSlots for enum").emit();
      proc_macro2::TokenStream::new()
    }

    syn::Data::Union(_) => {
      Diagnostic::new(Level::Error, "cannot implement RenderSlots for union").emit();
      proc_macro2::TokenStream::new()
    }
  }
}
