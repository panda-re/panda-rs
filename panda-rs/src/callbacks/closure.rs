use std::{
    collections::HashMap,
    ffi::c_void,
    sync::{
        atomic::{AtomicU64, Ordering},
        RwLock,
    },
};

use once_cell::sync::OnceCell;

use crate::sys::{hwaddr, target_ptr_t, CPUState, MachineState, Monitor, TranslationBlock};
use crate::{sys, PluginHandle};

/// A reference to a given callback slot which can be used to install,
/// enable, disable, or otherwise reference, a closure-based callback.
///
/// Since this is a reference to a callback slot and does not include storage
/// for the callback itself, it can be trivially copied, as well as included in
/// the callback itself (for the purposes of enabling/disabling).
///
/// ## Example
///
/// ```
/// use panda::prelude::*;
/// use panda::Callback;
///
/// let mut count = 0;
/// let bb_callback = Callback::new();
/// bb_callback.before_block_exec(move |_, _| {
///     count += 1;
///     println!("Basic block #{}", count);
///
///     if count > 5 {
///         bb_callback.disable();
///     }
/// });
///
/// Panda::new()
///    .generic("x86_64")
///    .run();
/// ```
///
/// ## Note
///
/// Callback closures must have a static lifetime in order to live past the end of the
/// function. This means that the only references a callback can include are references
/// to static variables or leaked objects on the heap (See `Box::leak` for more info).
///
/// If you'd like to reference shared data without leaking, this can be accomplished via
/// reference counting. See [`Arc`] for more info. If you want to capture data owned
/// by the current function without sharing it, you can mark your closure as `move` in
/// order to move all the variables you capture into your closure. (Such as in the above
/// example, where `count` is moved into the closure for modification)
///
/// [`Arc`]: https://doc.rust-lang.org/std/sync/struct.Arc.html
#[repr(transparent)]
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Callback(u64);

static CURRENT_CALLBACK_ID: AtomicU64 = AtomicU64::new(0);

impl Callback {
    /// Create a new callback slot which can then be used to install or modify
    /// a given callback.
    pub fn new() -> Self {
        Self(CURRENT_CALLBACK_ID.fetch_add(1, Ordering::SeqCst))
    }

    /// Enable the callback assigned to the given slot, if any.
    pub fn enable(&self) {
        let callbacks = CALLBACKS.read().unwrap();
        if let Some(callback) = callbacks.get(&self.0) {
            unsafe {
                sys::panda_enable_callback_with_context(
                    get_plugin_ref(),
                    callback.cb_kind,
                    callback.trampoline,
                    callback.closure_ref as *mut c_void,
                );
            }
        }
    }

    /// Disable the callback assigned to the given slot, if any.
    pub fn disable(&self) {
        let callbacks = CALLBACKS.read().unwrap();

        if let Some(callback) = callbacks.get(&self.0) {
            unsafe {
                sys::panda_disable_callback_with_context(
                    get_plugin_ref(),
                    callback.cb_kind,
                    callback.trampoline,
                    callback.closure_ref as *mut c_void,
                );
            }
        }
    }
}

struct ClosureCallback {
    closure_ref: *mut *mut c_void,
    cb_kind: sys::panda_cb_type,
    trampoline: sys::panda_cb_with_context,
    drop_fn: unsafe fn(*mut *mut c_void),
}

unsafe impl Sync for ClosureCallback {}
unsafe impl Send for ClosureCallback {}

lazy_static::lazy_static! {
    static ref CALLBACKS: RwLock<HashMap<u64, ClosureCallback>> = RwLock::new(HashMap::new());
}

static PLUGIN_REF: OnceCell<u64> = OnceCell::new();

#[doc(hidden)]
pub fn set_plugin_ref(plugin: *mut PluginHandle) {
    let _ = PLUGIN_REF.set(plugin as u64);
}

fn get_plugin_ref() -> *mut c_void {
    *PLUGIN_REF.get_or_init(|| &PLUGIN_REF as *const _ as u64) as _
}

fn install_closure_callback(id: u64, callback: ClosureCallback) {
    unsafe {
        sys::panda_register_callback_with_context(
            get_plugin_ref(),
            callback.cb_kind,
            callback.trampoline,
            callback.closure_ref as *mut c_void,
        );
    }

    CALLBACKS.write().unwrap().insert(id, callback);
}

impl std::ops::Drop for ClosureCallback {
    fn drop(&mut self) {
        unsafe { (self.drop_fn)(self.closure_ref) }
    }
}

panda_macros::define_closure_callbacks!();
