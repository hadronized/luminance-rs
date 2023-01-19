//! Implementation of the derive proc-macros for [`Std140`] and [`Std430`].

use proc_macro::{Diagnostic, Level};
use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;
use syn::DeriveInput;

pub enum Layout {
  Std140,
  Std430,
}

pub fn impl_memory_layout(item: DeriveInput, layout: Layout) -> TokenStream {
  match item.data {
    syn::Data::Struct(data) => {
      let (field_ctors, field_decls, off, mut max_align) =
        data
          .fields
          .iter()
          .fold((Vec::new(), Vec::new(), quote!{ 0 }, quote!{ 0 }), |(mut field_ctors, mut field_decls, off, max_align), field| {
            let field_ident = field.ident.as_ref().unwrap();
            let pad_field_name = Ident::new(&format!("_pad_{}", field_ident), Span::call_site());
            let field_ty = &field.ty;
            let field_align = quote! { <#field_ty as luminance::shader::MemoryAlign<luminance::shader::Std140>>::ALIGNMENT };
            let pad_size = quote! { (#field_align - (#off) % #field_align) % #field_align };

            // how to build those fields; padding then actual field
            field_ctors.push(quote! { #pad_field_name: [0; #pad_size] });
            field_ctors.push(quote! { #field_ident: s.#field_ident });

            // add padding to the list of fields
            field_decls.push(quote!{ #pad_field_name: [u8; #pad_size] });

            // add the regular field
            field_decls.push(quote!{ #field });

            // compute the new offset and new max_align
            let max_align = quote! { std::cmp::max(#max_align, #field_align) };
            let off = quote! { #off + #pad_size + std::mem::size_of::<#field_ty>() };

            (field_ctors, field_decls, off, max_align)
          });

      // the alignment of the struct must be rounded up to the aligment of a vec4 (16 bytes); this is only there for
      // Std140; Std430 doesn’t have that restriction (same for arrays)
      let struct_ident = &item.ident;
      let memory_layout;
      let aligned_ident;
      if let Layout::Std140 = layout {
        max_align = quote! { ((#max_align + 15) & !15) };
        memory_layout = quote! { luminance::shader::Std140 };
        aligned_ident = Ident::new(&format!("{}Std140", struct_ident), Span::call_site());
      } else {
        memory_layout = quote! { luminance::shader::Std430 };
        aligned_ident = Ident::new(&format!("{}Std430", struct_ident), Span::call_site());
      }

      let struct_padding = quote! { (#max_align - (#off) % #max_align) % #max_align };

      quote! {
        #[repr(C)]
        pub struct #aligned_ident {
          #(#field_decls ,)*

          // last padding, if any
          _pad_struct: [u8; #struct_padding],
        }

        impl From<#struct_ident> for #aligned_ident {
          fn from(s: #struct_ident) -> Self {
            #aligned_ident {
              #(#field_ctors ,)*

              _pad_struct: [0; #struct_padding],
            }
          }
        }

        unsafe impl luminance::shader::MemoryLayout<#memory_layout> for #struct_ident {
          type Aligned = #aligned_ident;
        }
      }
    }

    syn::Data::Enum(_) => {
      Diagnostic::new(Level::Error, "cannot implement MemoryLayout for enum").emit();
      proc_macro2::TokenStream::new()
    }

    syn::Data::Union(_) => {
      Diagnostic::new(Level::Error, "cannot implement MemoryLayout for union").emit();
      proc_macro2::TokenStream::new()
    }
  }
}
