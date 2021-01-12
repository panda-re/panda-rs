use crate::prelude::*;
use crate::cpu_arch_state;

use strum_macros::{EnumString, EnumIter};
use strum::IntoEnumIterator;

// Arch-specific mappings ----------------------------------------------------------------------------------------------

// TODO: handle AX/AH/AL, etc via shifts?
#[cfg(feature = "i386")]
#[derive(Debug, Copy, Clone, PartialEq, Eq, EnumString, EnumIter)]
enum Reg {
    EAX = 0,
    ECX = 1,
    EDX = 2,
    EBX = 3,
    ESP = 4,
    EBP = 5,
    ESI = 6,
    EDI = 7,
}

// TODO: handle EAX/AX/AH/AL, etc via shifts?
#[cfg(feature = "x86_64")]
#[derive(Debug, Copy, Clone, PartialEq, Eq, EnumString, EnumIter)]
enum Reg {
    RAX = 0,
    RCX = 1,
    RDX = 2,
    RBX = 3,
    RSP = 4,
    RBP = 5,
    RSI = 6,
    RDI = 7,
    R8 = 8,
    R9 = 9,
    R10 = 10,
    R11 = 11,
    R12 = 12,
    R13 = 13,
    R14 = 14,
    R15 = 15,
}

#[cfg(feature = "arm")]
#[derive(Debug, Copy, Clone, PartialEq, Eq, EnumString, EnumIter)]
enum Reg {
    R0 = 0,
    R1 = 1,
    R2 = 2,
    R3 = 3,
    R4 = 4,
    R5 = 5,
    R6 = 6,
    R7 = 7,
    R8 = 8,
    R9 = 9,
    R10 = 10,
    R11 = 11,
    R12 = 12,
    LR = 13,
    SP = 14,
    IP = 15,
}

#[cfg(any(feature = "mips", feature = "mipsel"))]
#[derive(Debug, Copy, Clone, PartialEq, Eq, EnumString, EnumIter)]
enum Reg {
    ZERO = 0,
    AT = 1,
    V0 = 2,
    V1 = 3,
    A0 = 4,
    A1 = 5,
    A2 = 6,
    A3 = 7,
    T0 = 8,
    T1 = 9,
    T2 = 10,
    T3 = 11,
    T4 = 12,
    T5 = 13,
    T6 = 14,
    T7 = 15,
    S0 = 16,
    S1 = 17,
    S2 = 18,
    S3 = 19,
    S4 = 20,
    S5 = 21,
    S6 = 22,
    S7 = 23,
    T8 = 24,
    T9 = 25,
    K0 = 26,
    K1 = 27,
    GP = 28,
    SP = 29,
    FP = 30,
    RA = 31,
}

// TODO: reg map
//#[cfg(feature = "aarch64")]
//#[derive(Debug, PartialEq, Eq, EnumString, EnumIter)]

// TODO: reg map
//#[cfg(feature = "ppc")]
//#[derive(Debug, PartialEq, Eq, EnumString, EnumIter)]

// Getter/setter -------------------------------------------------------------------------------------------------------

/// Get stack pointer register
fn reg_sp() -> Reg {

    #[cfg(feature = "i386")]
    return Reg::ESP;

    #[cfg(feature = "x86_64")]
    return Reg::RSP;

    #[cfg(feature = "arm")]
    return Reg::SP;

    #[cfg(any(feature = "mips", feature = "mipsel"))]
    return Reg::SP;
}

/// Get return value register
/// MIPS note: returns `v0`, but `v1` may additionally be used in some cases.
fn reg_ret_val() -> Reg {

    #[cfg(feature = "i386")]
    return Reg::EAX;

    #[cfg(feature = "x86_64")]
    return Reg::RAX;

    #[cfg(feature = "arm")]
    return Reg::SP;

    #[cfg(any(feature = "mips", feature = "mipsel"))]
    return Reg::V0;
}

/// Get return address register
fn reg_ret_addr() -> Option<Reg> {

    #[cfg(feature = "i386")]
    return None;

    #[cfg(feature = "x86_64")]
    return None;

    #[cfg(feature = "arm")]
    return Some(Reg::LR);

    #[cfg(any(feature = "mips", feature = "mipsel"))]
    return Some(Reg::RA);
}

/// Read the current value of a register
#[cfg(any(feature = "i386", feature = "x86_64", feature = "arm"))]
fn get_reg(cpu: &CPUState, reg: Reg) -> target_ulong {
    let cpu = cpu_arch_state!(cpu);
    let val;
    unsafe {
        val = (*cpu).regs[reg as usize];
    }
    val
}

/// Read the current value of a register
#[cfg(any(feature = "mips", feature = "mipsel"))]
fn get_reg(cpu: &CPUState, reg: Reg) -> target_ulong {
    let cpu = cpu_arch_state!(cpu);
    let val;
    unsafe {
        val = (*cpu).active_tc.gpr[reg as usize];
    }
    val
}

/// Set the value for a register
#[cfg(any(feature = "i386", feature = "x86_64", feature = "arm"))]
fn set_reg(cpu: &CPUState, reg: Reg, val: target_ulong) {
    let cpu = cpu_arch_state!(cpu);
    unsafe {
        (*cpu).regs[reg as usize] = val;
    }
}

/// Set the value for a register
#[cfg(any(feature = "mips", feature = "mipsel"))]
fn set_reg(cpu: &CPUState, reg: Reg, val: target_ulong) {
    let cpu = cpu_arch_state!(cpu);
    unsafe {
        (*cpu).active_tc.gpr[reg as usize] = val;
    }
}

// Printing ------------------------------------------------------------------------------------------------------------

/// Print the contents of all registers
fn dump_regs(cpu: &CPUState) {
    for reg in Reg::iter() {
        println!("{:?}: 0x{:016x}", reg, get_reg(cpu, reg));
    }
}