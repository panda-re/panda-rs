use panda::export_ppp_callback;
use panda::prelude::*;

export_ppp_callback! {
    // disables all callbacks when true is returned
    pub(crate) fn on_every_odd_block(cpu: &mut CPUState) -> bool;
    pub(crate) fn on_every_even_block(cpu: &mut CPUState);
}

fn main() {
    let mut i = 0;
    let callback = panda::Callback::new();
    callback.before_block_exec(move |cpu, _| {
        if i % 2 == 0 {
            on_every_even_block::trigger(cpu);
        } else {
            if on_every_odd_block::trigger(cpu) {
                callback.disable();
            }
        }
        i += 1;
    });

    on_every_even_block::add_callback(on_even_test);
    on_every_odd_block::add_callback(on_odd_test);

    Panda::new().generic("x86_64").replay("test").run();
}

// ===== test callbacks ======

use std::sync::atomic::{AtomicUsize, Ordering};

// using a global variable to keep track and disable after 3 odds
static TEST_COUNTER: AtomicUsize = AtomicUsize::new(0);

extern "C" fn on_odd_test(_: &mut CPUState) -> bool {
    println!("Odd!");

    TEST_COUNTER.fetch_add(1, Ordering::SeqCst) >= 3
}

extern "C" fn on_even_test(_: &mut CPUState) {
    println!("Even!");
}
