use panda::prelude::*;
use panda::regs::{get_reg, set_reg, set_pc, get_pc, Reg};
use panda::mem::{map_memory, physical_memory_write, PAGE_SIZE};

// ADD X1, X1, 1
// ADD X0, X1, X2
// ADD X2, X2, 1
const AARCH64_CODE: &[u8] = b"\x21\x04\x00\x91\x20\x00\x02\x8b\x42\x04\x00\x91";

const ADDRESS: target_ulong = 0x1000;
const STOP_ADDR: target_ulong = ADDRESS + (AARCH64_CODE.len() as target_ulong);

#[panda::after_machine_init]
fn setup(cpu: &mut CPUState) {
    // Map 2MB memory for this emulation
    map_memory("mymem", 2 * 1024 * PAGE_SIZE, ADDRESS).unwrap();

    // Write code into memory
    physical_memory_write(ADDRESS, AARCH64_CODE);

    // Setup registers
    set_reg(cpu, Reg::X0, 0x1);
    set_reg(cpu, Reg::X1, 0x2);
    set_reg(cpu, Reg::X2, 0x3);
    set_reg(cpu, Reg::X3, 0x4);

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
        .arch(panda::Arch::AArch64)
        .configurable()
        .run();
}
