use std::os::raw::c_int;
use std::ffi::CString;
use std::ptr;

use crate::Error;

/// RR point-in-time: get current count of instructions replayed
pub fn rr_get_guest_instr_count() -> c_int {
    unsafe {
        panda_sys::rr_get_guest_instr_count_external()
    }
}


/// Stop and quit, wraps QMP functions.
pub fn vm_quit() -> c_int {
    unsafe {
        panda_sys::panda_replay_end()
    }
}

/// Start recording.
/// If `snapshot.is_some()` restore the named snapshot prior to recording.
pub fn record_begin(name: &str, snapshot: Option<&str>) -> Result<c_int, Error> {
    match CString::new(name) {
        Ok(c_name) => match snapshot {
            Some(snap_name) => match CString::new(snap_name) {
            Ok(c_snap_name) => Ok(unsafe { panda_sys::panda_record_begin(c_name.as_ptr(), c_snap_name.as_ptr()) }),
                Err(e) => Err(Error::InvalidString(e))
            },
            None => Ok(unsafe { panda_sys::panda_record_begin(c_name.as_ptr(), ptr::null()) }),
        },
        Err(e) => Err(Error::InvalidString(e))
    }
}

/// End currently recording.
pub fn record_end() -> c_int {
    unsafe {
        panda_sys::panda_record_end()
    }
}

/// Start replay.
pub fn replay_begin(name: &str) -> Result<c_int, Error> {
    match CString::new(name) {
        Ok(c_name) => Ok(unsafe { panda_sys::panda_replay_begin(c_name.as_ptr()) }),
        Err(e) => Err(Error::InvalidString(e))
    }
}

/// End currently running replay.
pub fn replay_end() -> c_int {
    unsafe {
        panda_sys::panda_replay_end()
    }
}