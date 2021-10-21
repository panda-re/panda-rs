use crate::sys::panda_require;
use libloading::Symbol;
use std::ffi::CString;
use std::path::Path;

pub mod glib;
pub mod guest_plugin_manager;
pub mod hooks;
pub mod hooks2;
pub mod osi;
pub mod proc_start_linux;

#[cfg(not(feature = "ppc"))]
pub mod syscalls2;

/// A macro for importing an external PANDA plugin to use
///
/// **Note:** it is recommended that, if the plugin you want to use already has
/// panda-rs bindings, they should be used instead. Those are located in the
/// [`plugins`](crate::plugins) module, and typically include a note about where
/// the high-level bindings for the given plugin are located.
///
/// ## Example Usage
///
/// ### Declaring bindings for free function in an external plugin:
///
/// ```
/// plugin_import!{
///     static OSI: Osi = extern "osi" {
///         fn get_process_handles(cpu: *mut CPUState) -> GBoxedSlice<OsiProcHandle>;
///         fn get_current_thread(cpu: *mut CPUState) -> GBox<OsiThread>;
///         fn get_modules(cpu: *mut CPUState) -> GBoxedSlice<OsiModule>;
///         fn get_mappings(cpu: *mut CPUState, p: *mut OsiProc) -> GBoxedSlice<OsiModule>;
///         fn get_processes(cpu: *mut CPUState) -> GBoxedSlice<OsiProc>;
///         fn get_current_process(cpu: *mut CPUState) -> GBox<OsiProc>;
///     };
/// }
/// ```
///
/// This will create a lazy initialized static variable named `OSI` in the current
/// scope. This static will include all of the functions listed as methods, when
/// any function is run the plugin (the name of which is specified by `extern "osi"`)
/// will be loaded on the fly before executing the method.
///
/// To load a plugin without running any function, `plugin_import` also automatically
/// creates an `ensure_init` method which initializes the plugin without any other
/// side effects.
///
/// ### Plugin Callbacks
///
/// Plugin-to-Plugin callbacks in PANDA are typically quite verbose to make bindings for
/// by hand, so the `plugin_import` macro provides a shorthand for defining a function
/// prototype for the callback and it will generate all the code needed to add and remove
/// callbacks for it.
///
/// ```
/// plugin_import! {
///     static PROC_START_LINUX: ProcStartLinux = extern "proc_start_linux" {
///         callbacks {
///             fn on_rec_auxv(cpu: &mut CPUState, tb: &mut TranslationBlock, auxv: &AuxvValues);
///         }
///     };
/// }
/// ```
///
/// the above creates another lazy static which has the following methods for working with
/// the `on_rec_auxv` callback:
///
/// * `add_callback_on_rec_auxv` - add a callback by function pointer
/// * `remove_callback_on_rec_auxv` - remove a callback by function pointer
///
/// One requirement of these function pointers is that they must use the C ABI. So the
/// argument for both methods would be of the type:
///
/// ```
/// extern "C" fn (&mut CPUState, &mut TranslationBlock, &AuxvValues)
/// ```
#[macro_export]
macro_rules! plugin_import {
    {
        $(
            #[ $type_meta:meta ]
        )*
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
        $(
            #[ $type_meta ]
        )*
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
                $crate::paste::paste!{
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

                $crate::paste::paste!{
                    pub fn [<remove_callback_ $cb_fn_name>](
                        &self,
                        callback: extern "C" fn(
                            $($cb_arg_name: $cb_arg_ty),*
                        )
                    )
                    {
                        let remove_cb = self.plugin.get::<
                            extern "C" fn(
                                extern "C" fn(
                                    $($cb_arg_ty),*
                                ) $(-> $cb_fn_ret)?
                            )
                        >(
                            concat!("ppp_remove_cb_", stringify!($cb_fn_name))
                        );

                        remove_cb(callback);
                    }
                }
            )*)?
        }

        $crate::lazy_static::lazy_static!{
            $(
                #[ $type_meta ]
            )*
            pub static ref $static: $ty = $ty::new();
        }
    }
}

pub struct Plugin {
    lib: libloading::Library,
}

#[cfg(feature = "x86_64")]
const PLUGIN_DIR: &str = "x86_64-softmmu/panda/plugins";

#[cfg(feature = "i386")]
const PLUGIN_DIR: &str = "i386-softmmu/panda/plugins";

#[cfg(feature = "arm")]
const PLUGIN_DIR: &str = "arm-softmmu/panda/plugins";

#[cfg(feature = "aarch64")]
const PLUGIN_DIR: &str = "aarch64-softmmu/panda/plugins";

#[cfg(feature = "mips")]
const PLUGIN_DIR: &str = "mips-softmmu/panda/plugins";

#[cfg(feature = "mipsel")]
const PLUGIN_DIR: &str = "mipsel-softmmu/panda/plugins";

#[cfg(feature = "mips64")]
const PLUGIN_DIR: &str = "mips64-softmmu/panda/plugins";

#[cfg(feature = "ppc")]
const PLUGIN_DIR: &str = "ppc-softmmu/panda/plugins";

impl Plugin {
    pub fn new(name: &str) -> Self {
        std::env::set_var(
            "PANDA_DIR",
            std::env::var("PANDA_PATH").expect("Missing PANDA_PATH"),
        );
        let c_name = CString::new(name).unwrap();
        unsafe {
            panda_require(c_name.as_ptr());
        }
        let path = Path::new(&std::env::var("PANDA_PATH").unwrap())
            .join(&std::env::var("PANDA_PLUGIN_DIR").unwrap_or(PLUGIN_DIR.to_owned()))
            .join(&format!("panda_{}.so", name));
        Self {
            lib: libloading::Library::new(path).expect("Failed to load plugin"),
        }
    }

    pub fn get<T>(&self, sym: &str) -> Symbol<T> {
        let symbol: Vec<_> = sym.bytes().chain(std::iter::once(0)).collect();
        unsafe { self.lib.get(&symbol).expect("Could not find symbol") }
    }
}
