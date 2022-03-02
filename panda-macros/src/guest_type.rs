use darling::ast::Data;
use darling::{FromDeriveInput, FromField, FromVariant};

use proc_macro2::TokenStream;
use quote::quote;

#[derive(FromDeriveInput)]
pub(crate) struct GuestTypeInput {
    ident: syn::Ident,
    data: Data<GuestTypeVariant, GuestTypeField>,

    #[darling(default)]
    guest_repr: String,
}

#[allow(dead_code)]
#[derive(FromVariant)]
struct GuestTypeVariant {
    ident: syn::Ident,
    discriminant: Option<syn::Expr>,
    fields: darling::ast::Fields<GuestTypeVariantField>,
}

#[derive(FromField)]
struct GuestTypeVariantField {}

#[derive(FromField)]
struct GuestTypeField {
    ident: Option<syn::Ident>,
    ty: syn::Type,
}

enum IntRepr {
    U8,
    U16,
    U32,
    U64,
    I8,
    I16,
    I32,
    I64,
}

enum Repr {
    C,
    Packed,
    Int(IntRepr),
}

impl Repr {
    fn from_str(repr: &str) -> Self {
        match repr {
            "" | "c" | "C" => Repr::C,
            "packed" => Repr::Packed,
            "u8" => Repr::Int(IntRepr::U8),
            "u16" => Repr::Int(IntRepr::U16),
            "u32" => Repr::Int(IntRepr::U32),
            "u64" => Repr::Int(IntRepr::U64),
            "i8" => Repr::Int(IntRepr::I8),
            "i16" => Repr::Int(IntRepr::I16),
            "i32" => Repr::Int(IntRepr::I32),
            "i64" => Repr::Int(IntRepr::I64),
            _ => panic!("Invalid repr: must be one of 'c', 'packed', or integer type"),
        }
    }
}

struct Impls {
    guest_layout: TokenStream,
    read_from_guest: TokenStream,
    write_to_guest: TokenStream,
    read_from_guest_phys: TokenStream,
    write_to_guest_phys: TokenStream,
}

fn todo() -> TokenStream {
    quote! { todo!() }
}

mod struct_impl;

impl GuestTypeInput {
    pub(crate) fn to_tokens(self) -> TokenStream {
        let Self {
            ident,
            data,
            guest_repr,
        } = self;

        let ty = ident;
        let repr = Repr::from_str(&guest_repr);

        if data.is_struct() && matches!(repr, Repr::Int(_)) {
            panic!("guest_repr = \"{}\" is only allowed on enums", guest_repr);
        }

        let impls = match data {
            Data::Enum(_en) => Impls {
                guest_layout: todo(),
                read_from_guest: todo(),
                write_to_guest: todo(),
                read_from_guest_phys: todo(),
                write_to_guest_phys: todo(),
            },
            Data::Struct(st) => {
                let guest_layout =
                    struct_impl::struct_layout(st.fields.iter().map(|field| &field.ty));

                let read_from_guest = struct_impl::read_from_guest(&st.fields);
                let read_from_guest_phys = struct_impl::read_from_guest_phys(&st.fields);

                let write_to_guest = struct_impl::write_to_guest(&st.fields);
                let write_to_guest_phys = struct_impl::write_to_guest_phys(&st.fields);

                Impls {
                    guest_layout,
                    read_from_guest,
                    write_to_guest,
                    read_from_guest_phys,
                    write_to_guest_phys,
                }
            }
        };

        let Impls {
            guest_layout,
            read_from_guest,
            write_to_guest,
            read_from_guest_phys,
            write_to_guest_phys,
        } = impls;

        let ret_type = quote!( Result<Self, ::panda::GuestReadFail> );
        let write_ret = quote!( Result<(), ::panda::GuestWriteFail> );

        quote! {
            const _: fn() = || {
                use panda::prelude::*;

                impl ::panda::GuestType for #ty {
                    fn guest_layout() -> Option<::std::alloc::Layout> {
                        #guest_layout
                    }

                    fn read_from_guest(__cpu: &mut CPUState, __ptr: target_ptr_t) -> #ret_type {
                        #read_from_guest
                    }

                    fn write_to_guest(&self, __cpu: &mut CPUState, __ptr: target_ptr_t) -> #write_ret {
                        #write_to_guest
                    }

                    fn read_from_guest_phys(__ptr: target_ptr_t) -> #ret_type {
                        #read_from_guest_phys
                    }

                    fn write_to_guest_phys(&self, __ptr: target_ptr_t) -> #write_ret {
                        #write_to_guest_phys
                    }
                }
            };
        }
    }
}
