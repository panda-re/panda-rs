//! Taint analysis API
//!
//! This module provides a series of helpers designed for using the [`taint2`] PANDA plugin in order
//! to perform [dynamic taint analysis] in order to help track
//!
//! [`taint2`]: https://github.com/panda-re/panda/tree/dev/panda/plugins/taint2
//! [dynamic taint analysis]: https://citeseerx.ist.psu.edu/viewdoc/download?doi=10.1.1.681.4094&rep=rep1&type=pdf
//!
//! ## Example
//! 
//! ```no_run
//! use panda::taint;
//! use panda::regs::Reg;
//!
//! // show all registers are untainted
//! for reg in [Reg::RAX, Reg::RBX, Reg::RCX, Reg::RDX] {
//!     println!("{:?} is tained? {:?}", reg, taint::check_reg(reg));
//! }
//!
//! println!("Tainting RAX...");
//! taint::label_reg(Reg::RAX, 1);
//!
//! // ...
//!
//! // show taint has propagated to any values effected by the opterations performed on RAX
//! for reg in [Reg::RAX, Reg::RBX, Reg::RCX, Reg::RDX] {
//!     println!("{:?} is tained? {:?}", reg, taint::check_reg(reg));
//! }
//! ```
//!
//! ([Full Example](https://github.com/panda-re/panda-rs/blob/master/panda-rs/examples/unicorn_taint.rs))

use crate::sys::target_ptr_t;
use crate::api::regs::Reg;
use crate::plugin_import;

use std::os::raw::c_int;
use std::sync::Once;
use std::ops::Range;

plugin_import!{
    /// Direct access to the taint2 C API when direct use is needed
    static TAINT: Taint = extern "taint2" {
        fn taint2_enable_taint();
        fn taint2_enable_tainted_pointer();
        fn taint2_enabled() -> bool;
        fn taint2_label_ram(ram_offset: u64, label: u32);
        fn taint2_label_reg(reg_num: c_int, offset: c_int, label: u32);
        fn taint2_query_reg(reg_num: c_int, offset: c_int) -> u32;
        fn taint2_query_ram(ram_offset: u64) -> u32;
        fn taint2_query_laddr(la: u64, off: u64) -> u32;
    };
}

static TAINT_ENABLE: Once = Once::new();

/// Ensure the taint system is enabled
///
/// Note: most functions call this internally, check the docs for individual helpers, as in most
/// cases you don't need to call this directly unless you want to enable the taint system earlier
/// than directly before using it.
///
/// On subsequent calls, this function will have the same performance characteristics as
/// [`Once::call_once`](https://doc.rust-lang.org/std/sync/struct.Once.html#method.call_once).
pub fn enable() {
    TAINT_ENABLE.call_once(|| {
        TAINT.taint2_enable_taint();
    })
}

/// Check if the taint system is enabled
pub fn is_enabled() -> bool {
    TAINT.taint2_enabled()
}

/// Enable pointer tainting rules. May result in overtainting.
pub fn enable_tainted_pointer() {
    TAINT.taint2_enable_tainted_pointer()
}

/// Apply a 32-bit taint label to a given register.
///
/// ## Example
///
/// ```no_run
/// use panda::taint;
/// use panda::regs::Reg;
///
/// // Select register by enum for compile-time guarantees
/// taint::label_reg(Reg::RAX, 1);
///
/// // Select register by string when needed
/// taint::label_reg("rax", 1);
/// ```
///
/// If a register is not supported by the [`Reg`] API, either make an issue or use
/// [`taint2_label_reg`] directly. (example: `TAINT.taint2_label_reg(reg_num, 0, label)`)
///
/// [`taint2_label_reg`]: Taint::taint2_label_reg
///
/// **Note**: This will enable taint if not already enabled.
pub fn label_reg(register: impl Into<Reg>, label: u32) {
    let reg = register.into() as c_int;
    enable();
    for i in 0..std::mem::size_of::<target_ptr_t>() {
        TAINT.taint2_label_reg(reg, i as c_int, label);
    }
}

/// Apply a 32-bit taint label to a given byte in RAM.
///
/// ## Example
///
/// ```no_run
/// use panda::taint;
///
/// // taint the byte at address 0xfffffff01c5 with a label of 4
/// taint::label_ram(0xfffffff01c5, 4);
/// ```
///
/// **Note**: This will enable taint if not already enabled.
pub fn label_ram(addr: target_ptr_t, label: u32) {
    enable();
    TAINT.taint2_label_ram(addr as u64, label)
}

/// Apply a 32-bit taint label to a range of bytes in RAM.
///
/// ## Example
///
/// ```no_run
/// use panda::taint;
/// use panda::prelude::*;
///
/// // Select register by enum for compile-time guarantees
/// let start = 0xfffffff01c4;
/// let end = start + std::mem::size_of::<target_ptr_t>();
/// taint::label_ram(, 4);
/// ```
///
/// **Note**: This will enable taint if not already enabled.
pub fn label_ram_range(addr_range: Range<target_ptr_t>, label: u32) {
    enable();
    for addr in addr_range {
        TAINT.taint2_label_ram(addr as u64, label)
    }
}

/// Check if a register is tainted by any label
///
/// ## Example
///
/// ```no_run
/// use panda::taint;
/// use panda::regs::Reg;
///
/// taint::label_reg(Reg::RAX);
///
/// if taint::check_reg(Reg::RAX) {
///     println!("RAX is tainted by some label");
/// }
/// ```
pub fn check_reg(reg: impl Into<Reg>) -> bool {
    let reg_num = reg.into() as c_int;
    check_reg_num(reg_num)
}

/// Check if a register is tainted by any label, by the register number
///
/// ### Notes
///
/// * When your given register is supported in the [`Reg`] API, use [`check_reg`]
/// * If taint has not been enabled by **your** plugin, this will return false
pub fn check_reg_num(reg_num: c_int) -> bool {
    TAINT_ENABLE.is_completed() && {
        let reg_size = std::mem::size_of::<target_ptr_t>();

        (0..reg_size).any(|offset| TAINT.taint2_query_reg(reg_num, offset as c_int) > 0)
    }
}

/// Check if a byte in RAM is tainted by any label
///
/// ## Example
///
/// ```no_run
/// use panda::taint;
///
/// if taint::check_ram(0xffff_0034) {
///     println!("Variable at 0xffff_0034 is tainted")
/// }
/// ```
///
/// **Note:** If taint has not been enabled by **your** plugin, this will return false
pub fn check_ram(addr: target_ptr_t) -> bool {
    TAINT_ENABLE.is_completed() && TAINT.taint2_query_ram(addr as u64) > 0
}

/// Check if any of a range of bytes in RAM is tainted by any label
///
/// ## Example
///
/// ```no_run
/// use panda::taint;
///
/// if taint::check_ram_range(0xffff_0034..0xffff_0038) {
///     println!("Variable at 0xffff_0034 is tainted")
/// }
/// ```
///
/// **Note:** If taint has not been enabled by **your** plugin, this will return false
pub fn check_ram_range(mut addr_range: Range<target_ptr_t>) -> bool {
    TAINT_ENABLE.is_completed() && addr_range.any(|addr| TAINT.taint2_query_ram(addr as u64) > 0)
}

pub fn check_laddr(addr: u64, offset: u64) -> bool {
    TAINT_ENABLE.is_completed() && TAINT.taint2_query_laddr(addr, offset) > 0
}

// TODO: get_reg, get_ram, sym_enable, sym_label_ram, sym_label_reg

