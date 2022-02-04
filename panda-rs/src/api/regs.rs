use crate::prelude::*;
use crate::{cpu_arch_state, CPUArchPtr};

use strum::IntoEnumIterator;
use strum_macros::{EnumIter, EnumString, ToString};

/// Type-safe API to allow APIs to accept only program counters coming from
/// syscall callbacks. To convert to integer of the width of your target, use the
/// `.pc()` method.
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct SyscallPc(target_ulong);

impl SyscallPc {
    pub fn pc(self) -> target_ulong {
        self.0
    }
}

// Arch-specific mappings ----------------------------------------------------------------------------------------------

// TODO: handle AX/AH/AL, etc via shifts? Tricky b/c enum val used to index QEMU array
/// x86 named guest registers
#[cfg(feature = "i386")]
#[derive(Debug, Copy, Clone, PartialEq, Eq, EnumString, EnumIter, ToString)]
pub enum Reg {
    EAX = 0,
    ECX = 1,
    EDX = 2,
    EBX = 3,
    ESP = 4,
    EBP = 5,
    ESI = 6,
    EDI = 7,
}

/// x86 return registers
#[cfg(feature = "i386")]
static RET_REGS: &'static [Reg] = &[Reg::EAX];

// TODO: handle EAX/AX/AH/AL, etc via shifts? Tricky b/c enum val used to index QEMU array
/// x64 named guest registers
#[cfg(feature = "x86_64")]
#[derive(Debug, Copy, Clone, PartialEq, Eq, EnumString, EnumIter, ToString)]
pub enum Reg {
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

/// x64 return registers
#[cfg(feature = "x86_64")]
static RET_REGS: &'static [Reg] = &[Reg::RAX];

/// ARM named guest registers
#[cfg(feature = "arm")]
#[derive(Debug, Copy, Clone, PartialEq, Eq, EnumString, EnumIter, ToString)]
pub enum Reg {
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
    LR = 14,
    SP = 13,
    IP = 15,
}

/// ARM return registers
#[cfg(feature = "arm")]
static RET_REGS: &'static [Reg] = &[Reg::R0, Reg::R1, Reg::R2, Reg::R3];

/// AArch64 named guest registers
#[cfg(feature = "aarch64")]
#[derive(Debug, Copy, Clone, PartialEq, Eq, EnumString, EnumIter, ToString)]
pub enum Reg {
    X0 = 0,
    X1 = 1,
    X2 = 2,
    X3 = 3,
    X4 = 4,
    X5 = 5,
    X6 = 6,
    X7 = 7,
    X8 = 8,
    X9 = 9,
    X10 = 10,
    X11 = 11,
    X12 = 12,
    LR = 13,
    SP = 14,
    IP = 15,
}

/// AArch64 return registers
#[cfg(feature = "aarch64")]
static RET_REGS: &'static [Reg] = &[Reg::X0, Reg::X1, Reg::X2, Reg::X3];

/// MIPS named guest registers
#[cfg(any(feature = "mips", feature = "mipsel", feature = "mips64"))]
#[derive(Debug, Copy, Clone, PartialEq, Eq, EnumString, EnumIter, ToString)]
pub enum Reg {
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

/// MIPS return registers
#[cfg(any(feature = "mips", feature = "mipsel", feature = "mips64"))]
static RET_REGS: &'static [Reg] = &[Reg::V0, Reg::V1];

// TODO: support floating point set as well? Separate QEMU bank.
/// PPC named guest registers
#[cfg(feature = "ppc")]
#[derive(Debug, Copy, Clone, PartialEq, Eq, EnumString, EnumIter, ToString)]
pub enum Reg {
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
    R13 = 13,
    R14 = 14,
    R15 = 15,
    R16 = 16,
    R17 = 17,
    R18 = 18,
    R19 = 19,
    R20 = 20,
    R21 = 21,
    R22 = 22,
    R23 = 23,
    R24 = 24,
    R25 = 25,
    R26 = 26,
    R27 = 27,
    R28 = 28,
    R29 = 29,
    R30 = 30,
    R31 = 31,
    LR = 100, // Special case - separate bank in QEMU
}

/// PPC return registers
#[cfg(feature = "ppc")]
static RET_REGS: &'static [Reg] = &[Reg::R3, Reg::R4];

// Getters/setters -----------------------------------------------------------------------------------------------------

/// Get stack pointer register
pub fn reg_sp() -> Reg {
    #[cfg(feature = "i386")]
    return Reg::ESP;

    #[cfg(feature = "x86_64")]
    return Reg::RSP;

    #[cfg(any(
        feature = "arm",
        feature = "aarch64",
        feature = "mips",
        feature = "mipsel",
        feature = "mips64"
    ))]
    return Reg::SP;

    #[cfg(any(feature = "ppc"))]
    return Reg::R1;
}

/// Get return value registers
/// MIPS/ARM/PPC: Note that most C code will only use the first register, e.g. index 0 in returned `Vec`
pub fn reg_ret_val() -> &'static [Reg] {
    return &RET_REGS;
}

/// Get return address register
pub fn reg_ret_addr() -> Option<Reg> {
    #[cfg(feature = "i386")]
    return None;

    #[cfg(feature = "x86_64")]
    return None;

    #[cfg(any(feature = "arm", feature = "aarch64", feature = "ppc"))]
    return Some(Reg::LR);

    #[cfg(any(feature = "mips", feature = "mipsel", feature = "mips64"))]
    return Some(Reg::RA);
}

/// Read the current value of a register
pub fn get_reg<T: Into<Reg>>(cpu: &CPUState, reg: T) -> target_ulong {
    let cpu_arch = cpu_arch_state!(cpu);
    let val;

    #[cfg(any(feature = "i386", feature = "x86_64", feature = "arm"))]
    unsafe {
        val = (*cpu_arch).regs[reg.into() as usize];
    }

    #[cfg(feature = "aarch64")]
    unsafe {
        val = (*cpu_arch).xregs[reg.into() as usize];
    }

    #[cfg(any(feature = "mips", feature = "mipsel", feature = "mips64"))]
    unsafe {
        val = (*cpu_arch).active_tc.gpr[reg.into() as usize];
    }

    #[cfg(any(feature = "ppc"))]
    unsafe {
        let reg_enum = reg.into();
        if reg_enum == Reg::LR {
            val = (*cpu_arch).lr;
        } else {
            val = (*cpu_arch).gpr[reg_enum as usize];
        }
    }

    val
}

/// Set the value for a register
pub fn set_reg<T: Into<Reg>>(cpu: &CPUState, reg: T, val: target_ulong) {
    let cpu_arch = cpu_arch_state!(cpu);

    #[cfg(any(feature = "i386", feature = "x86_64", feature = "arm"))]
    unsafe {
        (*cpu_arch).regs[reg.into() as usize] = val;
    }

    #[cfg(any(feature = "mips", feature = "mipsel", feature = "mips64"))]
    unsafe {
        (*cpu_arch).active_tc.gpr[reg.into() as usize] = val;
    }

    #[cfg(any(feature = "ppc"))]
    unsafe {
        let reg_enum = reg.into();
        if reg_enum == Reg::LR {
            (*cpu_arch).lr = val;
        } else {
            (*cpu_arch).gpr[reg_enum as usize] = val;
        }
    }

    #[cfg(feature = "aarch64")]
    unsafe {
        (*cpu_arch).xregs[reg.into() as usize] = val;
    }
}

pub fn get_pc(cpu: &CPUState) -> target_ulong {
    let cpu_arch = cpu_arch_state!(cpu);
    let val;

    #[cfg(any(feature = "x86_64", feature = "i386"))]
    unsafe {
        val = (*cpu_arch).eip;
    }

    #[cfg(feature = "arm")]
    unsafe {
        val = (*cpu_arch).regs[15];
    }

    #[cfg(feature = "aarch64")]
    unsafe {
        val = (*cpu_arch).pc;
    }

    #[cfg(feature = "ppc")]
    unsafe {
        val = (*cpu_arch).nip;
    }

    #[cfg(any(feature = "mips", feature = "mipsel", feature = "mips64"))]
    unsafe {
        val = (*cpu_arch).active_tc.PC;
    }

    val
}

pub fn set_pc(cpu: &mut CPUState, pc: target_ulong) {
    let cpu_arch = cpu_arch_state!(cpu);

    #[cfg(any(feature = "x86_64", feature = "i386"))]
    unsafe {
        (*cpu_arch).eip = pc;
    }

    #[cfg(feature = "arm")]
    unsafe {
        (*cpu_arch).regs[15] = pc;
    }

    #[cfg(feature = "aarch64")]
    unsafe {
        (*cpu_arch).pc = pc;
    }

    #[cfg(feature = "ppc")]
    unsafe {
        (*cpu_arch).nip = pc;
    }

    #[cfg(any(feature = "mips", feature = "mipsel", feature = "mips64"))]
    unsafe {
        (*cpu_arch).active_tc.PC = pc;
    }
}

// Printing ------------------------------------------------------------------------------------------------------------

/// Print the contents of all registers
pub fn dump_regs(cpu: &CPUState) {
    for reg in Reg::iter() {
        println!("{:?}:\t0x{:016x}", reg, get_reg(cpu, reg));
    }
}
