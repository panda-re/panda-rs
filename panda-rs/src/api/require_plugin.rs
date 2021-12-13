use crate::sys::{panda_add_arg, panda_load_plugin, panda_plugin_path};
use crate::PandaArgs;

use std::ffi::CString;

/// Require a plugin to be loaded, and if it isn't loaded load it with the given
/// arguments. If the plugin is already loaded the arguments will be discarded.
pub fn require_plugin<Args: PandaArgs>(plugin: &Args) {
    let plugin_name = CString::new(Args::PLUGIN_NAME).unwrap();

    let path = unsafe { panda_plugin_path(plugin_name.as_ptr()) };

    for (name, arg) in plugin.to_panda_args() {
        let arg = CString::new(format!("{}={}", name, arg)).unwrap();
        unsafe {
            panda_add_arg(plugin_name.as_ptr(), arg.as_ptr());
        }
    }

    unsafe {
        panda_load_plugin(path, plugin_name.as_ptr());
    }
}
