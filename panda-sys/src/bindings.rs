#[cfg(feature = "x86_64")]
mod x86_64;
#[cfg(feature = "x86_64")]
pub use x86_64::*;

#[cfg(feature = "i386")]
mod i386;
#[cfg(feature = "i386")]
pub use i386::*;

#[cfg(feature = "arm")]
mod arm;
#[cfg(feature = "arm")]
pub use arm::*;

#[cfg(feature = "aarch64")]
mod aarch64;
#[cfg(feature = "aarch64")]
pub use aarch64::*;

#[cfg(feature = "ppc")]
mod ppc;
#[cfg(feature = "ppc")]
pub use ppc::*;

#[cfg(feature = "mips")]
mod mips;
#[cfg(feature = "mips")]
pub use mips::*;

#[cfg(feature = "mipsel")]
mod mipsel;
#[cfg(feature = "mipsel")]
pub use mipsel::*;

#[cfg(feature = "mips64")]
mod mipsel;
#[cfg(feature = "mips64")]
pub use mipsel::*;
