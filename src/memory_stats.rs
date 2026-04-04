//! Runtime memory usage reporting, backed by symbols defined in `mono_boot.ld`.
//!
//! The linker computes these at link time, so reading them is free and the
//! numbers always match the final binary. Addresses of these symbols *are*
//! the values — we never dereference them.

use crate::{gba_info, gba_warning};

unsafe extern "C" {
    static __ewram_capacity: u8;
    static __iwram_capacity: u8;
    static __ewram_used: u8;
    static __iwram_used: u8;
    static __ewram_free: u8;
    static __iwram_free: u8;
}

#[inline]
fn sym_addr(s: &u8) -> usize {
    core::ptr::addr_of!(*s) as usize
}

pub fn ewram_capacity() -> usize {
    unsafe { sym_addr(&__ewram_capacity) }
}
pub fn iwram_capacity() -> usize {
    unsafe { sym_addr(&__iwram_capacity) }
}
pub fn ewram_used() -> usize {
    unsafe { sym_addr(&__ewram_used) }
}
pub fn iwram_used() -> usize {
    unsafe { sym_addr(&__iwram_used) }
}
pub fn ewram_free() -> usize {
    unsafe { sym_addr(&__ewram_free) }
}
pub fn iwram_free() -> usize {
    unsafe { sym_addr(&__iwram_free) }
}

/// Log a one-shot memory report at boot.
pub fn log_report() {
    let ew_used = ewram_used();
    let ew_cap = ewram_capacity();
    let iw_used = iwram_used();
    let iw_cap = iwram_capacity();

    // percent * 10 (one decimal place) without floats
    let ew_pct_x10 = ((ew_used as u64) * 1000 / ew_cap as u64) as u32;
    let iw_pct_x10 = ((iw_used as u64) * 1000 / iw_cap as u64) as u32;

    gba_warning!(
        "EWRAM: {}/{} bytes used ({}.{}%) — {} free",
        ew_used,
        ew_cap,
        ew_pct_x10 / 10,
        ew_pct_x10 % 10,
        ewram_free()
    );
    gba_warning!(
        "IWRAM: {}/{} bytes used ({}.{}%) — {} free",
        iw_used,
        iw_cap,
        iw_pct_x10 / 10,
        iw_pct_x10 % 10,
        iwram_free()
    );
}
