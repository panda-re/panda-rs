//! Everything to perform async system call injection to perform system calls
//! within the guest.
//!
//! This feature allows for writing code using Rust's async model in such a manner
//! that allows you to treat guest system calls as I/O to be performed. This enables
//! writing code that feels synchronous while allowing for automatically running the
//! guest concurrently in order to perform any needed tasks such as filesystem access,
//! interacting with processes/signals, mapping memory, etc. all within the guest,
//! while all computation is performed on the host.
//!
//! A system call injector under this API is an async block which can make use of the
//! [`syscall`] function in order to perform system calls. An injector can only be run
//! (or, rather, started) within a syscall enter callback.
//!
//! ## Example
//!
//! ```
//! use panda::prelude::*;
//! use panda::syscall_injection::{run_injector, syscall};
//!
//! async fn getpid() -> target_ulong {
//!     syscall(GET_PID, ()).await
//! }
//!
//! async fn getuid() -> target_ulong {
//!     syscall(GET_UID, ()).await
//! }
//!
//! #[panda::on_all_sys_enter]
//! fn any_syscall(cpu: &mut CPUState, pc: SyscallPc, syscall_num: target_ulong) {
//!     run_injector(pc, async {
//!         println!("PID: {}", getpid().await);
//!         println!("UID: {}", getuid().await);
//!         println!("PID (again): {}", getpid().await);
//!     });
//! }
//!
//! fn main() {
//!     Panda::new()
//!         .generic("x86_64")
//!         .args(&["-loadvm", "root"])
//!         .run();
//! }
//! ```
//!
//! (Full example present in `examples/syscall_injection.rs`)

use std::{
    future::Future,
    sync::atomic::Ordering,
    task::{Context, Poll, RawWaker, RawWakerVTable, Waker},
};

use crate::prelude::*;
use crate::{plugins::syscalls2::Syscalls2Callbacks, regs, sys, PppCallback};

mod conversion;
mod pinned_queue;
mod syscall_future;
mod syscall_regs;
mod syscalls;

pub use {conversion::*, syscall_future::*};
use {pinned_queue::PinnedQueue, syscall_future::WAITING_FOR_SYSCALL, syscall_regs::SyscallRegs};

type Injector = dyn Future<Output = ()>;

static INJECTORS: PinnedQueue<Injector> = PinnedQueue::new();

/// Run a syscall injector in the form as an async block/value to be evaluated. If
/// another injector is already running, it will be queued to start after all previous
/// injectors have finished running.
///
/// This operates by running each system call before resuming the original system call,
/// allowing the guest to run until all injected system calls have finished.
///
/// ### Context Requirements
///
/// `run_injector` must be run within a syscall enter callback. This is enforced by
/// means of only accepting [`SyscallPc`] to prevent misuse.
///
/// If you'd like to setup an injector to run during the next system call to avoid this
/// requirement, see [`run_injector_next_syscall`].
///
/// ### Async Execution
///
/// The async runtime included allows for non-system call futures to be awaited, however
/// the async executor used does not provide any support for any level of parallelism
/// outside of Host/Guest parallelism. This means any async I/O performed will be
/// busily polled, wakers are no-ops, and executor-dependent futures will not function.
///
/// There are currently no plans for injectors to be a true-async context, so
/// outside of simple Futures it is recommended to only use the provided [`syscall`]
/// function and Futures built on top of it.
///
/// ### Behavior
///
/// The behavior of injecting into system calls which don't return, fork, or otherwise
/// effect control flow, are currently not defined.
pub fn run_injector(pc: SyscallPc, injector: impl Future<Output = ()> + 'static) {
    let pc = pc.pc();

    let is_first = INJECTORS.is_empty();
    INJECTORS.push_future(current_asid(), async {
        let backed_up_regs = SyscallRegs::backup();

        injector.await;

        backed_up_regs.restore();
        println!("Registers restored");
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

fn current_asid() -> target_ulong {
    unsafe { sys::panda_current_asid(sys::get_cpu()) }
}

/// Queue an injector to be run during the next system call.
///
/// For more information or for usage during a system call callback, see [`run_injector`].
pub fn run_injector_next_syscall(injector: impl Future<Output = ()> + 'static) {
    let next_syscall = PppCallback::new();
    let mut injector = Some(injector);

    next_syscall.on_all_sys_enter(move |_, pc, _| {
        let injector = injector.take().unwrap();
        run_injector(pc, injector);
        next_syscall.disable();
    });
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

    while let Some(mut current_injector_mutex_guard) = INJECTORS.current() {
        let (asid, ref mut current_injector) = &mut *current_injector_mutex_guard;
        // only poll from correct asid
        if *asid != current_asid() {
            return false;
        }
        match current_injector.as_mut().poll(&mut ctxt) {
            // If the current injector has finished running start polling the next
            // injector.
            Poll::Ready(_) => {
                drop(current_injector_mutex_guard);
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
