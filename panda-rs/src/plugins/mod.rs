use std::path::Path;
use std::ffi::CString;
use libloading::Symbol;
use crate::sys::panda_require;

pub mod glib;
pub mod osi;
pub mod hooks2;
pub mod syscalls2;
pub mod proc_start_linux;

#[macro_export]
macro_rules! plugin_import {
    {
        static $static:ident : $ty:ident = extern $name:literal {
        $(
            $(
                #[$meta:meta]
             )*
            fn $fn_name:ident
                $(
                    <
                        $(
                            $lifetimes:lifetime
                        ),*
                        $(,)?
                    >
                )?
            (
                $(
                    $arg_name:ident : $arg_ty:ty
                 ),*
                $(,)?
            ) $(-> $fn_ret:ty)?;
         )*
        $(
            callbacks {
                $(
                    fn $cb_fn_name:ident(
                        $(
                            $cb_arg_name:ident : $cb_arg_ty:ty
                         ),*
                        $(,)?
                    ) $(-> $cb_fn_ret:ty)?;
                )*
            }
        )?
        };
    } => {
        pub struct $ty {
            plugin: $crate::plugins::Plugin
        }

        impl $ty {
            /// Create a new handle to this plugin
            pub fn new() -> Self {
                Self {
                    plugin: $crate::plugins::Plugin::new($name)
                }
            }

            /// Load the plugin and initialize it if it hasn't been loaded already.
            pub fn ensure_init(&self) {}

            $(
                $(
                    #[$meta]
                 )*
                pub fn $fn_name $(< $($lifetimes),* >)? (&self $(, $arg_name : $arg_ty )*) $(-> $fn_ret)? {
                    unsafe {
                        self.plugin.get::<unsafe extern "C" fn($($arg_ty),*) $(-> $fn_ret)?>(
                            stringify!($fn_name)
                        )(
                            $(
                                $arg_name
                            ),*
                        )
                    }
                }
             )*

            $($(
                ::paste::paste!{
                    pub fn [<add_callback_ $cb_fn_name>](
                        &self,
                        callback: extern "C" fn(
                            $($cb_arg_name: $cb_arg_ty),*
                        )
                    )
                    {
                        let add_cb = self.plugin.get::<
                            extern "C" fn(
                                extern "C" fn(
                                    $($cb_arg_ty),*
                                ) $(-> $cb_fn_ret)?
                            )
                        >(
                            concat!("ppp_add_cb_", stringify!($cb_fn_name))
                        );

                        add_cb(callback);
                    }
                }
            )*)?
        }

        lazy_static::lazy_static!{
            pub static ref $static: $ty = $ty::new();
        }
    }
}

struct Plugin {
    lib: libloading::Library,
}

#[cfg(feature = "x86_64")]
const PLUGIN_DIR: &str = "x86_64-softmmu/panda/plugins";

#[cfg(feature = "i386")]
const PLUGIN_DIR: &str = "i386-softmmu/panda/plugins";

#[cfg(feature = "arm")]
const PLUGIN_DIR: &str = "arm-softmmu/panda/plugins";

#[cfg(feature = "mips")]
const PLUGIN_DIR: &str = "mips-softmmu/panda/plugins";

#[cfg(feature = "mipsel")]
const PLUGIN_DIR: &str = "mipsel-softmmu/panda/plugins";

#[cfg(feature = "ppc")]
const PLUGIN_DIR: &str = "ppc-softmmu/panda/plugins";

impl Plugin {
    pub fn new(name: &str) -> Self {
        std::env::set_var("PANDA_DIR", std::env::var("PANDA_PATH").expect("Missing PANDA_PATH"));
        let c_name = CString::new(name).unwrap();
        unsafe {
            panda_require(c_name.as_ptr());
        }
        let path = 
            Path::new(&std::env::var("PANDA_PATH").unwrap())
                .join(&std::env::var("PANDA_PLUGIN_DIR").unwrap_or(PLUGIN_DIR.to_owned()))
                .join(&format!("panda_{}.so", name));
        Self {
            lib: libloading::Library::new(
                path
            ).expect("Failed to load plugin")
        }
    }

    pub fn get<T>(&self, sym: &str) -> Symbol<T> {
        let symbol: Vec<_> = sym.bytes().chain(std::iter::once(0)).collect();
        unsafe {
            self.lib.get(&symbol).expect("Could not find symbol")
        }
    }
}
