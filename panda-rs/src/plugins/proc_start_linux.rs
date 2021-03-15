use crate::plugin_import;

use crate::sys::{CPUState, TranslationBlock, target_ulong};

plugin_import!{
    static PROC_START_LINUX: ProcStartLinux = extern "proc_start_linux" {
        callbacks {
            fn on_rec_auxv(cpu: &mut CPUState, tb: *mut TranslationBlock, auxv: AuxvValues);
        }
    };
}

const MAX_PATH_LEN: usize = 256;

#[repr(C)]
#[derive(Debug, Clone)]
pub struct AuxvValues {
    pub procname: [u8; MAX_PATH_LEN],
    pub phdr: target_ulong,
    pub entry: target_ulong,
}
