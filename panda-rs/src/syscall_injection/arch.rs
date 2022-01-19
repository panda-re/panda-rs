use crate::prelude::*;
use crate::regs::Reg::{self, *};
use crate::syscall_injection::StorageLocation;

#[cfg(feature = "x86_64")]
pub(crate) const FORK: target_ulong = 57;

#[cfg(feature = "i386")]
pub(crate) const FORK: target_ulong = 2;

#[cfg(feature = "arm")]
pub(crate) const FORK: target_ulong = 2;

#[cfg(feature = "aarch64")]
pub(crate) const FORK: target_ulong = 220;

#[cfg(feature = "mips64")]
pub(crate) const FORK: target_ulong = 5056;

#[cfg(any(feature = "mips", feature = "mipsel"))]
pub(crate) const FORK: target_ulong = 6056; // n32

pub(crate) const FORK_IS_CLONE: bool = cfg!(feature = "aarch64");

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
            #[cfg(any($(feature = $arch),*))]
            pub(crate) const $syscall_args: [StorageLocation; 6] = [$(
                StorageLocation::Reg($args) $(.with_offset($offset))?
            ),*];

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
