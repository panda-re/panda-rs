use crate::prelude::*;

/// Determine if guest is currently in kernelspace
pub fn in_kernel(cpu: &mut CPUState) -> bool {
    unsafe {
        panda_sys::panda_in_kernel_external(cpu)
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

/* TODO: expose in libpanda!
/// Get current guest userspace stack pointer
pub fn current_sp(cpu: &mut CPUState) -> target_ulong {
    unsafe {
        panda_sys::panda_current_sp(cpu)
    }
}
*/

/* TODO: expose in libpanda!
/// Get current guest kernelspace stack pointer
pub fn current_ksp(cpu: &mut CPUState) -> target_ulong {
    unsafe {
        panda_sys::panda_current_ksp(cpu)
    }
}
*/

/* TODO: expose in libpanda!
/// Get current guest function return value
pub fn get_ret_val(cpu: &mut CPUState) -> target_ulong {
    unsafe {
        panda_sys::panda_ret_val(cpu)
    }
}
*/