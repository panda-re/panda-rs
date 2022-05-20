use darling::{FromDeriveInput, FromField};
use proc_macro::TokenStream;
use quote::{quote, ToTokens};
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
            pub unsafe extern "C" fn init_plugin(plugin: *mut ::panda::PluginHandle) -> bool {
                ::panda::set_plugin_ref(plugin);

                for cb in ::panda::inventory::iter::<::panda::InternalCallback> {
                    ::panda::sys::panda_register_callback(plugin as _, cb.cb_type, ::core::mem::transmute(cb.fn_pointer));
                }

                for cb in ::panda::inventory::iter::<::panda::PPPCallbackSetup> {
                    cb.0();
                }

                ::panda::InitReturn::into_init_bool(#func_name(#args))
            }

            #[no_mangle]
             pub unsafe extern "C" fn uninit_plugin(plugin: *mut ::panda::PluginHandle) {
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

/// An attribute to declare a function for hooking using the PANDA 'hooks' plugin,
/// enabling the ability to add callbacks for when a specifc instruction is hit,
/// with control over the address space, kernel mode, and callback type to use.
///
/// ## Example
///
/// ```
/// use panda::plugins::proc_start_linux::AuxvValues;
/// use panda::plugins::hooks::Hook;
/// use panda::prelude::*;
///
/// #[panda::hook]
/// fn entry_hook(_: &mut CPUState, _: &mut TranslationBlock, _: u8, hook: &mut Hook) {
///     println!("\n\nHit entry hook!\n");
///
///     // only run hook once
///     hook.enabled = false;
/// }
///
/// #[panda::on_rec_auxv]
/// fn on_proc_start(_: &mut CPUState, _: &mut TranslationBlock, auxv: &AuxvValues) {
///     // when a process starts, hook the entrypoint
///     entry_hook::hook()
///         .after_block_exec()
///         .at_addr(auxv.entry)
/// }
///
/// Panda::new()
///     .generic("x86_64")
///     .replay("test")
///     .run();
/// ```
///
/// ## Supported Callback Types
///
/// ### Standard callbacks
///
/// These callbacks take the form of:
///
/// ```
/// #[panda::hook]
/// fn my_callback(cpu: &mut CPUState, tb: &mut TranslationBlock, hook: &mut Hook);
/// ```
///
/// |         Callback        | Info |
/// |:------------------------|------|
/// | `before_tcg_codegen`    | Callback at the start of the tcg IR being generated |
/// | `after_block_translate` | Callback after the block the hooked instruction is in gets translated |
/// | `before_block_exec`     | Callback before the block the given instruction is in gets run |
/// | `start_block_exec`      | Callback at the first instruction in the block the instruction is in |
/// | `end_block_exec`        | Callback after the last instruction in the block the hooked instruction is in |
///
/// ### Other Callbacks
///
/// These callbacks each have their own unique required function signature.
///
/// |          Callback        | Required Signature | Info |
/// |:-------------------------|--------------------|------|
/// | `before_block_translate` | `fn(cpu: &mut CPUState, pc: target_ptr_t, hook: &mut Hook)` | Callback that runs before the block the hooked instruction is translated to tcg |
/// | `after_block_exec`       | `fn(cpu: &mut CPUState, tb: &mut TranslationBlock, exitCode: u8, hook: &mut Hook)` | Callback that runs after the given block is executed |
/// | `before_block_exec_invalidate_opt` | `fn(env: &mut CPUState, tb: &mut TranslationBlock, hook: &mut Hook) -> bool` | Callback on translate to provide the option to invalidate the block the hooked instruction is generated in |
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

mod guest_type;
use guest_type::GuestTypeInput;

#[proc_macro_derive(GuestType)]
pub fn derive_guest_type(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);
    match GuestTypeInput::from_derive_input(&input) {
        Ok(input) => input.to_tokens().into(),
        Err(err) => err.write_errors().into(),
    }
}

mod osi_type;
use osi_type::OsiTypeInput;

#[proc_macro_derive(OsiType, attributes(osi))]
pub fn derive_osi_type(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);
    match OsiTypeInput::from_derive_input(&input) {
        Ok(input) => input.to_tokens().into(),
        Err(err) => err.write_errors().into(),
    }
}

mod osi_static;
use osi_static::OsiStatics;

#[proc_macro]
pub fn osi_static(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as OsiStatics);

    input.into_token_stream().into()
}

#[proc_macro_attribute]
pub fn channel_recv(_: TokenStream, func: TokenStream) -> TokenStream {
    let mut func = syn::parse_macro_input!(func as syn::ItemFn);

    let name = std::mem::replace(&mut func.sig.ident, syn::parse_quote!(inner));

    quote!(
        extern "C" fn #name(channel_id: u32, data: *const u8, len: usize) {
            #func

            let msg = unsafe {
                ::panda::plugins::guest_plugin_manager::FromChannelMessage::from_channel_message(
                    data, len
                )
            };

            match msg {
                Ok(msg) => {
                    inner(
                        channel_id,
                        msg
                    );
                },
                Err(err) => {
                    println!("Warning: could not parse channel message, {}", err);
                }
            }
        }
    )
    .into()
}

// derive PandaArgs
include!("panda_args.rs");

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
        ($attr_name:ident, $const_name:ident, ($($arg_name:ident : $arg:ty),*) $(-> $ret:ty)?)
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
                            ::panda::InternalCallback::new(
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

        #[proc_macro]
        pub fn define_closure_callbacks(_: TokenStream) -> TokenStream {
            quote!(
                impl Callback {
                    $(
                        /// Installs the given callback, assigning it to this `Callback`'s
                        /// slot. Any callbacks previously stored in that slot will be
                        /// freed.
                        ///
                        $(
                            #[doc = $doc]
                        )*
                        pub fn $attr_name<F>(self, callback: F)
                            where F: FnMut($($arg),*) $(-> $ret)? + 'static
                        {
                            unsafe extern "C" fn trampoline(context: *mut c_void, $($arg_name: $arg),*) $(-> $ret)? {
                                let closure: &mut &mut (
                                    dyn FnMut($($arg),*) $(-> $ret)?
                                ) = unsafe { std::mem::transmute(
                                    context as *mut *mut c_void
                                )};
                                closure($($arg_name),*)
                            }

                            unsafe fn drop_fn(this: *mut *mut c_void) {
                                let _: Box<Box<dyn FnMut($($arg),*) $(-> $ret)?>> = unsafe {
                                    std::mem::transmute(this)
                                };
                            }

                            let closure_ref: *mut *mut c_void = unsafe {
                                let x: Box<Box<
                                    dyn FnMut($($arg),*) $(-> $ret)?
                                >> = Box::new(
                                    Box::new(callback) as Box<_>
                                );

                                std::mem::transmute(x)
                            };

                            install_closure_callback(self.0, ClosureCallback {
                                closure_ref,
                                drop_fn,
                                trampoline: sys::panda_cb_with_context {
                                    $attr_name: Some(unsafe {
                                        std::mem::transmute(trampoline as *const c_void)
                                    })
                                },
                                cb_kind: sys::$const_name,
                            });
                        }
                    )*
                }
            ).into()
        }
    }
}

#[cfg(not(feature = "ppc"))]
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
                    ")\n### Example\n```rust\nuse panda::prelude::*;\n\n#[panda::on_sys::",
                    stringify!($syscall_name),
                    "_enter",
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

                            fn on_all_sys_enter(cpu: &mut CPUState, pc: SyscallPc, callno: target_ulong);
                            fn on_all_sys_return(cpu: &mut CPUState, pc: SyscallPc, callno: target_ulong);
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

/// (Callback) Runs when proc_start_linux recieves the [`AuxvValues`] for a given process.
///
/// Can be treated as a "on program start" callback, but one which provides a lot of
/// info about the contents of the initial program state and how it is being loaded.
/// The state at time of callback is before the C runtime is initialized, and before
/// the entrypoint is jumped to.
///
/// See [`AuxvValues`] to get a better understanding of the values provided.
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
/// fn on_proc_start(cpu: &mut CPUState, tb: &mut TranslationBlock, auxv: AuxvValues) {
///     // do stuff when a process starts
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

#[cfg(feature = "mips64")]
include!("syscalls/mips64.rs");
