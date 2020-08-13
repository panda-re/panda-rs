use crate::sys::panda_cb_type;

/// An opaque type used to register/unregister callbacks with PANDA. Passed into init/unit
/// callbacks
pub struct PluginHandle;

/// A typeless PANDA callback used internally by callback attributes. Not recommended for direct
/// use.
pub struct Callback {
    pub cb_type: panda_cb_type,
    pub fn_pointer: *const (),
}

impl Callback {
    pub fn new(cb_type: panda_cb_type, fn_pointer: *const ()) -> Self {
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

pub struct PPPCallbackSetup(pub fn());

inventory::collect!(Callback);
inventory::collect!(UninitCallback);
inventory::collect!(PPPCallbackSetup);
