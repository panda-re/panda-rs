use panda::prelude::*;
use panda::plugins::osi::OSI;
use std::sync::atomic::{AtomicU64, Ordering};

static NUM_BB: AtomicU64 = AtomicU64::new(0);

#[derive(PandaArgs)]
#[name = "panda_rs_example"]
struct Args {
    #[arg(default = 3)]
    num: u32,

    #[arg(required, about = "File to do a thing with")]
    file: String,
}

#[panda::on_sys_write_enter]
fn sys_write_test(cpu: &mut CPUState, pc: target_ulong, fd: target_ulong, buf: target_ulong, count: target_ulong) {
    println!(
        "sys_write buf = \"{}\"",
        String::from_utf8_lossy(&cpu.mem_read(buf, count as usize))
    );
}

#[panda::init]
fn init(_: &mut PluginHandle) {
    // let args = Args::from_panda_args();

    // println!("Test plugin init, num: {}, file: {}", args.num, args.file);
}

#[panda::before_block_exec]
fn every_basic_block(cpu: &mut CPUState, tb: &mut TranslationBlock) {
    // every 1000 basic blocks visited
    if NUM_BB.fetch_add(1, Ordering::SeqCst) % 1000 == 0 {
        println!("pc: {:X}", tb.pc);
        let proc = OSI.get_current_process(cpu);
        println!("pid: {:X}", (*proc).pid);
    }
}

#[derive(PandaArgs)]
#[name = "stringsearch"]
struct StringSearch {
    str: String
}

fn main() {
    Panda::new()
        .generic("x86_64")
        .replay("test")
        .plugin_args(&StringSearch {
            str: "test".into()
        })
        .run();
}
