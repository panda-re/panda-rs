use panda::prelude::*;
use panda::regs::{get_reg, set_reg, Reg};
use panda::mem::{map_memory, physical_memory_write, PAGE_SIZE};

const ADDRESS: target_ulong = 0x1000;

// inc eax
// add ebx, eax
// inc ecx
const X86_CODE: &[u8] = b"\x40\x01\xC3\x41";

#[panda::after_machine_init]
fn setup(cpu: &mut CPUState) {
    // Map 2MB memory for this emulation
    map_memory("mymem", 2 * 1024 * PAGE_SIZE, ADDRESS).unwrap();

    // Write code into memory
    physical_memory_write(ADDRESS, X86_CODE);

    // Setup registers
    set_reg(cpu, Reg::RAX, 0x1);
    set_reg(cpu, Reg::RBX, 0x2);
    set_reg(cpu, Reg::RCX, 0x3);
    set_reg(cpu, Reg::RDX, 0x4);

    // Set starting PC
    set_reg(cpu, Reg::RSP, ADDRESS);
    let pc = get_reg(cpu, Reg::RSP);
    println!("PC is {:#x?}", pc);
}

#[panda::insn_translate]
fn insn_translate(_: &mut CPUState, _: target_ptr_t) -> bool {
    true
}

#[panda::insn_exec]
fn on_instruction(_: &mut CPUState, pc: target_ptr_t) {
    dbg!(pc);
}

fn main() {
    Panda::new()
        .arch(panda::Arch::x86_64)
        .args(&["-M", "configurable"])
        .run();
}
