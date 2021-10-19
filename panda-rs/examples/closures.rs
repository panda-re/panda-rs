use panda::prelude::*;
use panda::Callback;

fn main() {
    let mut count = 1;
    let bb_callback = Callback::new();
    bb_callback.before_block_exec(move |cpu, _| {
        println!("Block: {} | PC: {:#x?}", count, panda::regs::get_pc(cpu));
        count += 1;
        if count > 5 {
            bb_callback.disable();
        }
    });

    Panda::new().generic("x86_64").replay("test").run();
}
