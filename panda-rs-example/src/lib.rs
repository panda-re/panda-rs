use panda::{PluginHandle, sys::{CPUState, TranslationBlock}};
use std::sync::atomic::{AtomicU64, Ordering};
use core::ptr;

static NUM_BB: AtomicU64 = AtomicU64::new(0);

#[panda::init]
fn init(plugin: &mut PluginHandle) {
    println!("Test plugin init");
}

#[panda::before_block_exec]
fn every_basic_block(cpu: &mut CPUState, tb: &mut TranslationBlock) {
    // every 1000 basic blocks visited
    if NUM_BB.fetch_add(1, Ordering::SeqCst) % 1000 == 0 {
        println!("pc: {:X}", tb.pc);
    }

    //let x: u32 = cpu.mem_read_val(tb.pc);
}

use panda::Panda;

fn main() {
    Panda::new()
        .generic("x86_64")
        .run();
}
