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

    #[darling(default)]
    osi_type: bool,
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

            let read_func = if field.osi_type {
                quote! {
                    <#ty as ::panda::plugins::osi2::OsiType>::osi_read
                }
            } else {
                quote! {
                    ::panda::mem::read_guest_type::<#ty>
                }
            };

            quote! {
                let __field_offset = {
                    static FIELD_OFFSET: ::panda::once_cell::sync::OnceCell<::panda::prelude::target_long>
                        = ::panda::once_cell::sync::OnceCell::new();

                    *FIELD_OFFSET.get_or_init(|| {
                        __osi_type.offset_of(#field_name)
                    })
                };

                let #ident = #read_func (
                    __cpu, __base_ptr + (__field_offset as ::panda::prelude::target_ptr_t)
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

            let read_func = if field.osi_type {
                quote! {
                    <#ty as ::panda::plugins::osi2::OsiType>::osi_read
                }
            } else {
                quote! {
                    ::panda::mem::read_guest_type::<#ty>
                }
            };

            quote! {
                pub(crate) fn #ident(&self, __cpu: &mut CPUState) -> Result<#ty, ::panda::GuestReadFail> {
                    let __osi_type = ::panda::plugins::osi2::type_from_name(#type_name)
                        .ok_or(::panda::GuestReadFail)?;

                    let is_per_cpu = self.1;
                    let __base_ptr = if is_per_cpu {
                        static PER_CPU_ADDR: ::panda::once_cell::sync::OnceCell<
                            Result<
                                ::panda::prelude::target_ptr_t,
                                ::panda::GuestReadFail
                            >
                        >
                            = ::panda::once_cell::sync::OnceCell::new();

                        (*PER_CPU_ADDR.get_or_init(|| {
                            ::panda::plugins::osi2::find_per_cpu_address(__cpu, self.0)
                        }))?
                    } else {
                        static SYMBOL_ADDR: ::panda::once_cell::sync::OnceCell<::panda::prelude::target_ptr_t>
                            = ::panda::once_cell::sync::OnceCell::new();

                        *SYMBOL_ADDR.get_or_init(|| {
                            ::panda::plugins::osi2::symbol_addr_from_name(
                                self.0
                            )
                        })
                    };

                    #read_func (
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
