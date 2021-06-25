use panda::prelude::*;
use panda::regs::{set_reg, set_pc, Reg};
use panda::mem::{map_memory, physical_memory_write, PAGE_SIZE};
use panda::taint;

// inc rax
// add rbx, rax
// inc rcx
const X86_CODE: &[u8] = b"\x48\xFF\xC0\x48\x01\xC3\x48\xFF\xC1";

const ADDRESS: target_ulong = 0x1000;
const STOP_ADDR: target_ulong = ADDRESS + (X86_CODE.len() as target_ulong);

// configure our state before running
#[panda::after_machine_init] // <--- runs immediately after the QEMU machine is accessible
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

    // Iterate over registers to show initial taint
    for reg in [Reg::RAX, Reg::RBX, Reg::RCX, Reg::RDX] {
        println!("{:?} is tained? {:?}", reg, taint::check_reg(reg));
    }

    println!("Tainting RAX with label '1'...");
    taint::label_reg(Reg::RAX, 1);

    println!("Tainting RBX with label '2'...");
    taint::label_reg(Reg::RBX, 2);

    // Set starting PC
    set_pc(cpu, ADDRESS);
}

// In order to ensure our `insn_exec` callback runs every instruction, we must instruct PANDA to
// translate and instrument (i.e. add code to call our callback) each instruction.
#[panda::insn_translate] // <--- runs every instruction
fn insn_translate(_cpu: &mut CPUState, _pc: target_ptr_t) -> bool {
    true
}

#[panda::insn_exec] // <--- runs every instrumented instruction
fn insn_exec(cpu: &mut CPUState, pc: target_ptr_t) {
    println!("pc: {:#x?}", pc);

    // if we've reached the end of our shellcode, dump the registers and taint to stdout, then quit
    if pc == STOP_ADDR {
        println!("Final CPU state:");
        panda::regs::dump_regs(cpu);

        for reg in [Reg::RAX, Reg::RBX, Reg::RCX, Reg::RDX] {
            println!("{:?} is tained? {:?}", reg, taint::check_reg(reg));

            if taint::check_reg(reg) {
                println!("(Tainted by {:?})", taint::get_reg(reg));
            }
        }

        unsafe {
            panda::sys::exit(0);
        }
    }
}

// When the example runs, start up a new PANDA instance. It's marked `.configurable()` so that
// instead of running a built-in QEMU system, we can just build our own baremetal execution
// environment
fn main() {
    Panda::new()
        .arch(panda::Arch::x86_64)
        .configurable()
        .run();
}
