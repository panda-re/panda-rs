//! panda-rs is a set of Rust bindings for PANDA.
//!
//! **The following are provided:**
//! * Callbacks to various PANDA events in the form of attribute macros
//! * Callbacks for when guest syscalls happen
//! * Bindings to various core PANDA plugins (hooks2, osi, etc)
//! * Safe bindings to the core PANDA API
//! * An API for driving PANDA via libpanda
//! * Access to raw PANDA and QEMU API bindings via panda_sys
//!
//! ### Feature flags:
//!
//! * `libpanda` - enable libpanda mode. This is used to allow for compiling as a binary that links
//! against libpanda, for pypanda-style use.
//!
//! #### Architecture-specific features
//!
//! PANDA supports multiple architectures, but requires plugins to be compiled for each
//! architecture. In order to target a specific guest arch, use exactly one of the following:
//! `x86_64`, `i386`, `arm`, `aarch64`, `mips`, `mipsel`, `mips64`, `ppc`
//!
//! Typically PANDA plugins forward each of these features in their Cargo.toml:
//!
//! ```toml
//! [features]
//! x86_64 = ["panda/x86_64"]
//! i386 = ["panda/i386"]
//! # ...
//! ```
//!
//! ### Callbacks
//!
//! `panda-rs` makes extensive use of callbacks for handling analyses on various events. To use
//! callbacks, you simply apply the callback's attribute to any functions which should be called
//! for the given callback. In order to use a callback in a PANDA plugin (not to be confused with
//! an application that uses libpanda), one function must be marked [`#[panda::init]`](init),
//! otherwise the plugin will not work in PANDA.
//!
//! Callbacks come in two forms: free form functions (which use the attribute macros)
//! mentioned above) and closure callbacks, which use the [`Callback`] API.
//!
//! ### libpanda Mode
//!
//! PANDA also offers a dynamic library (libpanda). panda-rs allows linking against libpanda
//! instead of linking as a PANDA plugin. This creates a executable that requires libpanda to run.
//! To compile in libpanda mode, make sure the `PANDA_PATH` environment variable is set to your
//! PANDA `build` folder.
//!
//! ## Helpful Links
//!
//! | Important |    Popular Callbacks    | Popular Plugins |
//! |:---------:|:-----------------------:|:---------------:|
//! | [`init`]  | [`before_block_exec`]   | [`osi`](plugins::osi) |
//! | [`Panda`] | [`virt_mem_after_read`] | [`proc_start_linux`](plugins::proc_start_linux) |
//! | [`mod@hook`]  | [`virt_mem_after_write`]| [`hooks2`](plugins::hooks2) |
//! | [`on_sys`]| [`asid_changed`]        | [`guest_plugin_manager`](plugins::guest_plugin_manager) |
//! | [`uninit`]| [`before_block_exec_invalidate_opt`] ||
//! | [`regs`]  | [`insn_translate`]      ||
//! | [`PandaArgs`] | [`insn_exec`] ||
#![cfg_attr(doc_cfg, feature(doc_cfg))]

/// Raw bindings to the PANDA API
#[doc(inline)]
pub use panda_sys as sys;
//pub use panda_macros::*;

/// PANDA callback macros
#[doc(hidden)]
pub use panda_macros as cbs;

#[doc(inline)]
pub use plugins::hooks::hook;

#[doc(hidden)]
pub use {lazy_static, paste};

#[doc(hidden)]
extern crate self as panda;

/// Helpers and constants for interacting with various ABIs
pub mod abi;

/// Callbacks for linux syscalls (from syscalls2)
pub mod on_sys;

/// Safe wrappers for the libpanda API for helping create and manage an instance of the PANDA API
mod library_mode;
pub use library_mode::*;

mod guest_ptr;
pub use guest_ptr::*;

/// Safe wrappers for the PANDA API
mod api;
pub use api::*;

/// Architecture-specific definitions
mod arch;
pub use arch::*;

mod error;
pub use error::*;

/// Event-based callbacks, for both VM events (e.g. translation of a basic block) and PANDA events (e.g. plugin init)
mod callbacks;
pub use callbacks::*;

mod init_return;
pub use init_return::InitReturn;

/// For internal use. Access to inventory for managing callbacks.
#[doc(hidden)]
pub use inventory;

/// Helpers for getting plugin arguments from panda
pub mod panda_arg;

#[doc(inline)]
pub use panda_arg::PandaArgs;

pub mod enums;
pub mod plugins;
pub mod taint;

#[cfg_attr(doc_cfg, doc(cfg(feature = "syscall-injection")))]
#[cfg(feature = "syscall-injection")]
pub mod syscall_injection;

pub use enums::arch::*;

/// A set of types PANDA frequently requires but have a low likelihood of clashing with
/// other types you import, for use as a wildcard import.
///
/// ## Example
///
/// ```
/// use panda::prelude::*;
/// ```
pub mod prelude {
    pub use crate::panda_arg::PandaArgs;
    pub use crate::regs::SyscallPc;
    pub use crate::sys::target_long;
    pub use crate::sys::target_pid_t;
    pub use crate::sys::target_ptr_t;
    pub use crate::sys::target_ulong;
    pub use crate::sys::CPUState;
    pub use crate::sys::TranslationBlock;
    pub use crate::Panda;
    pub use crate::PluginHandle;
    pub use panda_macros::PandaArgs;
}

#[cfg(not(feature = "ppc"))]
pub use panda_macros::{on_all_sys_enter, on_all_sys_return};

// callbacks
pub use panda_macros::{
    after_block_exec, after_block_translate, after_cpu_exec_enter, after_insn_exec,
    after_insn_translate, after_loadvm, after_machine_init, asid_changed, before_block_exec,
    before_block_exec_invalidate_opt, before_block_translate, before_cpu_exec_exit,
    before_handle_exception, before_handle_interrupt, before_loadvm, before_tcg_codegen,
    cpu_restore_state, during_machine_init, end_block_exec, guest_hypercall, hd_read, hd_write,
    hook, init, insn_exec, insn_translate, main_loop_wait, mmio_after_read, mmio_before_write,
    monitor, on_mmap_updated, on_process_end, on_process_start, on_rec_auxv, on_thread_end,
    on_thread_start, phys_mem_after_read, phys_mem_after_write, phys_mem_before_read,
    phys_mem_before_write, pre_shutdown, replay_after_dma, replay_before_dma, replay_handle_packet,
    replay_hd_transfer, replay_net_transfer, replay_serial_read, replay_serial_receive,
    replay_serial_send, replay_serial_write, start_block_exec, top_loop, unassigned_io_read,
    unassigned_io_write, uninit, virt_mem_after_read, virt_mem_after_write, virt_mem_before_read,
    virt_mem_before_write, GuestType,
};
