//! Rust bindings to syscalls2
//!
//! Not intended to be used directly, but is used internally for:
//!
//! * [`on_sys_write_enter`](crate::on_sys_write_enter)
//! * [`on_sys_execve_enter`](crate::on_sys_execve_enter)
//!

#[allow(unused_imports)]
use crate::sys::{target_ptr_t, target_ulong, CPUState};
use crate::{plugin_import, generate_syscalls_callbacks};

generate_syscalls_callbacks!();
