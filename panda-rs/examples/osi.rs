use std::ffi::CStr;
use std::sync::atomic::{AtomicU64, Ordering};

use panda::plugins::osi::OSI;
use panda::prelude::*;

static NUM_BB: AtomicU64 = AtomicU64::new(0);

#[panda::init]
fn init(_: &mut PluginHandle) {
    // No specialized init needed
}

// Print every 1000 basic blocks
#[panda::after_block_exec]
fn every_basic_block(cpu: &mut CPUState, tb: &mut TranslationBlock, exit_code: u8) {
    if (u32::from(exit_code) > panda_sys::TB_EXIT_IDX1) || (panda::in_kernel_mode(cpu)) {
        return;
    }

    let curr_proc = OSI.get_current_process(cpu);
    let curr_proc_name_c_str = unsafe { CStr::from_ptr((*curr_proc).name) };

    let curr_bb = NUM_BB.fetch_add(1, Ordering::SeqCst);
    if (curr_bb % 1000 == 0) && (curr_bb != 0) {
        println!(
            "{:?} @ 0x{:016x}, {} BBs in - in shared lib? {}",
            curr_proc_name_c_str,
            tb.pc,
            NUM_BB.load(Ordering::SeqCst) - 1,
            OSI.in_shared_object(cpu, curr_proc.as_ptr()),
        );
    }
}

use panda::plugins::proc_start_linux::AuxvValues;

#[panda::on_rec_auxv]
fn on_proc_start(cpu: &mut CPUState, tb: &mut TranslationBlock, auxv: &AuxvValues) {
    dbg!(auxv);
}

fn main() {
    Panda::new().generic("x86_64").replay("test").run();
}
