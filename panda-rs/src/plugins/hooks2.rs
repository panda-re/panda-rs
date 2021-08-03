//! Raw Rust bindings for hooks2 plugin
//!
//! Not designed to be used directly, but is used internally for:
//!
//! * [`on_process_start`](crate::on_process_start)
//! * [`on_process_end`](crate::on_process_end)
//! * [`on_thread_start`](crate::on_thread_start)
//! * [`on_thread_end`](crate::on_thread_end)
//! * [`on_mmap_updated`](crate::on_mmap_updated)
use std::os::raw::c_char;
use crate::plugin_import;
use crate::sys::{CPUState, target_pid_t, target_ulong};

panda_macros::generate_hooks2_callbacks!();
