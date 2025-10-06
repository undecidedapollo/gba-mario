use gba::prelude::ObjAttr;
use voladdress::{Safe, VolAddress};

pub trait VolAddressExt {
    fn write_consecutive(&self, objattrs: &[ObjAttr]);
}

impl VolAddressExt for VolAddress<ObjAttr, Safe, ()> {
    fn write_consecutive(&self, objattrs: &[ObjAttr]) {
        unsafe { write_consecutive(objattrs, self) }
    }
}

/**
 * Writes a slice of ObjAttr to consecutive memory addresses starting from addr_start.
 *
 * # Safety
 * - The caller must ensure that the memory region starting at `addr_start` is valid for writes
 *   and is large enough to hold all `ObjAttr` entries in `objattrs`.
 */
pub unsafe fn write_consecutive(objattrs: &[ObjAttr], addr_start: &VolAddress<ObjAttr, Safe, ()>) {
    for (i, attr) in objattrs.iter().enumerate() {
        attr.write(unsafe { addr_start.add(i) });
    }
}
