use panda::plugins::hooks::Hook;
use panda::plugins::proc_start_linux::AuxvValues;
use panda::prelude::*;

#[panda::init]
fn init(_: &mut PluginHandle) {
    // No specialized init needed
}

#[panda::hook]
fn entry_hook(_cpu: &mut CPUState, _tb: &mut TranslationBlock, _exit_code: u8, hook: &mut Hook) {
    println!("\n\nHit entry hook!\n");
    hook.enabled = false;
}

#[panda::on_rec_auxv]
fn on_proc_start(_cpu: &mut CPUState, _tb: &mut TranslationBlock, auxv: &AuxvValues) {
    let address = auxv.entry;
    panda::hook::before_block_exec(move |_, _, hook| {
        println!(
            "Before block exec of closure entry hook. (at address: {:#x?})",
            address
        );

        hook.enabled = false;
    })
    .at_addr(auxv.entry);

    entry_hook::hook().after_block_exec().at_addr(auxv.entry)
}

fn main() {
    Panda::new().generic("x86_64").replay("test").run();
}
