use crate::{plugin_import, generate_syscalls_callbacks};
use crate::sys::{target_ptr_t, target_ulong};

generate_syscalls_callbacks!();
