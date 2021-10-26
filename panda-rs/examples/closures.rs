use panda::plugins::proc_start_linux::ProcStartLinuxCallbacks;
use panda::prelude::*;
use panda::{Callback, PppCallback};

fn main() {
    // Callbacks can capture state
    let mut count = 1;
    let bb_callback = Callback::new();
    bb_callback.before_block_exec(move |cpu, _| {
        println!("Block: {} | PC: {:#x?}", count, panda::regs::get_pc(cpu));
        count += 1;
        if count > 5 {
            // callbacks can disable themselves by capturing a copy
            // of the `Callback` reference to it
            bb_callback.disable();
        }
    });

    // If you don't need to enable and disable the callback, you can just
    // use method chaining instead of assigning to a variable
    PppCallback::new().on_rec_auxv(|_, _, auxv| {
        // print out the auxillary vector when any process starts
        dbg!(auxv);
    });

    Panda::new().generic("x86_64").replay("test").run();
}
