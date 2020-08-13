//! panda-rs is a set of Rust bindings for PANDA.
//!
//! **The following are provided:**
//! * Callbacks in the form of attribute macros
//! * Access to raw PANDA API bindings via panda_sys
//!
//! ### Feature flags:
//! * `libpanda` - enable libpanda mode. This is used to allow for compiling as a binary that links
//! against libpanda, for pypanda-style use.
//!
//! ### Callbacks
//! panda-rs makes extensive use of callbacks for handling analyses on various events. To use
//! callbacks, you simply apply the callback's attribute to any functions which should be called
//! for the given callback. In order to use a callback in a PANDA plugin (not to be confused with
//! an application that uses libpanda), one plugin must be marked `#[panda::init]`, otherwise the
//! plugin will not work in PANDA.
//!
//! ### libpanda Mode
//!
//! PANDA also offers a dynamic library (libpanda). panda-rs allows linking against libpanda
//! instead of linking as a PANDA plugin. This creates a executable that requires libpanda to run.
//! To compile in libpanda mode, make sure the `PANDA_PATH` environment variable is set to your
//! PANDA `build` folder.

/// Raw bindings to the PANDA API
pub use panda_sys as sys;
pub use panda_macros::*;
pub use panda_macros as base_callbacks;

/// Safe wrappers for the libpanda API for helping create and manage an instance of the PANDA API
mod library_mode;
pub use library_mode::*;

/// For internal use. Access to inventory for managing callbacks.
pub use inventory;

/// An opaque type used to register/unregister callbacks with PANDA. Passed into init/unit
/// callbacks
pub struct PluginHandle;

/// A typeless PANDA callback used internally by callback attributes. Not recommended for direct
/// use.
pub struct Callback {
    pub cb_type: sys::panda_cb_type,
    pub fn_pointer: *const (),
}

impl Callback {
    pub fn new(cb_type: sys::panda_cb_type, fn_pointer: *const ()) -> Self {
        Self { cb_type, fn_pointer }
    }
}

/// A callback set to run on plugin uninit. To add an uninit callback use `#[panda::uninit]` on a
/// function which takes an `&mut PluginHandle` as an argument.
///
/// ### Example
///
/// ```rust
/// use panda::PluginHandle;
///
/// #[panda::uninit]
/// fn on_exit(plugin: &mut PluginHandle) {
///     // Do stuff
/// }
/// ```
pub struct UninitCallback(pub fn(&mut PluginHandle));

inventory::collect!(Callback);
inventory::collect!(UninitCallback);

/// Helpers for getting plugin arguments from panda
pub mod panda_arg;
pub use panda_arg::PandaArgs;

pub mod prelude {
    pub use crate::Panda;
    pub use crate::PluginHandle;
    pub use crate::sys::CPUState;
    pub use crate::sys::TranslationBlock;
    pub use crate::panda_arg::PandaArgs;
    pub use panda_macros::PandaArgs;
}