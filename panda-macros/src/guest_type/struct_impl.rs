use proc_macro2::TokenStream;
use quote::quote;

use super::GuestTypeField;

pub fn struct_layout<'a>(fields: impl Iterator<Item = &'a syn::Type> + 'a) -> TokenStream {
    quote! {
        Some(
            ::std::alloc::Layout::from_size_align(0, 1).ok()?
                #(
                    .extend(<#fields as ::panda::GuestType>::guest_layout()?).ok()?.0
                )*
                .pad_to_align()
        )
    }
}

fn read(is_virt: bool, fields: &[GuestTypeField]) -> TokenStream {
    let field_name = fields
        .iter()
        .map(|field| field.ident.as_ref().unwrap())
        .collect::<Vec<_>>();
    let field_ty = fields.iter().map(|field| &field.ty);

    let read_method = if is_virt {
        quote!(read_from_guest)
    } else {
        quote!(read_from_guest_phys)
    };

    let cpu = is_virt.then(|| quote! { __cpu, });
    let layout = quote! { __layout };
    quote! {
            let #layout = ::std::alloc::Layout::from_size_align(0, 1).unwrap();

            #(
                let (#layout, offset) = #layout.extend(
                    <#field_ty as ::panda::GuestType>::guest_layout().unwrap()
                ).unwrap();

                let #field_name = <#field_ty as ::panda::GuestType>::#read_method(
                    #cpu __ptr + (offset as ::panda::prelude::target_ptr_t)
                )?;
            )*

            Ok(Self { #( #field_name ),* })
    }
}

pub(super) fn read_from_guest(fields: &[GuestTypeField]) -> TokenStream {
    read(true, fields)
}

pub(super) fn read_from_guest_phys(fields: &[GuestTypeField]) -> TokenStream {
    read(false, fields)
}

fn write(is_virt: bool, fields: &[GuestTypeField]) -> TokenStream {
    let field_name = fields
        .iter()
        .map(|field| field.ident.as_ref().unwrap())
        .collect::<Vec<_>>();
    let field_ty = fields.iter().map(|field| &field.ty);

    let write_method = if is_virt {
        quote!(write_to_guest)
    } else {
        quote!(write_to_guest_phys)
    };

    let cpu = is_virt.then(|| quote! { __cpu, });
    let layout = quote! { __layout };
    quote! {
            let #layout = ::std::alloc::Layout::from_size_align(0, 1).unwrap();

            #(
                let (#layout, offset) = #layout.extend(
                    <#field_ty as ::panda::GuestType>::guest_layout().unwrap()
                ).unwrap();

                <#field_ty as ::panda::GuestType>::#write_method(
                    &self.#field_name,
                    #cpu __ptr + (offset as ::panda::prelude::target_ptr_t)
                )?;
            )*

            Ok(())
    }
}

pub(super) fn write_to_guest(fields: &[GuestTypeField]) -> TokenStream {
    write(true, fields)
}

pub(super) fn write_to_guest_phys(fields: &[GuestTypeField]) -> TokenStream {
    write(false, fields)
}
