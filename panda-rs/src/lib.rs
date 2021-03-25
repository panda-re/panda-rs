//! panda-rs is a set of Rust bindings for PANDA.
//!
//! **The following are provided:**
//! * Callbacks in the form of attribute macros
//! * Access to raw PANDA API bindings via panda_sys
//!
//! ### Feature flags:
//! * `libpanda` - enable libpanda mode. This is used to allow for compiling as a binary that links
//! against libpanda, for pypanda-style use.
//!
//! ### Callbacks
//! `panda-rs` makes extensive use of callbacks for handling analyses on various events. To use
//! callbacks, you simply apply the callback's attribute to any functions which should be called
//! for the given callback. In order to use a callback in a PANDA plugin (not to be confused with
//! an application that uses libpanda), one plugin must be marked `#[panda::init]`, otherwise the
//! plugin will not work in PANDA.
//!
//! ### libpanda Mode
//!
//! PANDA also offers a dynamic library (libpanda). panda-rs allows linking against libpanda
//! instead of linking as a PANDA plugin. This creates a executable that requires libpanda to run.
//! To compile in libpanda mode, make sure the `PANDA_PATH` environment variable is set to your
//! PANDA `build` folder.

/// Raw bindings to the PANDA API
pub use panda_sys as sys;
pub use panda_macros::*;
pub use panda_macros as base_callbacks;

/// Safe wrappers for the libpanda API for helping create and manage an instance of the PANDA API
mod library_mode;
pub use library_mode::*;

/// Safe wrappers for the PANDA API
mod api;
pub use api::*;

mod error;
pub use error::*;

/// Event-based callbacks, for both VM events (e.g. translation of a basic block) and PANDA events (e.g. plugin init)
mod callbacks;
pub use callbacks::*;

/// For internal use. Access to inventory for managing callbacks.
pub use inventory;

/// Helpers for getting plugin arguments from panda
pub mod panda_arg;
pub use panda_arg::PandaArgs;

pub mod enums;
pub mod plugins;

pub mod prelude {
    pub use crate::Panda;
    pub use crate::PluginHandle;
    pub use crate::sys::target_long;
    pub use crate::sys::target_ulong;
    pub use crate::sys::target_ptr_t;
    pub use crate::sys::target_pid_t;
    pub use crate::sys::CPUState;
    pub use crate::sys::TranslationBlock;
    pub use crate::panda_arg::PandaArgs;
    pub use panda_macros::PandaArgs;
}
