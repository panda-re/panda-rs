use crate::regs::Reg::{self, *};

#[cfg(feature = "x86_64")]
pub const SYSCALL_ARGS: [Reg; 6] = [RDI, RSI, RDX, RCX, R8, R9];

#[cfg(feature = "x86_64")]
pub const SYSCALL_RET: Reg = RAX;

#[cfg(feature = "x86_64")]
pub const SYSCALL_NUM_REG: Reg = RAX;
