use std::str;

/// Represents a type which can be converted to from a channel message sent by a
/// guest plugin. Used by the [`channel_recv`](super::channel_recv) macro.
///
/// ## Supported Types
/// * `&[u8]` - gives you the raw bytes
/// * `Vec<u8>` - gives you the raw bytes, but owned
/// * `&str` - gives you the bytes as a UTF-8 string. Prints a warning if invalid
/// unicode and skips your callback.
/// * `String` - same as `&str` but owned
/// * `Option<T>` - rather than print a warning if the type can't be decoded, pass None
/// * `Result<T, String>` - rather than print out the warning to stdout, pass the warning
/// as a `String`
pub trait FromChannelMessage: Sized {
    unsafe fn from_channel_message(data: *const u8, size: usize) -> Result<Self, String>;
}

impl<'a> FromChannelMessage for &'a [u8] {
    unsafe fn from_channel_message(data: *const u8, size: usize) -> Result<Self, String> {
        Ok(std::slice::from_raw_parts(data, size))
    }
}

impl FromChannelMessage for Vec<u8> {
    unsafe fn from_channel_message(data: *const u8, size: usize) -> Result<Self, String> {
        <&[u8]>::from_channel_message(data, size).map(ToOwned::to_owned)
    }
}

impl<'a> FromChannelMessage for &'a str {
    unsafe fn from_channel_message(data: *const u8, size: usize) -> Result<Self, String> {
        <&[u8]>::from_channel_message(data, size)
            .map(str::from_utf8)?
            .map_err(|_| String::from("Channel message is not valid UTF-8"))
    }
}

impl FromChannelMessage for String {
    unsafe fn from_channel_message(data: *const u8, size: usize) -> Result<Self, String> {
        <&str>::from_channel_message(data, size).map(ToOwned::to_owned)
    }
}

impl<T: FromChannelMessage> FromChannelMessage for Option<T> {
    unsafe fn from_channel_message(data: *const u8, size: usize) -> Result<Self, String> {
        Ok(T::from_channel_message(data, size).ok())
    }
}

impl<T: FromChannelMessage> FromChannelMessage for Result<T, String> {
    unsafe fn from_channel_message(data: *const u8, size: usize) -> Result<Self, String> {
        Ok(T::from_channel_message(data, size))
    }
}
