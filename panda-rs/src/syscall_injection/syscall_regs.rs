use super::arch::*;
use crate::prelude::*;
use crate::regs::{get_reg, set_reg};
use crate::sys::get_cpu;

#[derive(Copy, Clone, Debug)]
pub struct SyscallRegs {
    sys_num_reg: target_ulong,
    arg_regs: [target_ulong; 6],
}

impl SyscallRegs {
    /// Backup all the registers needed for performing a system call
    pub fn backup() -> Self {
        let cpu = unsafe { &mut *get_cpu() };

        let sys_num_reg = get_reg(cpu, SYSCALL_NUM_REG);
        let arg_regs = SYSCALL_ARGS.map(|storage| storage.read(cpu));

        Self {
            sys_num_reg,
            arg_regs,
        }
    }

    /// Restore the registers needed for performing a system call from a backup
    pub fn restore(self) {
        let Self {
            sys_num_reg,
            arg_regs,
        } = self;
        let cpu = unsafe { &mut *get_cpu() };

        set_reg(cpu, SYSCALL_NUM_REG, sys_num_reg);
        for (&storage, &val) in SYSCALL_ARGS.iter().zip(arg_regs.iter()) {
            storage.write(cpu, val);
        }
    }
}
