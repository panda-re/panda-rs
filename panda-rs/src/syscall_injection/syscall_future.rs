use std::{
    collections::HashMap,
    future::Future,
    pin::Pin,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    task::{Context, Poll},
};

use crate::{regs, sys};

use super::syscall_regs::{SYSCALL_ARGS, SYSCALL_NUM_REG, SYSCALL_RET};
use super::{IntoSyscallArgs, SyscallArgs};
use once_cell::sync::OnceCell;
use panda_sys::{get_cpu, target_ulong, CPUState};
use parking_lot::Mutex;

pub(crate) struct SyscallFuture {
    ret_val: Arc<OnceCell<target_ulong>>,
}

// write all the syscall arguments to their corresponding registers
fn set_syscall_args(cpu: &mut CPUState, args: SyscallArgs) {
    for (reg, arg) in SYSCALL_ARGS.iter().copied().zip(args.iter_args()) {
        regs::set_reg(cpu, reg, arg);
    }
}

/// Perform a system call in the guest. Should only be run within an injector being
/// run by [`run_injector`](crate::syscall_injection::run_injector)
pub async fn syscall(num: target_ulong, args: impl IntoSyscallArgs) -> target_ulong {
    let cpu = unsafe { &mut *get_cpu() };

    // Setup the system call (set syscall num and setup argument registers)
    regs::set_reg(cpu, SYSCALL_NUM_REG, num);
    set_syscall_args(cpu, args.into_syscall_args().await);

    // Wait until the system call has returned to get the return value
    Pin::new(&mut SyscallFuture {
        ret_val: Arc::new(OnceCell::new()),
    })
    .await
}

pub(crate) static WAITING_FOR_SYSCALL: AtomicBool = AtomicBool::new(false);

// Maps ASID to RET_SLOT
lazy_static::lazy_static! {
    static ref RET_SLOT: Mutex<HashMap<target_ulong, Arc<OnceCell<target_ulong>>>>
        = Mutex::new(HashMap::new());
}

pub(crate) fn set_ret_value(cpu: &mut CPUState) {
    let asid = unsafe { sys::panda_current_asid(cpu) };
    if let Some(ret_slot) = RET_SLOT.lock().get_mut(&asid) {
        if ret_slot.set(regs::get_reg(cpu, SYSCALL_RET)).is_err() {
            println!("WARNING: Attempted to double-set syscall return value");
        }
        //.expect("Attempted to double-set syscall return value");
    }
}

fn current_asid() -> target_ulong {
    unsafe { sys::panda_current_asid(sys::get_cpu()) }
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
                RET_SLOT.lock().insert(current_asid(), ret_val);

                Poll::Pending
            }
        }
    }
}
