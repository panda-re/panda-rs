use std::{
    collections::HashMap,
    ffi::c_void,
    sync::{
        atomic::{AtomicU64, Ordering},
        Mutex,
    },
};

/// A reference to a given callback slot which can be used to install,
/// enable, disable, or otherwise reference, a closure-based callback for
/// PANDA plugin-to-plugin ("PPP") callbacks.
///
/// Since this is a reference to a callback slot and does not include storage
/// for the callback itself, it can be trivially copied, as well as included in
/// the callback itself (for the purposes of enabling/disabling).
///
/// In order to actually install the callback, you will need to import the trait
/// for the specific plugin whose callbacks you want to run. In the example below,
/// the [`ProcStartLinuxCallbacks`] trait provides the [`on_rec_auxv`] method in
/// order to add the callback.
///
/// [`ProcStartLinuxCallbacks`]: crate::plugins::proc_start_linux::ProcStartLinuxCallbacks
/// [`on_rec_auxv`]: crate::plugins::proc_start_linux::ProcStartLinuxCallbacks::on_rec_auxv
///
/// ## Example
///
/// ```
/// use panda::plugins::proc_start_linux::ProcStartLinuxCallbacks;
/// use panda::PppCallback;
/// use panda::prelude::*;
///
/// PppCallback::new().on_rec_auxv(|_, _, auxv| {
///     dbg!(auxv);
/// });
///
/// Panda::new().generic("x86_64").replay("test").run();
/// ```
///
/// The above installs a callback to print out the contents of the auxillary vector
/// using [`dbg`] whenever a new process is spawned in the guest.
///
/// Example output:
///
/// ```no_run
/// ...
/// [panda-rs/examples/closures.rs:18] auxv = AuxvValues {
///     argc: 3,
///     argv_ptr_ptr: 0x7fffffffebb8,
///     arg_ptr: [
///         0x7fffffffede6,
///         0x7fffffffedeb,
///         0x7fffffffedee,
///     ],
///     argv: [
///         "bash",
///         "-c",
///         "echo test2",
///     ],
///     envc: 20,
///     env_ptr_ptr: 0x7fffffffebd8,
///     env_ptr: [
///         0x7fffffffedf9,
///         0x7fffffffee04,
///         // ...
///         0x7fffffffefc2,
///         0x7fffffffefe2,
///     ],
///     envp: [
///         "LS_COLORS=",
///         "LESSCLOSE=/usr/bin/lesspipe %s %s",
///         "LANG=C.UTwcap2: 0,F-8",
///         "INVOCATION_ID=0b2d5ea4eb39435388bf53e507047b2f",
///         "XDG_SESSION_ID=1",
///         "HUSHLOGIN=FALSE",
///         "USER=root",
///         "PWD=/root",
///         "HOME=/root",
///         "JOURNAL_STREAM=9:16757",
///         "XDG_DATA_DIRS=/usr/local/share:/usr/share:/var/lib/snapd/desktop",
///         "MAIL=/var/mail/root",
///         "SHELL=/bin/bash",
///         "TERM=vt220",
///         "SHLVL=1",
///         "LOGNAME=root",
///         "XDG_RUNTIME_DIR=/run/user/0",
///         "PATH=/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin:/usr/games:/usr/local/games:/snap/bin",
///         "LESSOPEN=| /usr/bin/lesspipe %s",
///         "_=/bin/bash",
///     ],
///     execfn_ptr: 0x7fffffffefee,
///     execfn: "/bin/bash",
///     phdr: 0x555555554040,
///     entry: 0x555555585520,
///     ehdr: 0x7ffff7ffa000,
///     // ...
/// }
/// Replay completed successfully
/// Exiting cpu_handle_exception loop
/// ```
///
/// ## Note
///
/// Callback closures must have a `'static` lifetime in order to live past the end of the
/// function. This means that the only references a callback can include are references
/// to static variables or leaked objects on the heap (See [`Box::leak`] for more info).
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
    PppCallback(id): PppCallback,
    mut callback: InternalPppClosureCallback,
) {
    (callback.enable)(callback.closure_ref);
    callback.is_enabled = true;

    let mut callbacks = CALLBACKS.lock().unwrap();
    callbacks.insert(id, callback);
}
