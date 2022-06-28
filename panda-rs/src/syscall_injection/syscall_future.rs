use std::{
    future::Future,
    pin::Pin,
    sync::{
        atomic::{AtomicBool, AtomicU64, Ordering},
        Arc,
    },
    task::{Context, Poll},
};

use super::arch::{SYSCALL_ARGS, SYSCALL_NUM_REG, SYSCALL_RET};
use super::{IntoSyscallArgs, SyscallArgs, ThreadId};
use crate::regs;

use dashmap::DashMap;
use lazy_static::lazy_static;
use once_cell::sync::OnceCell;
use panda_sys::{get_cpu, target_ulong, CPUState};

pub(crate) struct SyscallFuture {
    ret_val: Arc<OnceCell<target_ulong>>,
}

// write all the syscall arguments to their corresponding registers
fn set_syscall_args(cpu: &mut CPUState, args: SyscallArgs) {
    for (storage_location, arg) in SYSCALL_ARGS.iter().copied().zip(args.iter_args()) {
        storage_location.write(cpu, arg);
    }
}

lazy_static! {
    static ref LAST_INJECTED_SYSCALL: DashMap<ThreadId, AtomicU64> = DashMap::new();
}

pub(crate) fn last_injected_syscall() -> target_ulong {
    LAST_INJECTED_SYSCALL
        .get(&ThreadId::current())
        .map(|num| num.load(Ordering::SeqCst) as target_ulong)
        .unwrap_or_else(|| {
            log::warn!("No syscall num found for thread {:?}", ThreadId::current());
            0xBADCA11
        })
}

fn set_syscall_num(cpu: &mut CPUState, num: target_ulong) {
    LAST_INJECTED_SYSCALL
        .entry(ThreadId::current())
        .or_default()
        .store(num as u64, Ordering::SeqCst);
    regs::set_reg(cpu, SYSCALL_NUM_REG, num);
}

/// Perform a system call in the guest. Should only be run within an injector being
/// run by [`run_injector`](crate::syscall_injection::run_injector)
pub async fn syscall(num: target_ulong, args: impl IntoSyscallArgs) -> target_ulong {
    log::trace!("Injecting syscall {}", num);
    let cpu = unsafe { &mut *get_cpu() };

    let saved_sp = regs::get_reg(cpu, regs::reg_sp());

    #[cfg(feature = "i386")]
    let saved_bp = regs::get_reg(cpu, regs::Reg::EBP);

    // Setup the system call
    set_syscall_num(cpu, num);
    set_syscall_args(cpu, args.into_syscall_args().await);

    // Wait until the system call has returned to get the return value
    let ret = Pin::new(&mut SyscallFuture {
        ret_val: Arc::new(OnceCell::new()),
    })
    .await;

    log::trace!("Injected syscall {} returned {}", num, ret);

    regs::set_reg(cpu, regs::reg_sp(), saved_sp);

    #[cfg(feature = "i386")]
    regs::set_reg(cpu, regs::Reg::EBP, saved_bp);

    ret
}

/// Perform a system call in the guest. Should only be run within an injector being
/// run by [`run_injector`](crate::syscall_injection::run_injector). Registers will
/// not be restored after this syscall has been ran.
pub async fn syscall_no_return(num: target_ulong, args: impl IntoSyscallArgs) -> ! {
    log::trace!("syscall_no_return num={}", num);
    let cpu = unsafe { &mut *get_cpu() };

    // Setup the system call
    set_syscall_num(cpu, num);
    set_syscall_args(cpu, args.into_syscall_args().await);

    bail_no_restore_regs().await
}

/// Bail from the current injector without restoring the original registers
pub async fn bail_no_restore_regs() -> ! {
    log::trace!("Bailing without restoring syscall args");
    INJECTOR_BAIL.store(true, Ordering::SeqCst);

    std::future::pending().await
}

pub(crate) static INJECTOR_BAIL: AtomicBool = AtomicBool::new(false);
pub(crate) static WAITING_FOR_SYSCALL: AtomicBool = AtomicBool::new(false);

// Maps ASID to RET_SLOT
lazy_static! {
    static ref RET_SLOT: DashMap<ThreadId, Arc<OnceCell<target_ulong>>> = DashMap::new();
}

pub(crate) fn set_ret_value(cpu: &mut CPUState) {
    if let Some(ret_slot) = RET_SLOT.get(&ThreadId::current()) {
        if ret_slot.set(regs::get_reg(cpu, SYSCALL_RET)).is_err() {
            println!("WARNING: Attempted to double-set syscall return value");
        }

        log::trace!(
            "Return value set to {:#x?}",
            regs::get_reg(cpu, SYSCALL_RET)
        );
    }
}

impl Future for SyscallFuture {
    type Output = target_ulong;

    fn poll(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Self::Output> {
        match self.ret_val.get() {
            // if the return value of the syscall has already been set, then this
            // future can return
            Some(ret_val) => Poll::Ready(*ret_val),

            // if the return value hasn't been set, set this future as the next
            // return value to set
            None => {
                let ret_val = Arc::clone(&self.ret_val);

                WAITING_FOR_SYSCALL.store(true, Ordering::SeqCst);
                RET_SLOT.insert(ThreadId::current(), ret_val);

                Poll::Pending
            }
        }
    }
}
