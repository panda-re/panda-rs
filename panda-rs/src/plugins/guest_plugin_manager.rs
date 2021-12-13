use crate::plugin_import;

use std::{
    ffi::{CStr, CString},
    io::Write,
    os::raw::c_char,
    path::{Path, PathBuf},
    ptr,
};

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

#[repr(transparent)]
pub struct Channel(ChannelId);

impl Channel {
    fn write_packet(&mut self, buf: &[u8]) {
        GUEST_PLUGIN_MANAGER.channel_write(self.0, buf.as_ptr(), buf.len());
    }
}

impl Write for Channel {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.write_packet(buf);

        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

pub fn load_guest_plugin(name: impl Into<String>, msg_received: ChannelCB) -> Channel {
    Channel(GUEST_PLUGIN_MANAGER.add_guest_plugin(GuestPlugin::new(name.into(), msg_received)))
}

impl GuestPlugin {
    pub fn new(plugin_name: String, msg_receive_cb: ChannelCB) -> Self {
        let plugin_name = CString::new(plugin_name).unwrap().into_raw();

        GuestPlugin {
            plugin_name,
            guest_binary_path: ptr::null(),
            msg_receive_cb,
        }
    }

    pub fn new_with_path(
        plugin_name: String,
        guest_binary_path: &Path,
        msg_receive_cb: ChannelCB,
    ) -> Self {
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

/// Get the guest plugin's path from its name
pub fn guest_plugin_path(name: &str) -> Option<PathBuf> {
    extern "C" {
        fn panda_guest_plugin_path(name: *const c_char) -> *mut c_char;
    }

    let name = CString::new(name).ok()?;
    let path_result = unsafe { panda_guest_plugin_path(name.as_ptr()) };

    if path_result.is_null() {
        None
    } else {
        let path = unsafe { CStr::from_ptr(path_result) };
        let path = path.to_str().ok().map(PathBuf::from);

        unsafe {
            panda::sys::free(path_result as _);
        }

        path
    }
}
