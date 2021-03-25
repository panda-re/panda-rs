use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("The provided string contained a null, which is not permitted")]
    InvalidString(#[from] std::ffi::NulError),

    #[error("The provided size was not properly page-aligned")]
    UnalignedPageSize,
}
