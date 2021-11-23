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
    pin::Pin,
    sync::atomic::{AtomicBool, AtomicU64, Ordering},
    task::{Context, Poll, RawWaker, RawWakerVTable, Waker},
};

use dashmap::DashMap;
use lazy_static::lazy_static;
use parking_lot::{const_mutex, Mutex};

use crate::prelude::*;
use crate::{
    plugins::{osi::OSI, syscalls2::Syscalls2Callbacks},
    regs, sys, PppCallback,
};

mod arch;
mod conversion;
mod pinned_queue;
mod syscall_future;
mod syscall_regs;
mod syscalls;

use {
    arch::{FORK, FORK_IS_CLONE, SYSCALL_RET},
    pinned_queue::PinnedQueue,
    syscall_future::{INJECTOR_BAIL, WAITING_FOR_SYSCALL},
    syscall_regs::SyscallRegs,
};
pub use {conversion::*, syscall_future::*};

type Injector = dyn Future<Output = ()>;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct ThreadId {
    pid: target_ulong,
    tid: target_ulong,
}

impl ThreadId {
    fn current() -> Self {
        let cpu = unsafe { &mut *sys::get_cpu() };
        let thread = OSI.get_current_thread(cpu);

        let tid = thread.tid as target_ulong;
        let pid = thread.pid as target_ulong;
        Self { tid, pid }
    }
}

lazy_static! {
    static ref INJECTORS: DashMap<ThreadId, PinnedQueue<Injector>> = DashMap::new();
}

struct ChildInjector((SyscallRegs, Pin<Box<dyn Future<Output = ()> + 'static>>));

unsafe impl Send for ChildInjector {}
unsafe impl Sync for ChildInjector {}

static CHILD_INJECTOR: Mutex<Option<ChildInjector>> = const_mutex(None);

pub async fn fork(child_injector: impl Future<Output = ()> + 'static) -> target_ulong {
    let backed_up_regs = get_backed_up_regs().expect("Fork was run outside of an injector");
    CHILD_INJECTOR
        .lock()
        .replace(ChildInjector((backed_up_regs, Box::pin(child_injector))));

    if FORK_IS_CLONE {
        todo!()
    } else {
        syscall(FORK, ()).await
    }
}

fn get_child_injector() -> (SyscallRegs, Pin<Box<dyn Future<Output = ()> + 'static>>) {
    CHILD_INJECTOR.lock().take().unwrap().0
}

fn restart_syscall(cpu: &mut CPUState, pc: target_ulong) {
    regs::set_pc(cpu, pc);
    unsafe {
        panda::sys::cpu_loop_exit_noexc(cpu);
    }
}

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
    let thread_id = ThreadId::current();
    INJECTORS
        .entry(thread_id)
        .or_default()
        .push_future(current_asid(), async {
            let backed_up_regs = SyscallRegs::backup();
            set_backed_up_regs(backed_up_regs.clone());

            injector.await;

            backed_up_regs.restore();
            unset_backed_up_regs();
        });

    // Only install each callback once
    if is_first {
        let sys_enter = PppCallback::new();
        let sys_return = PppCallback::new();

        // after the syscall set the return value for the future then jump back to
        // the syscall instruction
        sys_return.on_all_sys_return(move |cpu: &mut CPUState, _, sys_num| {
            let is_fork = last_injected_syscall() == FORK || sys_num == FORK;
            let is_fork_child = is_fork && regs::get_reg(cpu, SYSCALL_RET) == 0;

            if is_fork_child {
                // set up a child-injector, which doesn't back up its registers, only
                // sets up to restore the registers of its parent
                let (backed_up_regs, child_injector) = get_child_injector();
                INJECTORS
                    .entry(ThreadId::current())
                    .or_default()
                    .push_future(current_asid(), async move {
                        child_injector.await;
                        backed_up_regs.restore();
                    });
            }

            // only run for the asid we're currently injecting into, unless we just forked
            if is_fork_child
                || (CURRENT_INJECTOR_ASID.load(Ordering::SeqCst) == current_asid() as u64)
            {
                SHOULD_LOOP_AGAIN.store(true, Ordering::SeqCst);
                set_ret_value(cpu);
                restart_syscall(cpu, pc);
            }
        });

        // poll the injectors and if they've all finished running, disable these
        // callbacks
        sys_enter.on_all_sys_enter(move |cpu, _, _| {
            if poll_injectors() {
                sys_enter.disable();
                sys_return.disable();
            }

            if SHOULD_LOOP_AGAIN.swap(false, Ordering::SeqCst) {
                restart_syscall(cpu, pc);
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

static SHOULD_LOOP_AGAIN: AtomicBool = AtomicBool::new(false);

lazy_static! {
    static ref CURRENT_REGS_BACKUP: DashMap<ThreadId, SyscallRegs> = DashMap::new();
}

pub fn get_backed_up_regs() -> Option<SyscallRegs> {
    CURRENT_REGS_BACKUP
        .get(&ThreadId::current())
        .map(|x| x.clone())
}

fn set_backed_up_regs(regs: SyscallRegs) {
    CURRENT_REGS_BACKUP.insert(ThreadId::current(), regs);
}

fn unset_backed_up_regs() {
    CURRENT_REGS_BACKUP.remove(&ThreadId::current());
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

static CURRENT_INJECTOR_ASID: AtomicU64 = AtomicU64::new(0);

/// Returns true if all injectors have been processed
fn poll_injectors() -> bool {
    let raw = RawWaker::new(std::ptr::null(), &VTABLE);
    let waker = unsafe { Waker::from_raw(raw) };
    let mut ctxt = Context::from_waker(&waker);

    // reset the 'waiting for system call' flag
    WAITING_FOR_SYSCALL.store(false, Ordering::SeqCst);

    // Clear in case we're looping without any injectors, so a stale 'current injector'
    // won't be injected into
    CURRENT_INJECTOR_ASID.store(u64::MAX, Ordering::SeqCst);

    if let Some(mut injectors) = INJECTORS.get_mut(&ThreadId::current()) {
        while let Some(ref mut current_injector) = injectors.current_mut() {
            let (asid, ref mut current_injector) = &mut *current_injector;
            CURRENT_INJECTOR_ASID.store(*asid as u64, Ordering::SeqCst);

            // only poll from correct asid
            if *asid != current_asid() {
                return false;
            }

            match current_injector.as_mut().poll(&mut ctxt) {
                // If the current injector has finished running start polling the next
                // injector. This includes if the current injector bails early.
                status
                    if matches!(status, Poll::Ready(_))
                        || INJECTOR_BAIL.swap(false, Ordering::SeqCst) =>
                {
                    injectors.pop();

                    // No more injectors in the current thread
                    if injectors.is_empty() {
                        drop(injectors);
                        INJECTORS.remove(&ThreadId::current());
                        break;
                    }

                    continue;
                }

                // If the future is now waiting on a syscall to be evaluated, return
                // so a system call can be run
                Poll::Pending if waiting_for_syscall() => return false,

                // If the future is not waiting on a system call we should keep polling
                Poll::Pending => continue,

                _ => unreachable!(),
            }
        }
    } else {
        return false;
    }

    let all_injectors_finished = INJECTORS.is_empty() && CHILD_INJECTOR.lock().is_none();

    all_injectors_finished
}
