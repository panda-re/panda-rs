use std::{
    collections::HashMap,
    ffi::c_void,
    sync::{
        atomic::{AtomicU64, Ordering},
        Mutex,
    },
};

/// A reference to a given callback slot which can be used to install,
/// enable, disable, or otherwise reference, a closure-based callback.
///
/// Since this is a reference to a callback slot and does not include storage
/// for the callback itself, it can be trivially copied, as well as included in
/// the callback itself (for the purposes of enabling/disabling).
#[repr(transparent)]
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct PppCallback(pub(crate) u64);

static CURRENT_CALLBACK_ID: AtomicU64 = AtomicU64::new(0);

impl PppCallback {
    /// Create a new callback slot which can then be used to install or modify
    /// a given callback.
    pub fn new() -> Self {
        Self(CURRENT_CALLBACK_ID.fetch_add(1, Ordering::SeqCst))
    }

    /// Enable the callback assigned to the given slot, if any.
    pub fn enable(&self) {
        let mut callbacks = CALLBACKS.lock().unwrap();
        if let Some(callback) = callbacks.get_mut(&self.0) {
            if !callback.is_enabled {
                unsafe {
                    (callback.enable)(callback.closure_ref);
                }
                callback.is_enabled = true;
            }
        }
    }

    /// Disable the callback assigned to the given slot, if any.
    pub fn disable(&self) {
        let mut callbacks = CALLBACKS.lock().unwrap();

        if let Some(callback) = callbacks.get_mut(&self.0) {
            if callback.is_enabled {
                unsafe {
                    (callback.disable)(callback.closure_ref);
                }
                callback.is_enabled = false;
            }
        }
    }
}

lazy_static::lazy_static! {
    static ref CALLBACKS: Mutex<HashMap<u64, InternalPppClosureCallback>> = Mutex::new(HashMap::new());
}

#[doc(hidden)]
pub struct InternalPppClosureCallback {
    pub closure_ref: *mut c_void,
    pub enable: unsafe fn(*mut c_void),
    pub disable: unsafe fn(*mut c_void),
    pub drop_fn: unsafe fn(*mut c_void),
    pub is_enabled: bool,
}

unsafe impl Sync for InternalPppClosureCallback {}
unsafe impl Send for InternalPppClosureCallback {}

#[doc(hidden)]
pub unsafe fn __internal_install_ppp_closure_callback(
    id: u64,
    mut callback: InternalPppClosureCallback,
) {
    (callback.enable)(callback.closure_ref);
    callback.is_enabled = true;

    let mut callbacks = CALLBACKS.lock().unwrap();
    callbacks.insert(id, callback);
}
