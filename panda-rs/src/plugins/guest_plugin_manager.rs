//! Bindings for the guest plugin manager
//!
//! The guest plugin manager is a PANDA plugin which manages "guest plugins", or programs
//! which are injected into the guest and can communicate back to the host.
//!
//! See [`load_guest_plugin`] and [`channel_recv`] for more info.
use crate::plugin_import;

use std::{
    ffi::{CStr, CString},
    os::raw::c_char,
    path::PathBuf,
};

mod guest_plugin;
pub use guest_plugin::{load_guest_plugin, Channel, ChannelCB, ChannelId, GuestPlugin};

mod from_channel_msg;
pub use from_channel_msg::FromChannelMessage;

/// Allows declaring a callback for recieving messages from a channel
///
/// Support functions with the signature `fn(u32, Msg)` where `u32` is the ID of the
/// channel being written to, while `Msg` is a type that implements [`FromChannelMessage`]
/// (&str, &[u8], String, etc).
///
/// ## Example
///
/// ```
/// use panda::plugins::guest_plugin_manager::{load_guest_plugin, channel_recv};
///
/// // Print every message and which channel it's sent to
/// #[channel_recv]
/// fn receive_message_callback(channel: u32, message: &str) {
///     println!("[channel {}] {}", channel, message);
/// }
///
/// // Alternatively, use `Option<T>`/`Result<T, String>` to opt-in to handling invalid unicode
/// #[channel_recv]
/// fn receive_message_callback(channel: u32, message: Option<&str>) {
///     if let Some(msg) = message {
///         println!("[channel {}] {}", channel, msg);
///     }
/// }
///
/// // Or just ask for raw bytes
/// #[channel_recv]
/// fn receive_message_callback(_: u32, message: &[u8]) {
///     println!("Message length: {}", message.len());
/// }
///
/// #[panda::init]
/// fn init() {
///     load_guest_plugin("my_guest_plugin", receive_message_callback);
/// }
/// ```
#[doc(inline)]
pub use panda_macros::channel_recv;

plugin_import! {
    /// A PANDA plugin which manages "guest plugins", programs which are injected into
    /// the guest which can communicate with the host process via "channels".test
    ///
    /// Unless you need greater control, it is recommended to use `load_guest_plugin` rather
    /// than using the `GUEST_PLUGIN_MANAGER` object directly.
    static GUEST_PLUGIN_MANAGER: GuestPluginManager = extern "guest_plugin_manager" {
        /// Add a guest plugin to the guest plugin manager, loading it into the guest
        /// as soon as possible.
        fn add_guest_plugin(plugin: GuestPlugin) -> ChannelId;

        /// Write to a channel, buffering the message until the guest performs a read
        /// to the channel.
        fn channel_write(channel: ChannelId, out: *const u8, out_len: usize);

        /// Get a channel given a name, typically the name of the guest plugin it is
        /// associated with, as each guest plugin is allocated a "main" channel of the
        /// same name.
        fn get_channel_from_name(channel_name: *const c_char) -> ChannelId;

        /// Create a new channel given a callback for handling writes, returns the ID
        /// of the newly allocated channel.
        fn allocate_channel(callback: ChannelCB) -> ChannelId;
    };
}

/// Get the guest plugin's path from its name, returning `None` if the guest plugin
/// could not be found.
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
