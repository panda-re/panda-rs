//! Raw Rust bindings for proc_start_linux plugin
//!
//! Not designed to be used directly, but is used internally for:
//!
//! * [`on_rec_auxv`](crate::on_rec_auxv)
use crate::plugin_import;
use std::os::raw::c_int;

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

/// A struct representing the contents of the Auxilary Vector, the
/// information provided by the kernel when starting up a new process.
///
/// Resources on the auxilary vector:
/// * <https://lwn.net/Articles/519085/>
/// * <http://articles.manugarg.com/aboutelfauxiliaryvectors.html>
#[repr(C)]
#[derive(Clone)]
pub struct AuxvValues {
    pub argc: c_int,
    pub argv_ptr_ptr: target_ulong,
    pub arg_ptr: [target_ulong; 10usize],
    pub argv: [[u8; 256usize]; 10usize],
    pub envc: c_int,
    pub env_ptr_ptr: target_ulong,
    pub env_ptr: [target_ulong; 20usize],
    pub envp: [[u8; 256usize]; 20usize],
    pub execfn_ptr: target_ulong,
    pub execfn: [u8; 256usize],
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

impl AuxvValues {
    pub fn argv(&self) -> Vec<String> {
        self.argv[..self.argc as usize]
            .iter()
            .map(|arg| {
                String::from_utf8_lossy(&arg[..arg.iter().position(|x| *x == 0).unwrap()])
                    .into_owned()
            })
            .collect()
    }

    pub fn envp(&self) -> Vec<String> {
        self.envp[..self.envc as usize]
            .iter()
            .map(|env| {
                String::from_utf8_lossy(&env[..env.iter().position(|x| *x == 0).unwrap()])
                    .into_owned()
            })
            .collect()
    }

    pub fn execfn(&self) -> String {
        let execfn = &self.execfn;
        String::from_utf8_lossy(&execfn[..execfn.iter().position(|x| *x == 0).unwrap()])
            .into_owned()
    }
}

use std::fmt;

struct HexDebug<D: fmt::Debug>(D);

impl<D: fmt::Debug> fmt::Debug for HexDebug<D> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:#x?}", self.0)
    }
}

impl fmt::Debug for AuxvValues {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AuxvValues")
            .field("argc", &self.argc)
            .field("argv_ptr_ptr", &HexDebug(self.argv_ptr_ptr))
            .field("arg_ptr", &HexDebug(&self.arg_ptr[..self.argc as usize]))
            .field("argv", &self.argv())
            .field("envc", &self.envc)
            .field("env_ptr_ptr", &HexDebug(self.env_ptr_ptr))
            .field("env_ptr", &HexDebug(&self.env_ptr[..self.envc as usize]))
            .field("envp", &self.envp())
            .field("execfn_ptr", &HexDebug(self.execfn_ptr))
            .field("execfn", &self.execfn())
            .field("phdr", &HexDebug(self.phdr))
            .field("entry", &HexDebug(self.entry))
            .field("ehdr", &HexDebug(self.ehdr))
            .field("hwcap", &self.hwcap)
            .field("hwcap2", &self.hwcap2)
            .field("pagesz", &HexDebug(self.pagesz))
            .field("clktck", &self.clktck)
            .field("phent", &self.phent)
            .field("phnum", &self.phnum)
            .field("base", &HexDebug(self.base))
            .field("flags", &self.flags)
            .field("uid", &self.uid)
            .field("euid", &self.euid)
            .field("gid", &self.gid)
            .field("egid", &self.egid)
            .field("secure", &self.secure)
            .field("random", &HexDebug(self.random))
            .field("platform", &HexDebug(self.platform))
            .field("program_header", &HexDebug(self.program_header))
            .finish()
    }
}
