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

use dashmap::{DashMap, DashSet};
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

pub(crate) use crate::abi::set_is_sysenter;
use {
    arch::{FORK_IS_CLONE, SYSCALL_RET, VFORK},
    pinned_queue::PinnedQueue,
    syscall_future::{INJECTOR_BAIL, WAITING_FOR_SYSCALL},
    syscall_regs::SyscallRegs,
};
pub use {conversion::*, syscall_future::*};

type Injector = dyn Future<Output = ()> + 'static;

/// A unique identifier for a thread of execution. The actual makeup is not relevant
/// to use, but currently consists of process ID and thread ID pairs. The only need
/// of this is for it to be equivelant if and only if it is the same thread of execution
/// at a given point in time.
///
/// `ThreadId`s *may* be reused if the thread of execution no longer exists. Previously
/// `ThreadId`s were just ASIDs, however this may not be enough on all platforms due to
/// things such as `fork(2)` using the same ASID for both processes.
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
    /// A list of injectors. Since multiple can run at the same time, we need a mapping
    /// of which threads run which injectors. Injectors can be queued in sequence but
    /// need to be capable of pinning[^1] the current injector, hence the `PinnedQueue`.
    ///
    /// [^1]: Pinning in Rust is a concept of being able to ensure that a struct does not
    /// move. This is used by async code due to the fact that "stack" references in an
    /// async function desugars down to a reference inside of the `Future` which points
    /// to other data within the `Future`. This means the type backing the `Future` can
    /// be self-referential, so if the underlying Future is moved then the reference would
    /// be invalid. For information see [`std::pin`].
    static ref INJECTORS: DashMap<ThreadId, PinnedQueue<Injector>> = DashMap::new();

    /// A list of thread ids which have started forking but not returned from the fork
    static ref FORKING_THREADS: DashSet<ThreadId> = DashSet::new();
}

struct ChildInjector((SyscallRegs, Pin<Box<Injector>>));

unsafe impl Send for ChildInjector {}
unsafe impl Sync for ChildInjector {}

static CHILD_INJECTOR: Mutex<Option<ChildInjector>> = const_mutex(None);

static PARENT_PID: AtomicU64 = AtomicU64::new(u64::MAX);

/// Fork the guest process being injected into and begin injecting into it using the
/// provided injector.
///
/// Registers will be restored once the child process completes as well, unless the
/// child injector bails.
pub async fn fork(child_injector: impl Future<Output = ()> + 'static) -> target_ulong {
    // Since all state needs to be copied when forking, we also need to copy *our*
    // state. Since we've backed up the registers to restore once we're done injecting
    // our system calls, we need to copy those registers as well in case the user wants
    // to resume the base program's execution within the child.
    let backed_up_regs = get_backed_up_regs().expect("Fork was run outside of an injector");

    PARENT_PID.store(ThreadId::current().pid as u64, Ordering::SeqCst);

    // Used to keep track of the threads from which parent processes are forking
    FORKING_THREADS.insert(ThreadId::current());

    // This code assumes that we aren't going to be injecting into multiple processes
    // and forking at same time in an overlapping manner. Effectively this is storing
    // the future (e.g. the second injector the user passes to `fork(...)` to run in the
    // child) so that once the child process starts we can begin syscall injection there.
    CHILD_INJECTOR
        .lock()
        .replace(ChildInjector((backed_up_regs, Box::pin(child_injector))));

    // aarch64 is a new enough Linux target that it deprecates `fork(2)` entirely and
    // replaces it with the `clone(2)`. This means that for certain targets we'll have
    // our syscall number for it (`FORK`) actually be the syscall number for clone, which
    // has a different set of arguments. Currently unsupported.
    if FORK_IS_CLONE {
        todo!()
    } else {
        syscall(VFORK, ()).await
    }
}

fn get_child_injector() -> Option<(SyscallRegs, Pin<Box<Injector>>)> {
    CHILD_INJECTOR.lock().take().map(|x| x.0)
}

fn restart_syscall(cpu: &mut CPUState, pc: target_ulong) {
    regs::set_pc(cpu, pc);
    unsafe {
        panda::sys::cpu_loop_exit_noexc(cpu);
    }
}

const SYSENTER_INSTR: &[u8] = &[0xf, 0x34];

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
    log::trace!("Running injector with syscall pc of {:#x?}", pc);

    // If our syscall is a `sysenter` instruction, we need to note this so that
    // we can handle the fact that `sysenter` uses a different syscall ABI involving
    // stack storage.
    #[cfg(any(feature = "x86_64", feature = "i386"))]
    {
        use crate::mem::virtual_memory_read;

        let cpu = unsafe { &mut *sys::get_cpu() };
        let is_sysenter = virtual_memory_read(cpu, pc, 2)
            .ok()
            .map(|bytes| bytes == SYSENTER_INSTR)
            .unwrap_or(false);

        log::trace!("is_sysenter = {}", is_sysenter);
        set_is_sysenter(is_sysenter);
    }

    // Now we push the injector into the queue for the current thread so that we can
    // begin polling it. Since we can't move it once we start polling it, we need to
    // put it in the PinnedQueue before we poll it the first time
    let is_first = INJECTORS.is_empty();
    let thread_id = ThreadId::current();
    INJECTORS.entry(thread_id).or_default().push_future(async {
        let backed_up_regs = SyscallRegs::backup();
        set_backed_up_regs(backed_up_regs.clone());

        injector.await;

        backed_up_regs.restore();
        unset_backed_up_regs();
    });

    // We only want to install the callbacks once, so if there's any existing
    // callbacks in place we don't want to install them. And if another one is
    // already running, we don't want to start polling either
    if is_first {
        log::trace!("Enabling callbacks...");

        // Make callback handles so they can be self-referential in order to uninstall
        // themselves when they are done running all our injectors.
        let sys_enter = PppCallback::new();
        let sys_return = PppCallback::new();

        let disable_callbacks = move || {
            log::trace!("Disabling callbacks...");
            sys_enter.disable();
            sys_return.disable();
        };

        // after the syscall set the return value for the future then jump back to
        // the syscall instruction
        sys_return.on_all_sys_return(move |cpu: &mut CPUState, sys_pc, sys_num| {
            log::trace!(
                "on_sys_return: {} @ {:#x?} ({:#x?}?) ({:?})",
                sys_num,
                sys_pc.pc(),
                pc,
                ThreadId::current(),
            );

            if sys_num == VFORK {
                log::trace!("ret = {:#x?}", regs::get_reg(cpu, SYSCALL_RET));
            }

            let thread_id = ThreadId::current();
            if FORKING_THREADS.contains(&thread_id) {
                //if sys_num != VFORK {
                //    println!("Non-fork ({}) return from {:?}", sys_num, thread_id);
                //    println!("Non-fork ret = {:#x?}", regs::get_reg(cpu, SYSCALL_RET));
                //    return;
                //} else {
                //}
                println!("Returning from fork {:?}", &thread_id);
                FORKING_THREADS.remove(&thread_id);
            }

            let forker_pid = PARENT_PID.load(Ordering::SeqCst);

            let parent_pid = OSI.get_current_process(cpu).ppid as u64;
            let is_fork_child = FORKING_THREADS
                .iter()
                .any(|thread| thread.pid as u64 == parent_pid);

            //let is_child_of_forker =
            //    forker_pid != u64::MAX && forker_pid == ;

            if is_fork_child {
                PARENT_PID.store(u64::MAX, Ordering::SeqCst);
            }

            //let is_fork_child = is_child_of_forker;
            //let is_fork = last_injected_syscall() == VFORK || sys_num == VFORK;
            //let is_fork_child =
            //    is_child_of_forker || (is_fork && regs::get_reg(cpu, SYSCALL_RET) == 0);

            if is_fork_child {
                // If we're returning from a fork and are in the child process, retrieve
                // the previously stored child-injector, which doesn't need to back up its
                // registers since we already did that from the parent process, we just
                // need to take the previously backed-up parent process registers in
                // case we end up wanting to restore them.
                if let Some((backed_up_regs, child_injector)) = get_child_injector() {
                    INJECTORS
                        .entry(ThreadId::current())
                        .or_default()
                        .push_future(async move {
                            child_injector.await;
                            backed_up_regs.restore();
                        });
                } else {
                    println!("WARNING: failed to get child injector");
                    return;
                }
            }

            log::trace!("Current asid = {:x}", current_asid());

            // only run for the asid we're currently injecting into, unless we just forked
            if is_fork_child || is_current_injector_thread() {
                SHOULD_LOOP_AGAIN.store(true, Ordering::SeqCst);
                if !is_fork_child {
                    set_ret_value(cpu);
                }
                restart_syscall(cpu, pc);
            }
        });

        // poll the injectors and if they've all finished running, disable these
        // callbacks
        sys_enter.on_all_sys_enter(move |cpu, sys_pc, sys_num| {
            log::trace!(
                "on_sys_enter: {} @ {:#x?} ({:#x?}?)",
                sys_num,
                sys_pc.pc(),
                pc
            );

            if poll_injectors() {
                disable_callbacks();
            }

            if SHOULD_LOOP_AGAIN.swap(false, Ordering::SeqCst) {
                restart_syscall(cpu, pc);
            }
        });

        // If this is the first syscall it needs to be polled too,
        // disabling if it's already finished running
        if poll_injectors() {
            println!("WARN: Injector seemed to not call any system calls?");
            disable_callbacks();
        }
    }
}

static SHOULD_LOOP_AGAIN: AtomicBool = AtomicBool::new(false);

lazy_static! {
    static ref CURRENT_REGS_BACKUP: DashMap<ThreadId, SyscallRegs> = DashMap::new();
}

/// Get the registers set to be restored when the current injector finishes
pub fn get_backed_up_regs() -> Option<SyscallRegs> {
    CURRENT_REGS_BACKUP
        .get(&ThreadId::current())
        .map(|regs| regs.clone())
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

lazy_static! {
    static ref CURRENT_INJECTOR_THREAD: Mutex<Option<ThreadId>> = Mutex::new(None);
}

fn is_current_injector_thread() -> bool {
    CURRENT_INJECTOR_THREAD
        .lock()
        .as_ref()
        .map(|&id| id == ThreadId::current())
        .unwrap_or(false)
}

/// Returns true if all injectors have been processed
fn poll_injectors() -> bool {
    let raw = RawWaker::new(std::ptr::null(), &VTABLE);
    let waker = unsafe { Waker::from_raw(raw) };
    let mut ctxt = Context::from_waker(&waker);

    // reset the 'waiting for system call' flag
    WAITING_FOR_SYSCALL.store(false, Ordering::SeqCst);

    // Clear in case we're looping without any injectors, so a stale 'current injector'
    // won't be injected into
    CURRENT_INJECTOR_THREAD.lock().take();

    if let Some(mut injectors) = INJECTORS.get_mut(&ThreadId::current()) {
        while let Some(ref mut current_injector) = injectors.current_mut() {
            //let current_injector = &mut *current_injector;

            CURRENT_INJECTOR_THREAD.lock().replace(ThreadId::current());

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
