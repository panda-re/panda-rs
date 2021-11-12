use crate::enums::Endian;

// ================ ARCH_NAME ================

/// The name of the architecture as used by PANDA
///
/// This can be one of:
///
/// * x86_64
/// * i386
/// * arm
/// * ppc
/// * mips
/// * mipsel
/// * mips64
/// * aarch64
pub const ARCH_NAME: &str = ARCH;

#[cfg(feature = "x86_64")]
const ARCH: &str = "x86_64";

#[cfg(feature = "i386")]
const ARCH: &str = "i386";

#[cfg(feature = "arm")]
const ARCH: &str = "arm";

#[cfg(feature = "ppc")]
const ARCH: &str = "ppc";

#[cfg(feature = "mips")]
const ARCH: &str = "mips";

#[cfg(feature = "mipsel")]
const ARCH: &str = "mipsel";

#[cfg(feature = "aarch64")]
const ARCH: &str = "aarch64";

#[cfg(feature = "mips64")]
const ARCH: &str = "mips64";

// ================ ARCH_ENDIAN ================

/// The byte order of the guest architecture being targetted by PANDA
pub const ARCH_ENDIAN: Endian = ENDIAN;

#[cfg(feature = "x86_64")]
const ENDIAN: Endian = Endian::Little;

#[cfg(feature = "i386")]
const ENDIAN: Endian = Endian::Little;

#[cfg(feature = "arm")]
const ENDIAN: Endian = Endian::Little;

#[cfg(feature = "ppc")]
const ENDIAN: Endian = Endian::Big;

#[cfg(feature = "mips")]
const ENDIAN: Endian = Endian::Big;

#[cfg(feature = "mipsel")]
const ENDIAN: Endian = Endian::Little;

#[cfg(feature = "aarch64")]
const ENDIAN: Endian = Endian::Little;

#[cfg(feature = "mips64")]
const ENDIAN: Endian = Endian::Big;
