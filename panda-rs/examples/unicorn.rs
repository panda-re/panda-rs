use panda::prelude::*;
use panda::regs::{get_reg, set_reg, set_pc, get_pc, Reg};
use panda::mem::{map_memory, physical_memory_write, PAGE_SIZE};

// inc rax
// add rbx, rax
// inc rcx
const X86_CODE: &[u8] = b"\x48\xFF\xC0\x48\x01\xC3\x48\xFF\xC1";

const ADDRESS: target_ulong = 0x1000;
const STOP_ADDR: target_ulong = ADDRESS + (X86_CODE.len() as target_ulong);

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
    set_pc(cpu, ADDRESS);
}

#[panda::insn_translate]
fn insn_translate(cpu: &mut CPUState, pc: target_ptr_t) -> bool {
    true
}

#[panda::insn_exec]
fn insn_exec(cpu: &mut CPUState, pc: target_ptr_t) {
    println!("pc: {:#x?}", pc);
    if pc == STOP_ADDR {
        println!("Final CPU state:");
        panda::regs::dump_regs(cpu);
        unsafe {
            // ?
            panda::sys::exit(0);
        }
    }
}

fn main() {
    Panda::new()
        .arch(panda::Arch::x86_64)
        .configurable()
        .run();
}
