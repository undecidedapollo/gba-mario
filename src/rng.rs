use gba::random::Lcg32;

#[unsafe(link_section = ".ewram")]
static mut RNG: Lcg32 = Lcg32::new(42);

pub fn next_u32() -> u32 {
    unsafe { &mut (*core::ptr::addr_of_mut!(RNG)) }.next_u32()
}
pub fn next_usize() -> usize {
    next_u32() as usize
}

pub fn next_u16() -> u16 {
    (next_u32() >> 16) as u16
}

pub fn next_u8() -> u8 {
    (next_u32() >> 24) as u8
}

pub fn next_bool() -> bool {
    next_u32() & 0x8000_0000 != 0
}
