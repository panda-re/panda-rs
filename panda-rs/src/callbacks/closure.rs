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
#[repr(transparent)]
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Callback(u64);

static CURRENT_CALLBACK_ID: AtomicU64 = AtomicU64::new(0);

impl Callback {
    pub fn new() -> Self {
        Self(CURRENT_CALLBACK_ID.fetch_add(1, Ordering::SeqCst))
    }

    pub fn enable(&self) {
        let callbacks = CALLBACKS.read().unwrap();
        let callback = callbacks.get(&self.0).unwrap();

        unsafe {
            sys::panda_enable_callback_with_context(
                get_plugin_ref(),
                callback.cb_kind,
                callback.trampoline,
                callback.closure_ref as *mut c_void,
            );
        }
    }

    pub fn disable(&self) {
        let callbacks = CALLBACKS.read().unwrap();
        let callback = callbacks.get(&self.0).unwrap();

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
