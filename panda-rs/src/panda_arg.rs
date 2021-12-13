use crate::sys::*;
use std::ffi::{CStr, CString};

/// A trait for allowing conversion to and from PANDA command line arguments. Should only be used
/// with the provided derive macro.
///
/// ### Example
/// ```rust
/// use panda::prelude::*;
///
/// #[derive(PandaArgs)]
/// #[name = "my_plugin"]
/// struct MyPluginArgs {
///     file: String,
/// }
///
/// let args = MyPluginArgs::from_panda_args();
/// ```
pub trait PandaArgs {
    const PLUGIN_NAME: &'static str;

    /// Get an instance of this struct from the PANDA arguments for the given plugin
    fn from_panda_args() -> Self;

    /// Convert this struct into a string to be passed via PANDA command line arguments.
    ///
    /// Used internally by `Panda::plugin_args`.
    fn to_panda_args_str(&self) -> std::string::String;

    /// Convert this struct into a set of argument pairs to be passed to PANDA
    ///
    /// Used internally by `plugin_require`
    fn to_panda_args(&self) -> Vec<(&'static str, std::string::String)>;
}

/// A wrapper trait for getting a PANDA argument as a given type. Used internally by the `PandaArgs`
/// derive macro.
pub trait GetPandaArg {
    fn get_panda_arg(
        args: *mut panda_arg_list,
        name: &str,
        default: Self,
        description: &str,
        required: bool,
    ) -> Self;
}

impl GetPandaArg for bool {
    fn get_panda_arg(
        args: *mut panda_arg_list,
        name: &str,
        _default: Self,
        description: &str,
        required: bool,
    ) -> Self {
        let name = CString::new(name).unwrap();
        let desc = CString::new(description).unwrap();

        unsafe {
            if required {
                panda_parse_bool_req(args, name.as_ptr(), desc.as_ptr())
            } else {
                panda_parse_bool_opt(args, name.as_ptr(), desc.as_ptr())
            }
        }
    }
}

impl GetPandaArg for u64 {
    fn get_panda_arg(
        args: *mut panda_arg_list,
        name: &str,
        default: Self,
        description: &str,
        required: bool,
    ) -> Self {
        let name = CString::new(name).unwrap();
        let desc = CString::new(description).unwrap();

        unsafe {
            if required {
                panda_parse_uint64_req(args, name.as_ptr(), desc.as_ptr())
            } else {
                panda_parse_uint64_opt(args, name.as_ptr(), default, desc.as_ptr())
            }
        }
    }
}

impl GetPandaArg for u32 {
    fn get_panda_arg(
        args: *mut panda_arg_list,
        name: &str,
        default: Self,
        description: &str,
        required: bool,
    ) -> Self {
        let name = CString::new(name).unwrap();
        let desc = CString::new(description).unwrap();

        unsafe {
            if required {
                panda_parse_uint32_req(args, name.as_ptr(), desc.as_ptr())
            } else {
                panda_parse_uint32_opt(args, name.as_ptr(), default, desc.as_ptr())
            }
        }
    }
}

impl GetPandaArg for f64 {
    fn get_panda_arg(
        args: *mut panda_arg_list,
        name: &str,
        default: Self,
        description: &str,
        required: bool,
    ) -> Self {
        let name = CString::new(name).unwrap();
        let desc = CString::new(description).unwrap();

        unsafe {
            if required {
                panda_parse_double_req(args, name.as_ptr(), desc.as_ptr())
            } else {
                panda_parse_double_opt(args, name.as_ptr(), default, desc.as_ptr())
            }
        }
    }
}

impl GetPandaArg for f32 {
    fn get_panda_arg(
        args: *mut panda_arg_list,
        name: &str,
        default: Self,
        description: &str,
        required: bool,
    ) -> Self {
        let name = CString::new(name).unwrap();
        let desc = CString::new(description).unwrap();

        unsafe {
            if required {
                panda_parse_double_req(args, name.as_ptr(), desc.as_ptr()) as f32
            } else {
                panda_parse_double_opt(args, name.as_ptr(), default as f64, desc.as_ptr()) as f32
            }
        }
    }
}

impl GetPandaArg for std::string::String {
    fn get_panda_arg(
        args: *mut panda_arg_list,
        name: &str,
        default: Self,
        description: &str,
        required: bool,
    ) -> Self {
        let name = CString::new(name).unwrap();
        let desc = CString::new(description).unwrap();
        let default = CString::new(default).unwrap();

        unsafe {
            if required {
                CStr::from_ptr(panda_parse_string_req(args, name.as_ptr(), desc.as_ptr()))
                    .to_str()
                    .unwrap()
                    .to_owned()
            } else {
                CStr::from_ptr(panda_parse_string_opt(
                    args,
                    name.as_ptr(),
                    default.as_ptr(),
                    desc.as_ptr(),
                ))
                .to_str()
                .unwrap()
                .to_owned()
            }
        }
    }
}
