//! Taint analysis API

use crate::sys::target_ptr_t;
use crate::api::regs::Reg;
use crate::plugin_import;

use std::os::raw::c_int;
use std::sync::Once;
use std::ops::Range;

plugin_import!{
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

pub fn enable() {
    TAINT_ENABLE.call_once(|| {
        TAINT.taint2_enable_taint();
    })
}

pub fn is_enabled() -> bool {
    TAINT.taint2_enabled()
}

pub fn enable_tainted_pointer() {
    TAINT.taint2_enable_tainted_pointer()
}

pub fn label_reg(register: impl Into<Reg>, label: u32) {
    let reg = register.into() as c_int;
    enable();
    for i in 0..std::mem::size_of::<target_ptr_t>() {
        TAINT.taint2_label_reg(reg, i as c_int, label);
    }
}

pub fn label_ram(addr: target_ptr_t, label: u32) {
    enable();
    TAINT.taint2_label_ram(addr as u64, label)
}

pub fn label_ram_range(addr_range: Range<target_ptr_t>, label: u32) {
    enable();
    for addr in addr_range {
        TAINT.taint2_label_ram(addr as u64, label)
    }
}

pub fn check_reg(reg: impl Into<Reg>) -> bool {
    let reg_num = reg.into() as c_int;
    check_reg_num(reg_num)
}

pub fn check_reg_num(reg_num: c_int) -> bool {
    TAINT_ENABLE.is_completed() && {
        let reg_size = std::mem::size_of::<target_ptr_t>();

        (0..reg_size).any(|offset| TAINT.taint2_query_reg(reg_num, offset as c_int) > 0)
    }
}

pub fn check_ram(addr: target_ptr_t) -> bool {
    TAINT_ENABLE.is_completed() && TAINT.taint2_query_ram(addr as u64) > 0
}

pub fn check_ram_range(mut addr_range: Range<target_ptr_t>) -> bool {
    TAINT_ENABLE.is_completed() && addr_range.any(|addr| TAINT.taint2_query_ram(addr as u64) > 0)
}

pub fn check_laddr(addr: u64, offset: u64) -> bool {
    TAINT_ENABLE.is_completed() && TAINT.taint2_query_laddr(addr, offset) > 0
}

// TODO: get_reg, get_ram, sym_enable, sym_label_ram, sym_label_reg

