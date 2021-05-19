use std::os::raw::c_int;
use std::ffi::CString;
use std::ptr;

use crate::{Error, RrError};

/// RR point-in-time: get current count of instructions replayed
pub fn rr_get_guest_instr_count() -> c_int {
    unsafe {
        panda_sys::rr_get_guest_instr_count_external()
    }
}

/// Stop and quit, wraps QMP functions.
pub fn vm_quit() {
    let rr_ctrl_ret = unsafe { panda_sys::panda_vm_quit() };
    // Non-fallible: https://sourcegraph.com/github.com/panda-re/panda/-/blob/panda/src/callbacks.c#L755:5
    // Defensive assert in case C-side ever becomes fallible
    assert_eq!(rr_ctrl_ret, panda_sys::RRCTRL_ret_RRCTRL_OK);
}

/// Start recording.
/// If `snapshot.is_some()` restore the named snapshot prior to recording.
pub fn record_begin(name: &str, snapshot: Option<&str>) -> Result<(), Error> {
    match CString::new(name) {
        Ok(c_name) => match snapshot {
            Some(snap_name) => match CString::new(snap_name) {
                Ok(c_snap_name) => {
                    let rr_ctrl_ret = unsafe { panda_sys::panda_record_begin(c_name.as_ptr(), c_snap_name.as_ptr()) };
                    RrError::translate_err_code(rr_ctrl_ret)
                },
                Err(e) => Err(Error::InvalidString(e))
            },
            None => {
                let rr_ctrl_ret = unsafe { panda_sys::panda_record_begin(c_name.as_ptr(), ptr::null()) };
                RrError::translate_err_code(rr_ctrl_ret)
            },
        },
        Err(e) => Err(Error::InvalidString(e))
    }
}

/// End currently recording.
pub fn record_end() -> Result<(), Error> {
    let rr_ctrl_ret = unsafe { panda_sys::panda_record_end() };
    RrError::translate_err_code(rr_ctrl_ret)
}

/// Start replay.
pub fn replay_begin(name: &str) -> Result<(), Error> {
    match CString::new(name) {
        Ok(c_name) => {
            let rr_ctrl_ret = unsafe { panda_sys::panda_replay_begin(c_name.as_ptr()) };
            RrError::translate_err_code(rr_ctrl_ret)
        },
        Err(e) => Err(Error::InvalidString(e))
    }
}

/// End currently running replay.
pub fn replay_end() -> Result<(), Error> {
    let rr_ctrl_ret = unsafe { panda_sys::panda_replay_end() };
    RrError::translate_err_code(rr_ctrl_ret)
}