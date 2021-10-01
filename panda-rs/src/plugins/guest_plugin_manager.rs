use crate::plugin_import;
use std::ffi::CString;
use std::os::raw::c_char;
use std::path::Path;

plugin_import! {
    static GUEST_PLUGIN_MANAGER: GuestPluginManager = extern "guest_plugin_manager" {
        fn add_guest_plugin(plugin: GuestPlugin) -> ChannelId;
        fn channel_write(channel: ChannelId, out: *const u8, out_len: usize);
        fn get_channel_from_name(channel_name: *const c_char) -> ChannelId;
    };
}

pub type ChannelId = u32;
pub type ChannelCB = extern "C" fn(ChannelId, *const u8, usize);

#[repr(C)]
pub struct GuestPlugin {
    pub plugin_name: *const c_char,
    pub guest_binary_path: *const c_char,
    pub msg_receive_cb: ChannelCB,
}

impl GuestPlugin {
    pub fn new(plugin_name: String, guest_binary_path: &Path, msg_receive_cb: ChannelCB) -> Self {
        let plugin_name = CString::new(plugin_name).unwrap().into_raw();
        let guest_binary_path = CString::new(guest_binary_path.to_string_lossy().into_owned())
            .unwrap()
            .into_raw();

        GuestPlugin {
            plugin_name,
            guest_binary_path,
            msg_receive_cb,
        }
    }
}
