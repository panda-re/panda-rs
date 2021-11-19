use panda::plugins::osi::OSI;
use panda::prelude::*;
use panda::syscall_injection::{run_injector, syscall};

const GET_PID: target_ulong = 39;
const GET_UID: target_ulong = 102;

async fn getpid() -> target_ulong {
    syscall(GET_PID, ()).await
}

async fn getuid() -> target_ulong {
    syscall(GET_UID, ()).await
}

#[panda::on_all_sys_enter]
fn any_syscall(cpu: &mut CPUState, pc: SyscallPc, syscall_num: target_ulong) {
    if FORBIDDEN_SYSCALLS.contains(&syscall_num) || in_same_asid(cpu) {
        return;
    }

    let current_pid = OSI.get_current_process(cpu).pid;
    println!("OSI PID: {}", current_pid);

    run_injector(pc, async {
        println!("PID: {}", getpid().await);
        println!("UID: {}", getuid().await);
        println!("PID (again): {}", getpid().await);
    });
}

fn main() {
    Panda::new()
        .generic("x86_64")
        //.args(&["-loadvm", "root"])
        .run();
}

// The rest is to prevent applying syscall injectors to syscalls which might
// cause issues

use std::sync::atomic::{AtomicU64, Ordering};

fn in_same_asid(cpu: &mut CPUState) -> bool {
    static LAST_ASID: AtomicU64 = AtomicU64::new(0x1234);

    let asid = unsafe { panda::sys::panda_current_asid(cpu) };

    LAST_ASID.swap(asid, Ordering::SeqCst) == asid
}

const FORBIDDEN_SYSCALLS: &[target_ulong] = &[FORK, VFORK, EXIT_GROUP, RT_SIGRETURN];

const FORK: target_ulong = 57;
const VFORK: target_ulong = 58;
const EXIT_GROUP: target_ulong = 231;
const RT_SIGRETURN: target_ulong = 15;
