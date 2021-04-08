use crate::prelude::*;
use std::os::raw::c_int;
use std::ffi::CStr;

/// Determine if guest is currently executing in kernel mode
pub fn in_kernel_mode(cpu: &mut CPUState) -> bool {
    unsafe {
        panda_sys::panda_in_kernel_mode_external(cpu)
    }
}

/// Determine if guest is currently executing kernel code
pub fn in_kernel_code_linux(cpu: &mut CPUState) -> bool {
    unsafe {
        panda_sys::panda_in_kernel_code_linux_external(cpu)
    }
}

/// Get current architecture independent Address-Space ID (ASID)
pub fn current_asid(cpu: &mut CPUState) -> target_ulong {
    unsafe {
        panda_sys::panda_current_asid(cpu)
    }
}

/// Get current guest program counter
pub fn current_pc(cpu: &mut CPUState) -> target_ulong {
    unsafe {
        panda_sys::panda_current_pc(cpu)
    }
}

/// Get current guest userspace stack pointer
pub fn current_sp(cpu: &mut CPUState) -> target_ulong {
    unsafe {
        panda_sys::panda_current_sp_external(cpu)
    }
}

/// Get current guest userspace stack pointer, masking of page size MSBs
pub fn current_sp_masked_pagesize(cpu: &mut CPUState, page_size: target_ulong) -> target_ulong {
    unsafe {
        panda_sys::panda_current_sp_masked_pagesize_external(cpu, page_size)
    }
}

/// Get current guest kernelspace stack pointer
pub fn current_ksp(cpu: &mut CPUState) -> target_ulong {
    unsafe {
        panda_sys::panda_current_ksp_external(cpu)
    }
}

/// Get current guest function return value
pub fn get_ret_val(cpu: &mut CPUState) -> target_ulong {
    unsafe {
        panda_sys::panda_get_retval_external(cpu)
    }
}

/// If required for the target architecture, enter into a high-privilege mode in order to conduct some memory access.
/// Returns true if a switch into high-privilege mode has been made.
/// A NO-OP on systems where such changes are unnecessary.
pub fn enter_priv(cpu: &mut CPUState) -> bool {
    unsafe {
        panda_sys::enter_priv(cpu)
    }
}

/// Revert the guest to the privilege mode it was in prior to the last call to enter_priv().
/// A NO-OP for architectures where enter_priv() is a NO-OP.
pub fn exit_priv(cpu: &mut CPUState) {
    unsafe {
        panda_sys::exit_priv(cpu)
    }
}

/// Get count of commandline arguments
pub fn argc() -> c_int {
    unsafe {
        panda_sys::panda_argc
    }
}

/// Get commandline arguments
pub fn argv() -> Vec<String> {
    let mut rs_argv = Vec::new();

    for char_ptr in unsafe { panda_sys::panda_argv }.iter() {
        if let Ok(str_slice) = unsafe { CStr::from_ptr(*char_ptr) }.to_str() {
            rs_argv.push(str_slice.to_owned());
        }
    }

    rs_argv
}