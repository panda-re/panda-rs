/// For fallible virt/phys memory R/W operations
#[repr(i32)]
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub enum MemRWStatus {
    Unknown = -2,
    GenericErrorRet = -1,
    MemTxOk = panda_sys::MEMTX_OK as i32,
    MemTxError = panda_sys::MEMTX_ERROR as i32,
    MemTxDecodeError = panda_sys::MEMTX_DECODE_ERROR as i32,
}

impl From<i32> for MemRWStatus {
    fn from(v: i32) -> Self {
        match v {
            x if x == MemRWStatus::GenericErrorRet as i32 => MemRWStatus::GenericErrorRet,
            x if x == MemRWStatus::MemTxOk as i32 => MemRWStatus::MemTxOk,
            x if x == MemRWStatus::MemTxError as i32 => MemRWStatus::MemTxError,
            x if x == MemRWStatus::MemTxDecodeError as i32 => MemRWStatus::MemTxDecodeError,
            _ => MemRWStatus::Unknown, // This means there is a bug in the C side of things
        }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, PartialOrd, Ord, Hash)]
pub enum Endian {
    Big,
    Little,
}

/// For fallible generic C functions
#[repr(i32)]
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub enum GenericRet {
    Unknown = -2,
    Error = -1,
    Success = 0,
}

impl From<i32> for GenericRet {
    fn from(v: i32) -> Self {
        match v {
            x if x == GenericRet::Error as i32 => GenericRet::Error,
            x if x == GenericRet::Success as i32 => GenericRet::Success,
            _ => GenericRet::Unknown, // This means there is a bug in the C side of things
        }
    }
}

pub(crate) mod arch {}
