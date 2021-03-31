use std::sync::atomic::{AtomicU64, Ordering};
use std::ffi::CStr;

use panda::prelude::*;
use panda::plugins::osi::OSI;

static NUM_BB: AtomicU64 = AtomicU64::new(0);

#[panda::init]
fn init(_: &mut PluginHandle) {
    // No specialized init needed
}

// Dump registers every 1000 basic blocks
#[panda::before_block_exec]
fn every_basic_block(cpu: &mut CPUState, tb: &mut TranslationBlock) {
    if panda::in_kernel(cpu) {
        return;
    }

    let curr_proc = OSI.get_current_process(cpu);
    let curr_proc_name_c_str = unsafe { CStr::from_ptr((*curr_proc).name) };
    let curr_bb = NUM_BB.fetch_add(1, Ordering::SeqCst);

    if  (curr_bb % 1000 == 0) && (curr_bb != 0) {
        println!("\nRegister state for process {:?} @ 0x{:016x}, {} basic blocks into execution:",
            curr_proc_name_c_str,
            tb.pc,
            NUM_BB.load(Ordering::SeqCst) - 1,
        );
        panda::regs::dump_regs(cpu);
    }
}

fn main() {
    Panda::new()
        .generic("x86_64")
        .replay("test")
        .run();
}
