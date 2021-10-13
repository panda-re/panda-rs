use darling::FromField;
use proc_macro::TokenStream;
use quote::quote;
use std::iter;

/// (**Required** Callback) Called when the plugin is being uninitialized
///
///### Example
///
/// ```rust
///use panda::PluginHandle;
///
/// #[panda::init]
/// fn start(_: &mut PluginHandle) {
///     println!("Plugin started up!");
/// }
/// ```
#[proc_macro_attribute]
pub fn init(_: TokenStream, function: TokenStream) -> TokenStream {
    let func = syn::parse_macro_input!(function as syn::ItemFn);

    let args = if func.sig.inputs.is_empty() {
        None
    } else {
        Some(quote!(unsafe { &mut *plugin }))
    };

    let func_name = &func.sig.ident;

    quote!(
        mod __panda_internal {
            use super::*;

            #[no_mangle]
            unsafe extern "C" fn init_plugin(plugin: *mut ::panda::PluginHandle) {
                for cb in ::panda::inventory::iter::<::panda::Callback> {
                    ::panda::sys::panda_register_callback(plugin as _, cb.cb_type, ::core::mem::transmute(cb.fn_pointer));
                }

                for cb in ::panda::inventory::iter::<::panda::PPPCallbackSetup> {
                    cb.0();
                }

                #func_name(#args);
            }

            #[no_mangle]
            unsafe extern "C" fn uninit_plugin(plugin: *mut ::panda::PluginHandle) {
                for cb in ::panda::inventory::iter::<::panda::UninitCallback> {
                    cb.0(unsafe { &mut *plugin });
                }
            }
        }

        #func
    ).into()
}

/// (Callback) Called when the plugin is being uninitialized
#[proc_macro_attribute]
pub fn uninit(_: TokenStream, function: TokenStream) -> TokenStream {
    let func = syn::parse_macro_input!(function as syn::ItemFn);
    let func_name = &func.sig.ident;

    quote!(
        ::panda::inventory::submit! {
            #![crate = ::panda]
            ::panda::UninitCallback(#func_name)
        }

        #func
    )
    .into()
}

#[proc_macro_attribute]
pub fn hook(_: TokenStream, func: TokenStream) -> TokenStream {
    let mut function = syn::parse_macro_input!(func as syn::ItemFn);
    function.sig.abi = Some(syn::parse_quote!(extern "C"));
    let vis = &function.vis;
    let func = &function.sig.ident;
    let cfgs = crate::get_cfg_attrs(&function);

    let args = &function.sig.inputs;
    let ret = &function.sig.output;
    let ty: syn::Type = syn::parse_quote! { extern "C" fn(  #args ) #ret };

    quote!(
        #( #cfgs )*
        #vis mod #func {
            use super::*;

            pub fn hook() -> <#ty as ::panda::plugins::hooks::IntoHookBuilder>::BuilderType {
                <#ty as ::panda::plugins::hooks::IntoHookBuilder>::hook(#func)
            }
        }

        #function
    )
    .into()
}

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
                }
            )
        }
        Err(err) => err.write_errors(),
    }
    .into()
}

struct Idents(syn::Ident, syn::Ident);

impl syn::parse::Parse for Idents {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        Ok(Idents(input.parse()?, input.parse()?))
    }
}

fn get_cfg_attrs(func: &syn::ItemFn) -> Vec<syn::Attribute> {
    func.attrs
        .iter()
        .filter(|attr| attr.path.get_ident().map(|x| *x == "cfg").unwrap_or(false))
        .cloned()
        .collect()
}

macro_rules! define_callback_attributes {
    ($(
        $($doc:literal)*
        ($attr_name:ident, $const_name:ident, ($($arg:ty),*) $(-> $ret:ty)?)
    ),*) => {
        $(
            doc_comment::doc_comment!{
                concat!(
                    "(Callback) ",
                    $($doc, "\n",)*
                    "\n\nCallback arguments: (",
                    $("`", stringify!($arg), "`, ",)*
                    ")\n### Example\n```rust\nuse panda::prelude::*;\n\n#[panda::",
                    stringify!($attr_name),
                    "]\nfn callback(",
                    $("_: ", stringify!($arg), ", ", )* ")",
                    $(" -> ", stringify!($ret),)?
                    " {\n    // do stuff\n}\n```"),
                #[proc_macro_attribute]
                pub fn $attr_name(_: TokenStream, function: TokenStream) -> TokenStream {
                    let mut function = syn::parse_macro_input!(function as syn::ItemFn);
                    function.sig.abi = Some(syn::parse_quote!(extern "C"));
                    let vis = &function.vis;
                    let func = &function.sig.ident;
                    let cfgs = crate::get_cfg_attrs(&function);

                    quote!(
                        #(
                            #cfgs
                         )*
                        const _: fn() = || {
                            use ::panda::sys::*;
                            fn assert_callback_arg_types(_ : extern "C" fn($($arg),*) $(-> $ret)?) {}

                            assert_callback_arg_types(#func);
                        };

                        ::panda::inventory::submit! {
                            #![crate = ::panda]
                            ::panda::Callback::new(
                                ::panda::sys::$const_name,
                                #func as *const ()
                            )
                        }

                        #vis mod #func {
                            pub fn enable() {
                                unsafe {
                                    ::panda::sys::panda_enable_callback(
                                        ::panda::sys::panda_get_plugin_by_name(
                                            ::std::concat!(
                                                ::std::env!("CARGO_PKG_NAME"),
                                                "\0"
                                            ).as_ptr() as _
                                        ),
                                        ::panda::sys::$const_name,
                                        ::std::mem::transmute(super::#func as *const ())
                                    );
                                }
                            }

                            pub fn disable() {
                                println!("plugin: {}", env!("CARGO_PKG_NAME"));
                                unsafe {
                                    ::panda::sys::panda_disable_callback(
                                        ::panda::sys::panda_get_plugin_by_name(
                                            ::std::concat!(
                                                ::std::env!("CARGO_PKG_NAME"),
                                                "\0"
                                            ).as_ptr() as _
                                        ),
                                        ::panda::sys::$const_name,
                                        ::std::mem::transmute(super::#func as *const ())
                                    );
                                }
                            }
                        }

                        #function
                    ).into()
                }
            }
        )*
    }
}

#[cfg(not(any(feature = "ppc", feature = "mips64")))]
macro_rules! define_syscalls_callbacks {
    ($(
        $($doc:literal)*
        (
            $attr_name:ident,
            $cb_name:ident,
            $syscall_name:ident,
            $before_or_after:literal,
            ($($arg_name:ident : $arg:ty),* $(,)?)
        )
    ),* $(,)?) => {
        $(
            doc_comment::doc_comment!{
                concat!(
                    "(Callback) A callback that runs ",
                    $before_or_after,
                    " the ",
                    stringify!($syscall_name),
                    " syscall runs.\n\nCallback arguments: (",
                    $("`", stringify!($arg), "`,",)*
                    ")\n### Example\n```rust\nuse panda::prelude::*;\n\n#[panda::",
                    stringify!($attr_name),
                    "]\nfn callback(",
                    $("_: ", stringify!($arg), ", ",)*
                    ") {\n    // do stuff\n}\n```"
                ),
                #[proc_macro_attribute]
                pub fn $attr_name(_: TokenStream, function: TokenStream) -> TokenStream {
                    let mut function = syn::parse_macro_input!(function as syn::ItemFn);
                    function.sig.abi = Some(syn::parse_quote!(extern "C"));
                    let func = &function.sig.ident;
                    let cfgs = crate::get_cfg_attrs(&function);

                    quote!(
                        #(
                            #cfgs
                         )*
                        ::panda::inventory::submit! {
                            #![crate = ::panda]
                            ::panda::PPPCallbackSetup(
                                || {
                                    ::panda::plugins::syscalls2::SYSCALLS.$cb_name(#func);
                                }
                            )
                        }

                        #function
                    ).into()
                }
            }
        )*

        /// For internal use only
        #[proc_macro]
        #[doc(hidden)]
        pub fn generate_syscalls_callbacks(_: TokenStream) -> TokenStream {
            quote!(
                plugin_import!{
                    static SYSCALLS: Syscalls2 = extern "syscalls2" {
                        callbacks {
                            $(
                                fn $attr_name(
                                    $($arg_name : $arg),*
                                );
                            )*

                            fn on_all_sys_enter(cpu: &mut CPUState, pc: target_ulong, callno: target_ulong);
                            fn on_all_sys_return(cpu: &mut CPUState, pc: target_ulong, callno: target_ulong);
                        }
                    };
                }
            ).into()
        }

        /// Callback that runs when any syscall is entered
        ///
        /// ### Args
        ///
        /// * `cpu` - a reference to the currently executing [`CPUState`] object
        /// * `pc` - the current program counter of the system when the syscall callback is hit
        /// * `callno` - the syscall number called
        ///
        /// ### Example
        /// ```rust
        /// use panda::prelude::*;
        ///
        /// #[panda::on_all_sys_enter]
        /// fn callback(cpu: &mut CPUState, pc: target_ulong, callno: target_ulong) {
        ///     // do stuff
        /// }
        /// ```
        ///
        /// [`CPUState`]: https://docs.rs/panda-re/*/panda/prelude/struct.CPUState.html
        #[proc_macro_attribute]
        pub fn on_all_sys_enter(_: TokenStream, function: TokenStream) -> TokenStream {
            let mut function = syn::parse_macro_input!(function as syn::ItemFn);
            function.sig.abi = Some(syn::parse_quote!(extern "C"));
            let func = &function.sig.ident;
            let cfgs = crate::get_cfg_attrs(&function);

            quote!(
                #(
                    #cfgs
                 )*
                ::panda::inventory::submit! {
                    #![crate = ::panda]
                    ::panda::PPPCallbackSetup(
                        || {
                            ::panda::plugins::syscalls2::SYSCALLS.add_callback_on_all_sys_enter(#func);
                        }
                    )
                }

                #function
            ).into()
        }

        /// Callback that runs when any syscall returns.
        ///
        /// Note that some syscalls do not return and thus will not have this callback run.
        ///
        /// ### Args
        ///
        /// * `cpu` - a reference to the currently executing [`CPUState`] object
        /// * `pc` - the current program counter of the system when the syscall callback is hit
        /// * `callno` - the syscall number called
        ///
        /// ### Example
        /// ```rust
        /// use panda::prelude::*;
        ///
        /// #[panda::on_all_sys_return]
        /// fn callback(cpu: &mut CPUState, pc: target_ulong, callno: target_ulong) {
        ///     // do stuff
        /// }
        /// ```
        ///
        /// [`CPUState`]: https://docs.rs/panda-re/*/panda/prelude/struct.CPUState.html
        #[proc_macro_attribute]
        pub fn on_all_sys_return(_: TokenStream, function: TokenStream) -> TokenStream {
            let mut function = syn::parse_macro_input!(function as syn::ItemFn);
            function.sig.abi = Some(syn::parse_quote!(extern "C"));
            let func = &function.sig.ident;
            let cfgs = crate::get_cfg_attrs(&function);

            quote!(
                #(
                    #cfgs
                 )*
                ::panda::inventory::submit! {
                    #![crate = ::panda]
                    ::panda::PPPCallbackSetup(
                        || {
                            ::panda::plugins::syscalls2::SYSCALLS.add_callback_on_all_sys_return(#func);
                        }
                    )
                }

                #function
            ).into()
        }
    };
}

/// (Callback) Runs when proc_start_linux recieves the `AuxvValues` for a given process.
///
/// ### Args
///
/// * `cpu` - a reference to the currently executing [`CPUState`] object
/// * `tb` - the current [`TranslationBlock`] at time of recieving
/// * `auxv` - the auxillary vector ([`AuxvValues`]) of the program starting
///
/// ### Example
/// ```rust
/// use panda::prelude::*;
/// use panda::plugins::proc_start_linux::AuxvValues;
///
/// #[panda::on_rec_auxv]
/// fn callback(cpu: &mut CPUState, tb: &mut TranslationBlock, auxv: AuxvValues) {
///     // do stuff
/// }
/// ```
///
/// [`CPUState`]: https://docs.rs/panda-re/*/panda/prelude/struct.CPUState.html
/// [`TranslationBlock`]: https://docs.rs/panda-re/*/panda/prelude/struct.TranslationBlock.html
/// [`AuxvValues`]: https://docs.rs/panda-re/*/panda/plugins/proc_start_linux/struct.AuxvValues.html
#[proc_macro_attribute]
pub fn on_rec_auxv(_: TokenStream, function: TokenStream) -> TokenStream {
    let mut function = syn::parse_macro_input!(function as syn::ItemFn);
    function.sig.abi = Some(syn::parse_quote!(extern "C"));
    let func = &function.sig.ident;
    let cfgs = crate::get_cfg_attrs(&function);

    quote!(
        #(
            #cfgs
         )*
        ::panda::inventory::submit! {
            #![crate = ::panda]
            ::panda::PPPCallbackSetup(
                || {
                    ::panda::plugins::proc_start_linux::PROC_START_LINUX.add_callback_on_rec_auxv(#func);
                }
            )
        }

        #function
    ).into()
}

macro_rules! define_hooks2_callbacks {
    ($(
        $($doc:literal)*
        fn($cb_name:ident) $attr_name:ident ($($arg_name:ident : $arg:ty),* $(,)?);
    )*) => {
        $(
            doc_comment::doc_comment!{
                concat!("(Callback) ", $($doc, "\n",)* "\n\nCallback arguments: ("$(, "`", stringify!($arg), "`")*, ")\n### Example\n```rust\nuse panda::prelude::*;\n\n#[panda::", stringify!($attr_name),"]\nfn callback(", $(", _: ", stringify!($arg), )* ") {\n    // do stuff\n}\n```"),
                #[proc_macro_attribute]
                pub fn $attr_name(_: TokenStream, function: TokenStream) -> TokenStream {
                    let mut function = syn::parse_macro_input!(function as syn::ItemFn);
                    function.sig.abi = Some(syn::parse_quote!(extern "C"));
                    let func = &function.sig.ident;
                    let cfgs = crate::get_cfg_attrs(&function);

                    quote!(
                        #(
                            #cfgs
                         )*
                        ::panda::inventory::submit! {
                            #![crate = ::panda]
                            ::panda::PPPCallbackSetup(
                                || {
                                    ::panda::plugins::hooks2::HOOKS.$cb_name(#func);
                                }
                            )
                        }

                        #function
                    ).into()
                }
            }
        )*

        /// For internal use only
        #[doc(hidden)]
        #[proc_macro]
        pub fn generate_hooks2_callbacks(_: TokenStream) -> TokenStream {
            quote!(

                plugin_import!{
                    static HOOKS: Hooks2 = extern "hooks2" {
                        callbacks {
                            $(
                                fn $attr_name(
                                    $($arg_name : $arg),*
                                );
                            )*
                        }
                    };
                }
            ).into()
        }
    };
}

include!("base_callbacks.rs");
include!("hooks2.rs");

#[cfg(feature = "x86_64")]
include!("syscalls/x86_64.rs");

#[cfg(feature = "i386")]
include!("syscalls/i386.rs");

#[cfg(feature = "arm")]
include!("syscalls/arm.rs");

#[cfg(feature = "aarch64")]
include!("syscalls/aarch64.rs");

// PANDA doesn't have PPC syscalls support!
//#[cfg(feature = "ppc")]
//include!("syscalls/ppc.rs");

#[cfg(feature = "mips")]
include!("syscalls/mips.rs");

#[cfg(feature = "mipsel")]
include!("syscalls/mipsel.rs");

// PANDA doesn't have MIPS64 syscalls support
//#[cfg(feature = "mips64")]
//include!("syscalls/mips64.rs");
