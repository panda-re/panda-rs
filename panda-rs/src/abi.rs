use crate::prelude::*;
use crate::regs::Reg::{self, *};

use crate::mem::{virtual_memory_read, virtual_memory_write};
use crate::regs;

use std::convert::TryInto;
use std::sync::atomic::{AtomicBool, Ordering};

static IS_SYSENTER: AtomicBool = AtomicBool::new(false);

pub(crate) fn set_is_sysenter(is_sysenter: bool) {
    IS_SYSENTER.store(is_sysenter, Ordering::SeqCst);
}

fn is_sysenter() -> bool {
    IS_SYSENTER.load(Ordering::SeqCst)
}

pub mod syscall {
    use super::*;

    macro_rules! syscall_regs {
        {
            const { $syscall_args:ident, $syscall_ret:ident, $syscall_num_reg:ident };
            $(
                #[cfg($(arch = $arch:literal),+)] {
                    args = [$( $args:ident $(@ $offset:literal)? ),*];
                    return = $ret:ident;
                    syscall_number = $sys_num:ident;
                }
            )*
        } => {
            $(
                /// Argument registers for performing syscalls
                #[cfg(any($(feature = $arch),*))]
                pub const $syscall_args: [StorageLocation; 6] = [$(
                    StorageLocation::Reg($args) $(.with_offset($offset))?
                ),*];

                /// Register where syscall return value is stored on syscall exit
                #[cfg(any($(feature = $arch),*))]
                pub const $syscall_ret: Reg = $ret;

                /// Register where the syscall number is stored on syscall enter
                #[cfg(any($(feature = $arch),*))]
                pub const $syscall_num_reg: Reg = $sys_num;
            )*
        }
    }

    syscall_regs! {
        const {SYSCALL_ARGS, SYSCALL_RET, SYSCALL_NUM_REG};

        #[cfg(arch = "x86_64")] {
            args = [RDI, RSI, RDX, R10, R8, R9];
            return = RAX;
            syscall_number = RAX;
        }

        #[cfg(arch = "i386")] {
            args = [EBX, ECX @ 0x8, EDX @ 0x4, ESI, EDI, EBP @ 0x0];
            return = EAX;
            syscall_number = EAX;
        }

        // we primarily support EABI systems, but this might work for OABI too
        #[cfg(arch = "arm")] {
            args = [R0, R1, R2, R3, R4, R5];
            return = R0;
            syscall_number = R7;
        }

        #[cfg(arch = "aarch64")] {
            args = [X0, X1, X2, X3, X4, X5];
            return = X0;
            syscall_number = X8;
        }

        // we "only" "support" the n32 ABI (syscalls2 supports configuring o32 ABI at
        // compile-time, other things probably(?) don't)
        #[cfg(arch = "mips", arch = "mipsel", arch = "mips64")] {
            args = [A0, A1, A2, A3, T0, T1];
            return = V0;
            syscall_number = V0;
        }
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum StorageLocation {
    Reg(Reg),
    StackReg(Reg, target_ulong),
}

impl From<Reg> for StorageLocation {
    fn from(reg: Reg) -> Self {
        Self::Reg(reg)
    }
}

impl From<(Reg, target_ulong)> for StorageLocation {
    fn from((reg, offset): (Reg, target_ulong)) -> Self {
        Self::StackReg(reg, offset)
    }
}

const REG_SIZE: usize = std::mem::size_of::<target_ulong>();

impl StorageLocation {
    #[allow(dead_code)]
    pub(crate) const fn with_offset(self, offset: target_ulong) -> Self {
        let (Self::Reg(reg) | Self::StackReg(reg, _)) = self;

        Self::StackReg(reg, offset)
    }

    pub fn read(self, cpu: &mut CPUState) -> target_ulong {
        match self {
            Self::StackReg(_, stack_offset) if is_sysenter() => {
                let sp = regs::get_reg(cpu, regs::reg_sp());

                target_ulong::from_le_bytes(
                    virtual_memory_read(cpu, sp + stack_offset, REG_SIZE)
                        .expect("Failed to read syscall argument from stack")
                        .try_into()
                        .unwrap(),
                )
            }
            Self::Reg(reg) | Self::StackReg(reg, _) => regs::get_reg(cpu, reg),
        }
    }

    pub fn write(self, cpu: &mut CPUState, val: target_ulong) {
        match self {
            Self::StackReg(reg, stack_offset) if is_sysenter() => {
                let sp = regs::get_reg(cpu, regs::reg_sp());

                virtual_memory_write(cpu, sp + stack_offset, &val.to_le_bytes());

                #[cfg(feature = "i386")]
                if reg == Reg::EBP {
                    return;
                }

                regs::set_reg(cpu, reg, val);
            }
            Self::Reg(reg) | Self::StackReg(reg, _) => regs::set_reg(cpu, reg, val),
        }
    }
}
