//! Implementation of the derive proc-macro for `RenderSlots`.

use proc_macro::{Diagnostic, Level};
use proc_macro2::TokenStream;
use quote::quote;
use syn::DeriveInput;

pub fn impl_render_slots(item: DeriveInput) -> TokenStream {
  let type_ident = &item.ident;

  match item.data {
    syn::Data::Struct(data) => {
      let per_channel = data
        .fields
        .iter()
        .enumerate()
        .map(|(index, field)| {
          let field_ident = field.ident.as_ref().expect("ident name").to_string();
          let field_ty = &field.ty;

          let render_channel = quote! {
            luminance::render_channel::RenderChannel {
              index: #index,
              name: #field_ident,
              ty: <#field_ty as luminance::render_channel::IsRenderChannelType>::CHANNEL_TY,
              dim: <#field_ty as luminance::render_channel::IsRenderChannelType>::CHANNEL_DIM,
            }
          };

          let impl_has_field = quote! {
            luminance::has_field::HasField<#field_ident> {
              type FieldType = #field_ty;
            }
          };

          let has_field_trait_bound = quote! {
            luminance::has_field::HasField<#field_ident, FieldType = #field_ty>
          };

          let render_layer_field = quote! {
            #field_ident: luminance::render_slots::RenderLayer<#field_ty>
          };

          (
            render_channel,
            impl_has_field,
            has_field_trait_bound,
            render_layer_field,
            field_ident,
          )
        })
        .collect::<Vec<_>>();
      let channels = per_channel.iter().map(|f| &f.0);
      let has_field_impls = per_channel.iter().map(|f| &f.1);
      let has_field_trait_bounds = per_channel.iter().map(|f| &f.2);
      let render_layer_fields = per_channel.iter().map(|f| &f.3);
      let field_idents = per_channel.iter().map(|f| &f.4);

      let channels_count = data.fields.len();
      let render_layers_ty = format!("{}RenderLayers", type_ident);

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

          const CHANNELS: &'static [luminance::render_channel::RenderChannel] = &[ #(#channels),* ];

          fn channels_count() -> usize {
            #channels_count
          }

          unsafe fn new_render_slots<B, D>(backend: &mut B, size: D::Size) -> Result<Self::RenderLayers, B::Err>
          where
            B: Backend,
            D: Dimensionable,
          {
            let layers =
              #render_layers_ty {
                #( #field_idents: backend.new_render_layer(size) ),*
              };
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
