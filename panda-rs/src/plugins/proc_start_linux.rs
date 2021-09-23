//! Raw Rust bindings for proc_start_linux plugin
//!
//! Not designed to be used directly, but is used internally for:
//!
//! * [`on_rec_auxv`](crate::on_rec_auxv)
use crate::plugin_import;
use std::os::raw::{c_char, c_int};

use crate::sys::{target_ulong, CPUState, TranslationBlock};

plugin_import! {
    static PROC_START_LINUX: ProcStartLinux = extern "proc_start_linux" {
        callbacks {
            fn on_rec_auxv(cpu: &mut CPUState, tb: &mut TranslationBlock, auxv: &AuxvValues);
        }
    };
}

pub const MAX_PATH_LEN: u32 = 256;
pub const MAX_NUM_ARGS: u32 = 10;
pub const MAX_NUM_ENV: u32 = 20;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct AuxvValues {
    pub argc: c_int,
    pub argv_ptr_ptr: target_ulong,
    pub arg_ptr: [target_ulong; 10usize],
    pub argv: [[c_char; 256usize]; 10usize],
    pub envc: c_int,
    pub env_ptr_ptr: target_ulong,
    pub env_ptr: [target_ulong; 20usize],
    pub envp: [[c_char; 256usize]; 20usize],
    pub execfn_ptr: target_ulong,
    pub execfn: [c_char; 256usize],
    pub phdr: target_ulong,
    pub entry: target_ulong,
    pub ehdr: target_ulong,
    pub hwcap: target_ulong,
    pub hwcap2: target_ulong,
    pub pagesz: target_ulong,
    pub clktck: target_ulong,
    pub phent: target_ulong,
    pub phnum: target_ulong,
    pub base: target_ulong,
    pub flags: target_ulong,
    pub uid: target_ulong,
    pub euid: target_ulong,
    pub gid: target_ulong,
    pub egid: target_ulong,
    pub secure: bool,
    pub random: target_ulong,
    pub platform: target_ulong,
    pub program_header: target_ulong,
}
