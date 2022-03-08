use darling::ast::Data;
use darling::{FromDeriveInput, FromField, FromVariant};

use proc_macro2::TokenStream;
use quote::quote;

#[derive(FromDeriveInput)]
#[darling(attributes(osi))]
pub(crate) struct OsiTypeInput {
    ident: syn::Ident,
    data: Data<OsiTypeVariant, OsiTypeField>,

    type_name: String,
}

#[allow(dead_code)]
#[derive(FromVariant, Clone)]
struct OsiTypeVariant {
    ident: syn::Ident,
    discriminant: Option<syn::Expr>,
    fields: darling::ast::Fields<OsiTypeVariantField>,
}

#[derive(FromField, Clone)]
struct OsiTypeVariantField {}

#[derive(FromField, Clone)]
#[darling(attributes(osi))]
struct OsiTypeField {
    ident: Option<syn::Ident>,
    ty: syn::Type,

    #[darling(default)]
    rename: Option<String>,
}

impl OsiTypeInput {
    pub(crate) fn to_tokens(self) -> TokenStream {
        let method_dispatcher = quote::format_ident!("{}MethodDispatcher", self.ident);
        let self_ident = &self.ident;

        let type_name = &self.type_name;

        let self_struct = self.data.clone().take_struct().unwrap();
        let read_fields = self_struct.fields.iter().map(|field| {
            let ident = &field.ident;
            let ty = &field.ty;

            let field_name = field.rename
                .clone()
                .or_else(|| ident.as_ref().map(ToString::to_string))
                .unwrap();

            quote! {
                let #ident = ::panda::mem::read_guest_type::<#ty>(
                    __cpu, __base_ptr + (__osi_type.offset_of(#field_name) as ::panda::prelude::target_ptr_t)
                )?;
            }
        });

        let read_field_methods = self_struct.fields.iter().map(|field| {
            let ident = &field.ident;
            let ty = &field.ty;

            let field_name = field.rename
                .clone()
                .or_else(|| ident.as_ref().map(ToString::to_string))
                .unwrap();

            quote! {
                pub fn #ident(&self, __cpu: &mut CPUState) -> Result<#ty, ::panda::GuestReadFail> {
                    let __osi_type = ::panda::plugins::osi2::type_from_name(#type_name)
                        .ok_or(::panda::GuestReadFail)?;

                    let is_per_cpu = self.1;
                    let __base_ptr = if is_per_cpu {
                        ::panda::plugins::osi2::find_per_cpu_address(__cpu, self.0)?
                    } else {
                        ::panda::plugins::osi2::symbol_addr_from_name(
                            self.0
                        )
                    };

                    ::panda::mem::read_guest_type::<#ty>(
                        __cpu, __base_ptr + (__osi_type.offset_of(#field_name) as ::panda::prelude::target_ptr_t)
                    )
                }
            }
        });

        let field_names = self_struct.fields.iter().map(|field| &field.ident);

        quote! {
            #[doc(hidden)]
            pub struct #method_dispatcher(&'static str, bool);

            impl #method_dispatcher {
                pub const fn new(symbol: &'static str, is_per_cpu: bool) -> Self {
                    Self(symbol, is_per_cpu)
                }

                #(
                    #read_field_methods
                )*
            }

            impl ::panda::plugins::osi2::OsiType for #self_ident {
                type MethodDispatcher = #method_dispatcher;

                fn osi_read(
                    __cpu: &mut ::panda::prelude::CPUState,
                    __base_ptr: ::panda::prelude::target_ptr_t,
                ) -> Result<Self, ::panda::GuestReadFail> {
                    let __osi_type = ::panda::plugins::osi2::type_from_name(#type_name)
                        .ok_or(::panda::GuestReadFail)?;


                    #(
                        #read_fields
                    )*

                    Ok(Self { #( #field_names ),* })
                }
            }
        }
    }
}
