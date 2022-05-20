use std::{ffi::CString, io::Write, os::raw::c_char, path::Path, ptr};

use super::GUEST_PLUGIN_MANAGER;

/// A raw Channel ID
pub type ChannelId = u32;

/// A callback for recieving writes to a channel performed by the guest
pub type ChannelCB = extern "C" fn(ChannelId, *const u8, usize);

/// A guest plugin to be loaded by the guest plugin manager
#[repr(C)]
pub struct GuestPlugin {
    pub plugin_name: *const c_char,
    pub guest_binary_path: *const c_char,
    pub msg_receive_cb: ChannelCB,
}

/// An [`io::Write`](Write) type for writing to a guest plugin channel
#[repr(transparent)]
pub struct Channel(ChannelId);

impl Channel {
    /// Write data to a single packet without going through the io::Write trait
    pub fn write_packet(&mut self, buf: &[u8]) {
        GUEST_PLUGIN_MANAGER.channel_write(self.0, buf.as_ptr(), buf.len());
    }

    /// Creates a new anonymous channel provided a callback for handling writes
    pub fn new(callback: ChannelCB) -> Self {
        Channel(GUEST_PLUGIN_MANAGER.allocate_channel(callback))
    }

    /// Get the raw channel ID of this channel
    pub fn id(&self) -> ChannelId {
        self.0
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

/// Load a guest plugin given the guest plugin's name and a callback for when a message
/// is recieved from this plugin.
///
/// Returns a channel with the same name as the plugin for use when communicating with
/// the guest plugin.
pub fn load_guest_plugin(name: impl Into<String>, msg_received: ChannelCB) -> Channel {
    Channel(GUEST_PLUGIN_MANAGER.add_guest_plugin(GuestPlugin::new(name.into(), msg_received)))
}

impl GuestPlugin {
    /// Initiailizes a `GuestPlugin` to be passed to `add_guest_plugin` by name, finding
    /// the path of the plugin by name lookup.
    pub fn new(plugin_name: String, msg_receive_cb: ChannelCB) -> Self {
        let plugin_name = CString::new(plugin_name).unwrap().into_raw();

        GuestPlugin {
            plugin_name,
            guest_binary_path: ptr::null(),
            msg_receive_cb,
        }
    }

    /// Initiailizes a `GuestPlugin` to be passed to `add_guest_plugin` by name, using
    /// a set path rather than lookup by name.
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
