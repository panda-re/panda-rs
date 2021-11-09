use std::{
    future::Future,
    sync::atomic::Ordering,
    task::{Context, Poll, RawWaker, RawWakerVTable, Waker},
};

use crate::prelude::*;
use crate::{plugins::syscalls2::Syscalls2Callbacks, regs, PppCallback};

mod conversion;
mod pinned_queue;
mod syscall_future;
mod syscall_regs;
mod syscalls;

use pinned_queue::PinnedQueue;
use syscall_future::WAITING_FOR_SYSCALL;
use syscall_regs::SyscallRegs;
pub use {conversion::*, syscall_future::*, syscalls::*};

type Injector = dyn Future<Output = ()>;

static INJECTORS: PinnedQueue<Injector> = PinnedQueue::new();

/// Queue a syscall injector in the form as an async block/value to be evaluated
pub fn queue_injector(pc: target_ptr_t, injector: impl Future<Output = ()> + 'static) {
    let is_first = INJECTORS.is_empty();
    INJECTORS.push_future(async {
        let backed_up_regs = SyscallRegs::backup();

        injector.await;

        backed_up_regs.restore();
    });

    // Only install each callback once
    if is_first {
        let sys_enter = PppCallback::new();
        let sys_return = PppCallback::new();

        // after the syscall set the return value for the future then jump back to
        // the syscall instruction
        sys_return.on_all_sys_return(move |cpu, _, _| {
            set_ret_value(cpu);
            regs::set_pc(cpu, pc);
            unsafe {
                panda::sys::cpu_loop_exit_noexc(cpu);
            }
        });

        // poll the injectors and if they've all finished running, disable these
        // callbacks
        sys_enter.on_all_sys_enter(move |_, _, _| {
            if poll_injectors() {
                sys_enter.disable();
                sys_return.disable();
            }
        });

        // If this is the first syscall it needs to be polled too,
        // disabling if it's already finished running
        if poll_injectors() {
            println!("WARN: Injector seemed to not call any system calls?");
            sys_enter.disable();
            sys_return.disable();
        }
    }
}

fn do_nothing(_ptr: *const ()) {}

fn clone(ptr: *const ()) -> RawWaker {
    RawWaker::new(ptr, &VTABLE)
}

static VTABLE: RawWakerVTable = RawWakerVTable::new(clone, do_nothing, do_nothing, do_nothing);

fn waiting_for_syscall() -> bool {
    WAITING_FOR_SYSCALL.load(Ordering::SeqCst)
}

/// Returns true if all injectors have been processed
fn poll_injectors() -> bool {
    let raw = RawWaker::new(std::ptr::null(), &VTABLE);
    let waker = unsafe { Waker::from_raw(raw) };
    let mut ctxt = Context::from_waker(&waker);

    // reset the 'waiting for system call' flag
    WAITING_FOR_SYSCALL.store(false, Ordering::SeqCst);

    while let Some(mut current_injector) = INJECTORS.current() {
        match current_injector.as_mut().poll(&mut ctxt) {
            // If the current injector has finished running start polling the next
            // injector.
            Poll::Ready(_) => {
                drop(current_injector);
                INJECTORS.pop();
                continue;
            }

            // If the future is now waiting on a syscall to be evaluated, return
            // so a system call can be run
            Poll::Pending if waiting_for_syscall() => return false,

            // If the future is not waiting on a system call we should keep polling
            Poll::Pending => continue,
        }
    }

    true
}
