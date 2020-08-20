use std::os::raw::c_char;
use crate::plugin_import;
use crate::sys::{CPUState, target_pid_t, target_ulong};

crate::generate_hooks2_callbacks!();
