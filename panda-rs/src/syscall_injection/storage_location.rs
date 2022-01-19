use crate::mem::{virtual_memory_read, virtual_memory_write};
use crate::prelude::*;
use crate::regs::{self, Reg};

use std::convert::TryInto;
use std::sync::atomic::{AtomicBool, Ordering};

static IS_SYSENTER: AtomicBool = AtomicBool::new(false);

pub(crate) fn set_is_sysenter(is_sysenter: bool) {
    IS_SYSENTER.store(is_sysenter, Ordering::SeqCst);
}

#[derive(Clone, Copy)]
pub(crate) enum StorageLocation {
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

fn is_sysenter() -> bool {
    IS_SYSENTER.load(Ordering::SeqCst)
}

impl StorageLocation {
    pub(crate) fn read(self, cpu: &mut CPUState) -> target_ulong {
        match self {
            Self::StackReg(_, stack_offset) if is_sysenter() => target_ulong::from_le_bytes(
                virtual_memory_read(
                    cpu,
                    regs::get_reg(cpu, regs::reg_sp()) + stack_offset,
                    std::mem::size_of::<target_ulong>(),
                )
                .expect("Failed to read syscall argument from stack")
                .try_into()
                .unwrap(),
            ),
            Self::Reg(reg) | Self::StackReg(reg, _) => regs::get_reg(cpu, reg),
        }
    }

    pub(crate) fn write(self, cpu: &mut CPUState, val: target_ulong) {
        match self {
            Self::StackReg(_, stack_offset) if is_sysenter() => {
                virtual_memory_write(
                    cpu,
                    regs::get_reg(cpu, regs::reg_sp()) + stack_offset,
                    &val.to_le_bytes(),
                );
            }
            Self::Reg(reg) | Self::StackReg(reg, _) => regs::set_reg(cpu, reg, val),
        }
    }

    #[allow(dead_code)]
    pub(crate) const fn with_offset(self, offset: target_ulong) -> Self {
        let (Self::Reg(reg) | Self::StackReg(reg, _)) = self;

        Self::StackReg(reg, offset)
    }
}
