use crate::prelude::*;
use crate::regs::{
    get_reg, set_reg,
    Reg::{self, *},
};
use crate::sys::get_cpu;

pub(crate) struct SyscallRegs {
    sys_num_reg: target_ulong,
    arg_regs: [target_ulong; 6],
}

impl SyscallRegs {
    /// Backup all the registers needed for performing a system call
    pub(crate) fn backup() -> Self {
        let cpu = unsafe { &mut *get_cpu() };

        let sys_num_reg = get_reg(cpu, SYSCALL_NUM_REG);
        let arg_regs = SYSCALL_ARGS.map(|reg| get_reg(cpu, reg));

        Self {
            sys_num_reg,
            arg_regs,
        }
    }

    /// Restore the registers needed for performing a system call from a backup
    pub(crate) fn restore(self) {
        let Self {
            sys_num_reg,
            arg_regs,
        } = self;
        let cpu = unsafe { &mut *get_cpu() };

        set_reg(cpu, SYSCALL_NUM_REG, sys_num_reg);
        for (&reg, &val) in SYSCALL_ARGS.iter().zip(arg_regs.iter()) {
            set_reg(cpu, reg, val);
        }
    }
}

macro_rules! syscall_regs {
    {
        const { $syscall_args:ident, $syscall_ret:ident, $syscall_num_reg:ident };
        $(
            #[cfg($(arch = $arch:literal),+)] {
                args = [$( $args:ident ),*];
                return = $ret:ident;
                syscall_number = $sys_num:ident;
            }
        )*
    } => {
        $(
            #[cfg(any($(feature = $arch),*))]
            pub(crate) const $syscall_args: [Reg; 6] = [$($args),*];

            #[cfg(any($(feature = $arch),*))]
            pub(crate) const $syscall_ret: Reg = $ret;

            #[cfg(any($(feature = $arch),*))]
            pub(crate) const $syscall_num_reg: Reg = $sys_num;
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
        args = [EBX, ECX, EDX, ESI, EDI, EBP];
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
