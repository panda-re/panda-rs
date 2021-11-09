use crate::prelude::*;
use crate::regs::{get_reg, set_reg};
use crate::sys::get_cpu;

use super::{SYSCALL_ARGS, SYSCALL_RET};

pub(crate) struct SyscallRegs {
    ret_reg: target_ulong,
    arg_regs: [target_ulong; 6],
}

impl SyscallRegs {
    pub(crate) fn backup() -> Self {
        let cpu = unsafe { &mut *get_cpu() };

        let ret_reg = get_reg(cpu, SYSCALL_RET);
        let arg_regs = SYSCALL_ARGS.map(|reg| get_reg(cpu, reg));

        Self { ret_reg, arg_regs }
    }

    pub(crate) fn restore(self) {
        let Self { ret_reg, arg_regs } = self;
        let cpu = unsafe { &mut *get_cpu() };

        set_reg(cpu, SYSCALL_RET, ret_reg);
        for (&reg, &val) in SYSCALL_ARGS.iter().zip(arg_regs.iter()) {
            set_reg(cpu, reg, val);
        }
    }
}
