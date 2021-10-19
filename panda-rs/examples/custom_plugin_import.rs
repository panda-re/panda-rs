use panda::prelude::*;

#[repr(C)]
pub struct AuxvValues {
    // TODO
}

panda::plugin_import! {
    static PROC_START_LINUX: ProcStartLinux = extern "proc_start_linux" {
        callbacks {
            fn on_rec_auxv(cpu: &mut CPUState, tb: &mut TranslationBlock, auxv: &AuxvValues);
        }
    };
}

fn main() {}
