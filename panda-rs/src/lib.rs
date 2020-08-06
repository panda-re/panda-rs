pub use panda_sys as sys;
pub use panda_macros::*;
pub use inventory;

pub struct PluginHandle;

pub struct Callback {
    pub cb_type: sys::panda_cb_type,
    pub fn_pointer: *const (),
}

impl Callback {
    pub fn new(cb_type: sys::panda_cb_type, fn_pointer: *const ()) -> Self {
        Self { cb_type, fn_pointer }
    }
}

pub struct UninitCallback(pub fn(&mut PluginHandle));

inventory::collect!(Callback);
inventory::collect!(UninitCallback);

#[macro_export]
macro_rules! plugin {
    (
        static $plugin:ident: $ty:ty = $expr:expr;
    ) => {
        static $plugin: $ty = $expr;

        #[no_mangle]
        unsafe extern "C" fn init_plugin(plugin: *mut Plugin) -> bool {
            $plugin.init(&mut *plugin);

            true
        }

        #[no_mangle]
        extern "C" fn uninit_plugin(plugin: *mut c_void) {
            $plugin.uninit(&mut *plugin);
        }
    }
}
