//! panda-rs is a set of Rust bindings for PANDA.
//!
//! **The following are provided:**
//! * Callbacks in the form of attribute macros
//! * Access to raw PANDA API bindings via panda_sys
//!
//! ### Feature flags:
//! * `libpanda` - enable libpanda mode. This is used to allow for compiling as a binary that links
//! against libpanda, for pypanda-style use.

/// Raw bindings to the PANDA API
pub use panda_sys as sys;
pub use panda_macros::*;

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
