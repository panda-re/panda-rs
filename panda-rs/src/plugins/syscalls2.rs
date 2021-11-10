//! Rust bindings to syscalls2
//!
//! Not intended to be used directly, but is used internally for the callbacks in [`on_sys`]
//!
//! [`on_sys`]: crate::on_sys
//!

#[allow(unused_imports)]
use crate::sys::{target_ptr_t, target_ulong, CPUState};
use crate::{cbs::generate_syscalls_callbacks, plugin_import, regs::SyscallPc};

generate_syscalls_callbacks!();
