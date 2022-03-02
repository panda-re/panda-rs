use quote::{quote, quote_spanned};
use syn::spanned::Spanned;

pub(crate) struct OsiStatics {
    inner: Vec<syn::ForeignItemStatic>,
}

impl syn::parse::Parse for OsiStatics {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut inner = vec![];

        while !input.is_empty() {
            inner.push(input.parse()?);
        }

        Ok(OsiStatics { inner })
    }
}

impl quote::ToTokens for OsiStatics {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        for item in &self.inner {
            let is_per_cpu = item.attrs.iter().any(|attr| {
                attr.path
                    .get_ident()
                    .map(|ident| ident == "per_cpu")
                    .unwrap_or(false)
            });

            let symbol = item.attrs.iter().find_map(|attr| {
                (attr.path.get_ident()? == "symbol").then(|| {
                    let meta = attr.parse_meta().map_err(|_| {
                        let span = attr.path.span();
                        quote_spanned! { span =>
                            compile_error!("Symbol must take the form of `#[symbol = \"…\"]`")
                        }
                    })?;

                    if let syn::Meta::NameValue(syn::MetaNameValue {
                        lit: syn::Lit::Str(symbol),
                        ..
                    }) = meta
                    {
                        Ok(symbol)
                    } else {
                        let span = attr.path.span();

                        Err(quote_spanned! { span =>
                            compile_error!("Symbol must take the form of `#[symbol = \"…\"]`")
                        })
                    }
                })
            });

            match symbol {
                Some(Ok(symbol)) => {
                    let osi_static_type = if is_per_cpu {
                        quote! { ::panda::plugins::osi2::PerCpu }
                    } else {
                        quote! { ::panda::plugins::osi2::OsiGlobal }
                    };
                    let ident = &item.ident;
                    let ty = &item.ty;

                    tokens.extend(quote! {
                        static #ident: #osi_static_type<#ty> = #osi_static_type(
                            #symbol,
                            <#ty as ::panda::plugins::osi2::OsiType>::MethodDispatcher::new(
                                #symbol, #is_per_cpu
                            ),
                        );
                    });
                }
                Some(Err(err)) => tokens.extend(err),
                None => {
                    let span = item.span();

                    tokens.extend(quote_spanned! { span =>
                        compile_error!("Missing attribute `#[symbol = \"…\"]`")
                    });
                }
            }
        }
    }
}
