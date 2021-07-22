use thiserror::Error;
use std::os::raw::c_int;

// Top-level -----------------------------------------------------------------------------------------------------------

#[derive(Debug, Error)]
pub enum Error {
    #[error("The provided string contained a null, which is not permitted")]
    InvalidString(#[from] std::ffi::NulError),

    #[error("The provided size was not properly page-aligned")]
    UnalignedPageSize,

    #[error(transparent)]
    RecordReplayError(#[from] RrError)
}

// Transparent Subclasses ----------------------------------------------------------------------------------------------

#[derive(Debug, Error)]
pub enum RrError {
    #[error("Recording already enabled")]
    RrCtrlEInvalid,

    #[error("Recording enable request already pending")]
    RrCtrlEPending,
}

impl RrError {
    pub fn translate_err_code(code: c_int) -> Result<(), Error> {
        match code {
            panda_sys::RRCTRL_ret_RRCTRL_EINVALID => Err(Error::RecordReplayError(RrError::RrCtrlEInvalid)),
            panda_sys::RRCTRL_ret_RRCTRL_EPENDING => Err(Error::RecordReplayError(RrError::RrCtrlEPending)),
            panda_sys::RRCTRL_ret_RRCTRL_OK => Ok(()),
            _ => unreachable!()
        }
    }
}