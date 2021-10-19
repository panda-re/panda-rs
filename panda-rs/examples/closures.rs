use panda::prelude::*;
use panda::Callback;

fn main() {
    let mut x = 1;
    let bb_callback = Callback::new();
    bb_callback.before_block_exec(move |cpu, _| {
        println!("Block: {}", x);
        x += 1;
        if x > 5 {
            bb_callback.disable();
        }
    });

    Panda::new().generic("x86_64").replay("test").run();
}
