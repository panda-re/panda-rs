#[derive(FromField)]
#[darling(attributes(arg))]
struct DeriveArgs {
    #[darling(default)]
    about: Option<String>,
    #[darling(default)]
    default: Option<syn::Lit>,
    #[darling(default)]
    required: bool,
    ident: Option<syn::Ident>,
    ty: syn::Type,
}

fn derive_args_to_mappings(
    DeriveArgs {
        about,
        default,
        ident,
        ty,
        required,
    }: DeriveArgs,
) -> (syn::Stmt, syn::Ident) {
    let name = &ident;
    let default = if let Some(default) = default {
        match default {
            syn::Lit::Str(string) => quote!(::std::string::String::from(#string)),
            default => quote!(#default),
        }
    } else {
        quote!(Default::default())
    };
    let about = about.unwrap_or_default();
    (
        syn::parse_quote!(
            let #name = <#ty as ::panda::panda_arg::GetPandaArg>::get_panda_arg(
                __args_ptr,
                stringify!(#name),
                #default,
                #about,
                #required
            );
        ),
        ident.unwrap(),
    )
}

fn get_field_statements(
    fields: &syn::Fields,
) -> Result<(Vec<syn::Stmt>, Vec<syn::Ident>), darling::Error> {
    Ok(fields
        .iter()
        .map(DeriveArgs::from_field)
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .map(derive_args_to_mappings)
        .unzip())
}

fn get_name(attrs: &[syn::Attribute]) -> Option<String> {
    attrs
        .iter()
        .find(|attr| attr.path.get_ident().map(|x| *x == "name").unwrap_or(false))
        .map(|attr| attr.parse_meta().ok())
        .flatten()
        .map(|meta| {
            if let syn::Meta::NameValue(syn::MetaNameValue {
                lit: syn::Lit::Str(s),
                ..
            }) = meta
            {
                Some(s.value())
            } else {
                None
            }
        })
        .flatten()
}

#[proc_macro_derive(PandaArgs, attributes(name, arg))]
pub fn derive_panda_args(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as syn::ItemStruct);

    let name = match get_name(&input.attrs) {
        Some(name) => name,
        None => {
            return quote!(compile_error!(
                "Missing plugin name, add `#[name = ...]` above struct"
            ))
            .into()
        }
    };

    let ident = &input.ident;

    match get_field_statements(&input.fields) {
        Ok((statements, fields)) => {
            let format_args = iter::repeat("{}={}")
                .take(statements.len())
                .collect::<Vec<_>>()
                .join(",");
            quote!(
                impl ::panda::PandaArgs for #ident {
                    const PLUGIN_NAME: &'static str = #name;

                    fn from_panda_args() -> Self {
                        let name = ::std::ffi::CString::new(#name).unwrap();

                        unsafe {
                            let __args_ptr = ::panda::sys::panda_get_args(name.as_ptr());

                            #(
                                #statements
                            )*

                            ::panda::sys::panda_free_args(__args_ptr);

                            Self {
                                #(#fields),*
                            }
                        }
                    }

                    fn to_panda_args_str(&self) -> ::std::string::String {
                        format!(
                            concat!(#name, ":", #format_args),
                            #(
                                stringify!(#fields), self.#fields
                            ),*
                       )
                    }

                    fn to_panda_args(&self) -> ::std::vec::Vec<(&'static str, ::std::string::String)> {
                        ::std::vec![
                            #(
                                (stringify!(#fields), self.#fields.to_string()),
                            )*
                        ]
                    }
                }
            ).into()
        }
        Err(err) => err.write_errors().into(),
    }
}
