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

use crate::api::regs::Reg;
use crate::plugin_import;
use crate::sys::{target_ptr_t, CPUState};

use std::collections::HashSet;
use std::ops::Range;
use std::os::raw::{c_int, c_long, c_void};
use std::ptr;
use std::sync::Once;

plugin_import! {
    /// Direct access to the taint2 C API when direct use is needed
    static TAINT: Taint = extern "taint2" {
        fn taint2_enable_taint();
        fn taint2_enable_tainted_pointer();
        fn taint2_enabled() -> bool;

        fn taint2_label_addr(a: Addr, offset: c_int, label: u32);
        fn taint2_label_ram(ram_offset: u64, label: u32);
        fn taint2_label_reg(reg_num: c_int, offset: c_int, label: u32);
        fn taint2_label_io(ia: u64, label: u32);
        fn taint2_label_ram_additive(ram_offset: u64, label: u32);
        fn taint2_label_reg_additive(reg_num: c_int, offset: c_int, label: u32);
        fn taint2_label_io_additive(ia: u64, label: u32);
        fn taint2_add_taint_ram_pos(cpu: &mut CPUState, addr: u64, length: u32, start_label: u32);
        fn taint2_add_taint_ram_single_label(cpu: &mut CPUState, addr: u64, length: u32, label: c_long);

        fn taint2_delete_ram(ram_offset: u64);
        fn taint2_delete_reg(reg_num: c_int, offset: c_int);
        fn taint2_delete_io(ia: u64);

        fn taint2_query_pandalog(addr: Addr, offset: u32) -> *mut c_void;
        fn pandalog_taint_query_free(tq: *mut c_void);

        fn taint2_query(addr: Addr) -> u32;
        fn taint2_query_reg(reg_num: c_int, offset: c_int) -> u32;
        fn taint2_query_ram(ram_offset: u64) -> u32;
        fn taint2_query_laddr(la: u64, off: u64) -> u32;
        fn taint2_query_io(ia: u64) -> u32;
        fn taint2_query_llvm(reg_num: c_int, offset: c_int) -> u32;

        fn taint2_query_set_a(a: Addr, out: &mut *mut u32, outsz: &mut u32) -> u32;

        fn taint2_query_set(a: Addr, out: *mut u32);
        fn taint2_query_set_ram(ram_offset: u64, out: *mut u32);
        fn taint2_query_set_reg(reg_num: c_int, offset: c_int, out: *mut u32);
        fn taint2_query_set_io(ia: u64, out: *mut u32);

        fn taint2_query_tcn(a: Addr) -> u32;
        fn taint2_query_tcn_ram(ram_offset: u64) -> u32;
        fn taint2_query_tcn_reg(reg_num: c_int, offset: c_int) -> u32;
        fn taint2_query_tcn_io(ia: u64) -> u32;
        fn taint2_query_tcn_llvm(reg_num: c_int, offset: c_int) -> u32;

        fn taint2_query_cb_mask(a: Addr, size: u8) -> u64;

        fn taint2_labelset_addr_iter(addr: Addr, app: LabelSetVisitorRawFn, stuff: *mut c_void);
        fn taint2_labelset_ram_iter(ram_offset: u64, app: LabelSetVisitorRawFn, stuff: *mut c_void);
        fn taint2_labelset_reg_iter(reg_num: c_int, offset: c_int, app: LabelSetVisitorRawFn, stuff: *mut c_void);
        fn taint2_labelset_io_iter(ia: u64, app: LabelSetVisitorRawFn, stuff: *mut c_void);
        fn taint2_labelset_llvm_iter(reg_num: c_int, offset: c_int, app: LabelSetVisitorRawFn, stuff: *mut c_void);

        fn taint2_num_labels_applied() -> u32;

        fn taint2_track_taint_state();

        fn taint2_query_results_iter(qr: &mut QueryResult);
        fn taint2_query_result_next(qr: &mut QueryResult, done: &mut bool) -> u32;
        fn taint2_query_laddr_full(reg_num: u64, offset: u64, qr: &mut QueryResult);
        fn taint2_query_reg_full(reg_num: u32, offset: u32, qr: &mut QueryResult);
        fn taint2_query_ram_full(addr: u64, qr: &mut QueryResult);
    };
}

pub type LabelSetVisitorRawFn = extern "C" fn(u32, *mut c_void) -> c_int;

#[derive(Clone, Copy)]
#[repr(C)]
pub union ValueUnion {
    pub ha: u64,
    pub ma: u64,
    pub ia: u64,
    pub pa: u64,
    pub la: u64,
    pub gr: u64,
    pub gs: u64,
    pub ua: u64,
    pub con: u64,
    pub ret: u64,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[repr(C)]
#[allow(non_camel_case_types)]
pub enum AddrType {
    HADDR,
    MADDR,
    IADDR,
    PADDR,
    LADDR,
    GREG,
    GSPEC,
    UNK,
    CONST,
    RET,
    ADDR_LAST,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[repr(C)]
#[allow(non_camel_case_types)]
pub enum AddrFlag {
    IRRELEVANT = 5,
    EXCEPTION = 1,
    READLOG,
    FUNCARG,
}

#[derive(Clone, Copy)]
#[repr(C)]
pub struct Addr {
    pub typ: AddrType,
    pub val: ValueUnion,
    pub off: u16,
    pub flag: AddrFlag,
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

/// Add a 32-bit taint label to a given register. Any previous taint labels on the same register are not removed.
///
/// ## Example
///
/// ```no_run
/// use panda::taint;
/// use panda::regs::Reg;
///
/// // Select register by enum for compile-time guarantees
/// taint::label_reg_additive(Reg::RAX, 1);
///
/// // Select register by string when needed
/// taint::label_reg_additive("rax", 1);
/// ```
///
/// If a register is not supported by the [`Reg`] API, either make an issue or use
/// [`taint2_label_reg_additive`] directly. (example: `TAINT.taint2_label_reg_additive(reg_num, 0, label)`)
///
/// [`taint2_label_reg_additive`]: Taint::taint2_label_reg_additive
///
/// **Note**: This will enable taint if not already enabled.
pub fn label_reg_additive(register: impl Into<Reg>, label: u32) {
    let reg = register.into() as c_int;
    enable();
    for i in 0..std::mem::size_of::<target_ptr_t>() {
        TAINT.taint2_label_reg_additive(reg, i as c_int, label);
    }
}

/// Apply a 32-bit taint label to a specific byte of a given register.
///
/// ## Panics
///
/// This function panics if `byte_offset` is greater than or equal to the size of the register.
///
/// ## Example
///
/// ```no_run
/// use panda::taint;
/// use panda::regs::Reg;
///
/// // Select register by enum for compile-time guarantees
/// taint::label_reg_byte(Reg::RAX, 0, 1);
///
/// // Select register by string when needed
/// taint::label_reg_byte("rax", 0, 1);
/// ```
///
/// **Note**: This will enable taint if not already enabled.
pub fn label_reg_byte(register: impl Into<Reg>, byte_offset: usize, label: u32) {
    assert!(byte_offset < std::mem::size_of::<target_ptr_t>());

    let reg = register.into() as c_int;
    enable();
    TAINT.taint2_label_reg(reg, byte_offset as c_int, label);
}

/// Apply a 32-bit taint label to a specific byte of a given register. Any previous taint labels on the same register
/// byte are not removed.
///
/// ## Panics
///
/// This function panics if `byte_offset` is greater than or equal to the size of the register.
///
/// ## Example
///
/// ```no_run
/// use panda::taint;
/// use panda::regs::Reg;
///
/// // Select register by enum for compile-time guarantees
/// taint::label_reg_byte_additive(Reg::RAX, 0, 1);
///
/// // Select register by string when needed
/// taint::label_reg_byte_additive("rax", 0, 1);
/// ```
///
/// **Note**: This will enable taint if not already enabled.
pub fn label_reg_byte_additive(register: impl Into<Reg>, byte_offset: usize, label: u32) {
    assert!(byte_offset < std::mem::size_of::<target_ptr_t>());

    let reg = register.into() as c_int;
    enable();
    TAINT.taint2_label_reg_additive(reg, byte_offset as c_int, label);
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

/// Add a 32-bit taint label to a given byte in RAM. Any previous taint labels on the same byte are not removed.
///
/// ## Example
///
/// ```no_run
/// use panda::taint;
///
/// // Add a new taint label `4` to the byte at address 0xfffffff01c5
/// taint::label_ram_additive(0xfffffff01c5, 4);
/// ```
///
/// **Note**: This will enable taint if not already enabled.
pub fn label_ram_additive(addr: target_ptr_t, label: u32) {
    enable();
    TAINT.taint2_label_ram_additive(addr as u64, label);
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
/// taint::label_ram(start..end, 4);
/// ```
///
/// **Note**: This will enable taint if not already enabled.
pub fn label_ram_range(addr_range: Range<target_ptr_t>, label: u32) {
    enable();
    for addr in addr_range {
        TAINT.taint2_label_ram(addr as u64, label);
    }
}

/// Add a 32-bit taint label to a range of bytes in RAM. Any previous taint labels on the same range of bytes are not
/// removed.
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
/// taint::label_ram_range_additive(start..end, 4);
/// ```
///
/// **Note**: This will enable taint if not already enabled.
pub fn label_ram_range_additive(addr_range: Range<target_ptr_t>, label: u32) {
    enable();
    for addr in addr_range {
        TAINT.taint2_label_ram_additive(addr as u64, label);
    }
}

/// Removes all taint labels on all bytes of a given register.
///
/// This function effectively does nothing if taint is not enabled.
pub fn unlabel_reg(register: impl Into<Reg>) {
    if !TAINT_ENABLE.is_completed() {
        return;
    }

    let reg = register.into() as c_int;
    for i in 0..std::mem::size_of::<target_ptr_t>() {
        TAINT.taint2_delete_reg(reg, i as c_int);
    }
}

/// Removes all taint labels on a specific byte of a given register.
///
/// This function effectively does nothing if taint is not enabled.
///
/// ## Panics
///
/// This function panics if `byte_offset` is greater than or equal to the size of the register.
pub fn unlabel_reg_byte(register: impl Into<Reg>, byte_offset: usize) {
    assert!(byte_offset < std::mem::size_of::<target_ptr_t>());

    if !TAINT_ENABLE.is_completed() {
        return;
    }

    let reg = register.into() as c_int;
    TAINT.taint2_delete_reg(reg, byte_offset as c_int);
}

/// Removes all taint labels on a given byte in RAM.
///
/// This function effectively does nothing if taint is not enabled.
pub fn unlabel_ram(addr: target_ptr_t) {
    if !TAINT_ENABLE.is_completed() {
        return;
    }

    TAINT.taint2_delete_ram(addr as u64);
}

/// Removes all taint labels on a range of bytes in RAM.
///
/// This function effectively does nothing if taint is not enabled.
pub fn unlabel_ram_range(addr_range: Range<target_ptr_t>) {
    if !TAINT_ENABLE.is_completed() {
        return;
    }

    for addr in addr_range {
        TAINT.taint2_delete_ram(addr as u64);
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

/// Check if a specific byte of a register is tainted by any label
///
/// ## Panics
///
/// This function panics if `byte_offset` is greater than or equal to the size of the register.
///
/// ## Example
///
/// ```no_run
/// use panda::taint;
/// use panda::regs::Reg;
///
/// taint::label_reg_byte(Reg::RAX, 1, 1);
///
/// if taint::check_reg_byte(Reg::RAX, 1) {
///     println!("RAX[1] is tainted by some label");
/// }
/// ```
pub fn check_reg_byte(reg: impl Into<Reg>, byte_offset: usize) -> bool {
    assert!(byte_offset < std::mem::size_of::<target_ptr_t>());

    let reg_num = reg.into() as c_int;
    check_reg_num_byte(reg_num, byte_offset)
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

/// Check if a specific byte of a register is tainted by any label, by the register number
///
/// ## Panics
///
/// This function panics if `byte_offset` is greater than or equal to the size of the register.
///
/// ### Notes
///
/// * When your given register is supported in the [`Reg`] API, use [`check_reg_byte`]
/// * If taint has not been enabled by **your** plugin, this will return false
pub fn check_reg_num_byte(reg_num: c_int, byte_offset: usize) -> bool {
    assert!(byte_offset < std::mem::size_of::<target_ptr_t>());
    TAINT_ENABLE.is_completed() && TAINT.taint2_query_reg(reg_num, byte_offset as c_int) > 0
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

/// Get a list of all taint labels applied to a register, excluding duplicates across bytes
pub fn get_reg(reg: impl Into<Reg>) -> Vec<u32> {
    let labels: HashSet<u32> = iter_reg_labels(reg).collect();

    labels.into_iter().collect()
}

/// Get a list of all taint labels applied to a specific byte of a register
///
/// ## Panics
///
/// This function panics if `byte_offset` is greater than or equal to the size of the register.
pub fn get_reg_byte(reg: impl Into<Reg>, byte_offset: usize) -> Vec<u32> {
    assert!(byte_offset < std::mem::size_of::<target_ptr_t>());
    iter_reg_byte_labels(reg, byte_offset).collect()
}

/// Get a list of all taint labels applied to a byte of memory
pub fn get_ram(addr: target_ptr_t) -> Vec<u32> {
    let mut query_result = QueryResult::empty();
    TAINT.taint2_query_ram_full(addr as u64, &mut query_result);

    if check_ram(addr) {
        LabelIter {
            done: query_result.num_labels == 0,
            query_result,
        }
        .collect()
    } else {
        Vec::with_capacity(0)
    }
}

/// Get a unique list of all taint labels applied to a segment of memory
pub fn get_ram_range(addr_range: Range<target_ptr_t>) -> Vec<u32> {
    let labels: HashSet<u32> = iter_ram_labels(addr_range).collect();

    labels.into_iter().collect()
}

/// Iterate over all the taint labels applied to a given register
///
/// **NOTE**: this will repeat labels if they are applied to multiple bytes in
/// the register. For automatic deduplication behavior, try [`get_reg`].
pub fn iter_reg_labels(reg: impl Into<Reg>) -> impl Iterator<Item = u32> {
    let reg_size = std::mem::size_of::<target_ptr_t>();

    let reg = reg.into();
    (0..reg_size)
        .map(move |i| iter_reg_byte_labels(reg, i))
        .flatten()
}

/// Iterate over all the taint labels applied to a specific byte of a given register
///
/// ## Panics
///
/// This function panics if `byte_offset` is greater than or equal to the size of the register.
pub fn iter_reg_byte_labels(reg: impl Into<Reg>, byte_offset: usize) -> impl Iterator<Item = u32> {
    assert!(byte_offset < std::mem::size_of::<target_ptr_t>());

    let reg = reg.into();

    let mut query_result = QueryResult::empty();
    TAINT.taint2_query_reg_full(reg as u32, byte_offset as u32, &mut query_result);

    if TAINT.taint2_query_reg(reg as c_int, byte_offset as c_int) > 0 {
        LabelIter {
            done: query_result.is_empty_or_invalid(),
            query_result,
        }
    } else {
        LabelIter {
            done: true,
            query_result,
        }
    }
}

/// Iterate over all the taint labels applied to a segment of memory
///
/// **NOTE**: this will repeat labels if they are applied to multiple bytes in
/// the memory range. For automatic deduplication behavior, try [`get_ram_range`].
pub fn iter_ram_labels(addr_range: Range<target_ptr_t>) -> impl Iterator<Item = u32> {
    addr_range
        .map(move |addr| {
            let mut query_result = QueryResult::empty();
            TAINT.taint2_query_ram_full(addr as u64, &mut query_result);

            if check_ram(addr) {
                LabelIter {
                    done: query_result.is_empty_or_invalid(),
                    query_result,
                }
            } else {
                LabelIter {
                    done: true,
                    query_result,
                }
            }
        })
        .flatten()
}

#[repr(C)]
pub struct QueryResult {
    num_labels: u32,
    ls: *mut c_void,
    it_end: *mut c_void,
    it_curr: *mut c_void,
    tcn: u32,
    cb_mask: u8,
}

impl QueryResult {
    fn empty() -> Self {
        Self {
            num_labels: 0,
            ls: ptr::null_mut(),
            it_end: ptr::null_mut(),
            it_curr: ptr::null_mut(),
            tcn: 0,
            cb_mask: 0,
        }
    }

    fn is_empty_or_invalid(&self) -> bool {
        self.num_labels == 0 || self.it_end.is_null() || self.it_curr.is_null()
    }
}

pub struct LabelIter {
    query_result: QueryResult,
    done: bool,
}

impl Iterator for LabelIter {
    type Item = u32;

    fn next(&mut self) -> Option<Self::Item> {
        if self.done {
            None
        } else {
            Some(TAINT.taint2_query_result_next(&mut self.query_result, &mut self.done))
        }
    }
}

// TODO: sym_enable, sym_label_ram, sym_label_reg
