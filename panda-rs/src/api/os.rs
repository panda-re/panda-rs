use crate::sys::{
    panda_os_bits, panda_os_family, panda_os_familyno, panda_os_name, panda_os_variant,
};

use std::ffi::CStr;

macro_rules! convert_static_str {
    ($str_name:ident) => {
        if unsafe { $str_name.is_null() } {
            None
        } else {
            let c_string = unsafe { CStr::from_ptr($str_name) };

            Some(c_string.to_string_lossy().into_owned())
        }
    };
}

/// Get the name of the OS currently set. This is typically set by the `-os` command line
/// argument passed to a PANDA instance.
pub fn name() -> Option<String> {
    convert_static_str!(panda_os_name)
}

/// Get the family name of the OS currently set. This is typically set by the `-os`
/// command line argument passed to a PANDA instance.
pub fn family_name() -> Option<String> {
    convert_static_str!(panda_os_family)
}

/// Get the name of the variation of the OS currently set. This is typically set by the
/// `-os` command line argument passed to a PANDA instance.
pub fn variant() -> Option<String> {
    convert_static_str!(panda_os_variant)
}

/// The bit-width of the OS being currently run. This is not necessarily equivelant to the
/// bit-width of the architecture as, for example, 32-bit Windows can run on a 64-bit
/// x86 processor.
///
/// This is typically set by the `-os` command line argument passed to a PANDA instance.
pub fn bits() -> u32 {
    unsafe { panda_os_bits }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u32)]
pub enum OsFamily {
    Unknown = 0,
    Windows = 1,
    Linux = 2,
    FreeBsd = 3,
}

impl OsFamily {
    pub fn is_linux(self) -> bool {
        self == OsFamily::Linux
    }

    pub fn is_windows(self) -> bool {
        self == OsFamily::Windows
    }

    pub fn is_bsd(self) -> bool {
        self == OsFamily::FreeBsd
    }

    pub fn is_unix(self) -> bool {
        self.is_linux() | self.is_bsd()
    }
}

impl From<u32> for OsFamily {
    fn from(fam: u32) -> Self {
        match fam {
            1 => OsFamily::Windows,
            2 => OsFamily::Linux,
            3 => OsFamily::FreeBsd,
            _ => OsFamily::Unknown,
        }
    }
}

/// The family of OS being run (Windows, Linux, etc).
///
/// This is typically set by the `-os` command line argument passed to a PANDA instance.
pub fn family() -> OsFamily {
    OsFamily::from(unsafe { panda_os_familyno })
}
