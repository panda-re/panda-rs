pub(crate) use crate::abi::syscall::*;
use crate::prelude::*;

#[cfg(feature = "x86_64")]
pub(crate) const VFORK: target_ulong = 58;

#[cfg(feature = "i386")]
pub(crate) const VFORK: target_ulong = 190;

#[cfg(feature = "arm")]
pub(crate) const VFORK: target_ulong = 190;

#[cfg(feature = "aarch64")]
pub(crate) const VFORK: target_ulong = 220;

// TODO: mips needs to be changed to VFORK
#[cfg(feature = "mips64")]
pub(crate) const VFORK: target_ulong = 5056;

#[cfg(any(feature = "mips", feature = "mipsel"))]
pub(crate) const VFORK: target_ulong = 6056; // n32

pub(crate) const FORK_IS_CLONE: bool = cfg!(feature = "aarch64");
