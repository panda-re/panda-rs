use crate::sys::target_ulong;
use async_trait::async_trait;

use std::convert::TryInto;

#[cfg(doc)]
use super::syscall;

/// A trait for converting a single value into a syscall argument.
///
/// This trait is asynchronous to allow for system calls to be performed
/// during the conversion (for example to map memory in the guest).
#[async_trait]
pub trait IntoSyscallArg {
    async fn into_syscall_arg(self) -> target_ulong;
}

macro_rules! impl_for_ints {
    ($($int:ty),*) => {
        $(
            #[async_trait]
            impl IntoSyscallArg for $int {
                async fn into_syscall_arg(self) -> target_ulong {
                    self.try_into().unwrap()
                }
            }
        )*
    };
}

impl_for_ints!(u8, u16, u32, u64);

/// A trait for converting a set of values into a full set of arguments for
/// performing a system call. This trait is primarily used to provide arguments
/// to the [`syscall`] function.
///
/// This trait is asynchronous to allow for system calls to be performed
/// during the conversion (for example to map memory in the guest).
///
/// This is implemented both for arrays and tuples, up to length 6 (the max number of
/// system call arguments).
#[async_trait]
pub trait IntoSyscallArgs {
    async fn into_syscall_args(self) -> SyscallArgs;
}

/// Arguments to be passed to a system call
///
/// Should be converted to using [`IntoSyscallArgs`]. Conversion is handled generically
/// by [`syscall`].
pub struct SyscallArgs {
    regs: [target_ulong; 6],
    regs_used: usize,
}

impl SyscallArgs {
    pub fn iter_args(&self) -> impl Iterator<Item = target_ulong> + '_ {
        self.regs.iter().copied().take(self.regs_used)
    }
}

#[doc(hidden)]
pub struct SyscallCount<const N: usize>;

#[doc(hidden)]
pub trait LessThan7 {}

impl LessThan7 for SyscallCount<0> {}
impl LessThan7 for SyscallCount<1> {}
impl LessThan7 for SyscallCount<2> {}
impl LessThan7 for SyscallCount<3> {}
impl LessThan7 for SyscallCount<4> {}
impl LessThan7 for SyscallCount<5> {}
impl LessThan7 for SyscallCount<6> {}

#[async_trait]
impl<Arg: IntoSyscallArg + Send, const N: usize> IntoSyscallArgs for [Arg; N]
where
    SyscallCount<N>: LessThan7,
{
    async fn into_syscall_args(self) -> SyscallArgs {
        assert!(N <= 6, "Only up to 6 syscall arguments are allowed");
        let mut regs = [0; 6];
        for (i, arg) in IntoIterator::into_iter(self).enumerate() {
            regs[i] = arg.into_syscall_arg().await;
        }

        SyscallArgs {
            regs,
            regs_used: N as _,
        }
    }
}

macro_rules! impl_for_tuples {
    ($first:ident $(, $nth:ident)*) => {
        #[async_trait]
        impl<$first $(, $nth)*> IntoSyscallArgs for ($first, $($nth),*)
            where $first: IntoSyscallArg + Send + Sync,
                  $($nth: IntoSyscallArg + Send + Sync),*
        {
            #[allow(non_snake_case)]
            async fn into_syscall_args(self) -> SyscallArgs {
                let ($first, $($nth),*) = self;
                let arr = [
                    $first.into_syscall_arg().await,
                    $($nth.into_syscall_arg().await),*
                ];
                let mut regs = [0; 6];
                let regs_used = arr.len();

                regs[..regs_used].copy_from_slice(&arr[..]);

                SyscallArgs { regs, regs_used }
            }
        }

        impl_for_tuples!($($nth),*);
    };
    () => {
        #[async_trait]
        impl IntoSyscallArgs for () {
            async fn into_syscall_args(self) -> SyscallArgs {
                SyscallArgs { regs: [0; 6], regs_used: 0 }
            }
        }
    }
}

impl_for_tuples!(Arg1, Arg2, Arg3, Arg4, Arg5, Arg6);
