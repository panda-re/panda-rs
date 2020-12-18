//use crate::enums::GenericRet;
//use std::ffi::CString;
//use std::path::Path;
//use llvm_ir::Module;

/// Enable translating TCG -> LLVM and executing LLVM
pub fn enable_llvm() {
    unsafe {
        panda_sys::panda_enable_llvm()
    }
}

/// Enable translating TCG -> LLVM, but still execute TCG
pub fn enable_llvm_no_exec() {
    unsafe {
        panda_sys::panda_enable_llvm_no_exec()
    }
}

/// Disable LLVM translation and execution
pub fn disable_llvm() {
    unsafe {
        panda_sys::panda_disable_llvm()
    }
}

/// Enable LLVM helpers
pub fn enable_llvm_helpers() {
    unsafe {
        panda_sys::panda_enable_llvm_helpers()
    }
}

/// Disable LLVM helpers
pub fn disable_llvm_helpers() {
    unsafe {
        panda_sys::panda_disable_llvm_helpers()
    }
}

/*
// TODO: Fix and test
/// Get current (last translated) LLVM module.
pub fn get_current_llvm_mod() -> Result<Module, String> {

    // Try three RAM-backed Linux dirs (for speed), fallback to OS-agnostic temp dir
    let file_path = if Path::new("/dev/run").exists() {
        Path::new("/dev/run/curr_llvm.bc")
    } else if Path::new("/run/shm").exists() {
        Path::new("/run/shm/curr_llvm.bc")
    } else if Path::new("/dev/shm").exists() {
        Path::new("/dev/shm/curr_llvm.bc")
    } else {
        let mut path_buf = std::env::temp_dir();
        path_buf.push("curr_llvm.bc");
        path_buf.as_path()
    };

    if let Some(path_str) = file_path.to_str() {
        if let Ok(path_c_str) = CString::new(path_str.as_bytes()) {
            unsafe {
                match panda_sys::panda_write_current_llvm_bitcode_to_file(
                    path_c_str.as_ptr()
                ).into() {
                    GenericRet::Success => Module::from_bc_path(file_path),
                    GenericRet::Error | GenericRet::Unknown => Err("Failed to write bitcode file".to_string())
                }
            }
        } else {
            Err(format!("Failed to convert path \'{:?}\' to C string!", file_path))
        }
    } else {
        Err(format!("Failed to convert path \'{:?}\' to string!", file_path))
    }
}
*/