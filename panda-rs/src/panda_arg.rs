use crate::sys::*;
use std::ffi::{CStr, CString};

pub trait PandaArgs {
    fn from_panda_args() -> Self;

    fn to_panda_args_str(&self) -> std::string::String;
}

pub trait GetPandaArg {
    fn get_panda_arg(args: *mut panda_arg_list, name: &str, default: Self, description: &str, required: bool) -> Self;
}



impl GetPandaArg for u64 {
    fn get_panda_arg(args: *mut panda_arg_list, name: &str, default: Self, description: &str, required: bool) -> Self {
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
    fn get_panda_arg(args: *mut panda_arg_list, name: &str, default: Self, description: &str, required: bool) -> Self {
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
    fn get_panda_arg(args: *mut panda_arg_list, name: &str, default: Self, description: &str, required: bool) -> Self {
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
    fn get_panda_arg(args: *mut panda_arg_list, name: &str, default: Self, description: &str, required: bool) -> Self {
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
    fn get_panda_arg(args: *mut panda_arg_list, name: &str, default: Self, description: &str, required: bool) -> Self {
        let name = CString::new(name).unwrap();
        let desc = CString::new(description).unwrap();
        let default = CString::new(default).unwrap();

        unsafe {
            if required {
                CStr::from_ptr(
                    panda_parse_string_req(args, name.as_ptr(), desc.as_ptr())
                ).to_str().unwrap().to_owned()
            } else {
                CStr::from_ptr(
                    panda_parse_string_opt(args, name.as_ptr(), default.as_ptr(), desc.as_ptr())
                ).to_str().unwrap().to_owned()
            }
        }
    }
}
