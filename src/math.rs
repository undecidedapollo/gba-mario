use core::ops::Deref;

use crate::fixed_string::FixedString;

#[repr(u16)]
#[derive(Clone, Copy)]
pub enum Powers {
    _2 = 1,
    _4 = 2,
    _8 = 3,
    _16 = 4,
    _32 = 5,
    _64 = 6,
    _128 = 7,
    _256 = 8,
    _512 = 9,
    _1024 = 10,
}

impl Into<u16> for Powers {
    fn into(self) -> u16 {
        return self as u16;
    }
}

impl Powers {
    pub fn as_u16(self) -> u16 {
        Into::<u16>::into(self)
    }
}

pub fn divisible_by_num(x: u16, power: Powers) -> bool {
    let power = power.as_u16();
    (x >> power << power) == x
}

pub fn mod_mask_u32(x: u32, power: Powers) -> u32 {
    x & masks::POWERS[power.as_u16() as usize] as u32
}

pub mod masks {
    pub const POWERS: [u8; 9] = [
        0b0000_0000,
        0b0000_0001,
        0b0000_0011,
        0b0000_0111,
        0b0000_1111,
        0b0001_1111,
        0b0011_1111,
        0b0111_1111,
        0b1111_1111,
    ];
}

#[inline(always)]
pub fn div10_u32(n: u32) -> (u32, u32) {
    // q = floor(n / 10) using reciprocal multiply: (n * 0xCCCCCCCD) >> 35
    // 0xCCCCCCCD ≈ ceil(2^35 / 10)
    let q = ((n as u64 * 0xCCCC_CCCD) >> 35) as u32;
    let r = n - q * 10;
    (q, r)
}

#[inline(always)]
pub fn div10_u16(n: u16) -> (u16, u16) {
    // From chat
    // q = floor(n / 10) using reciprocal multiply: (n * 52429) >> 19
    let q = ((n as u32 * 52429) >> 19) as u16;
    let r = n - q * 10;
    (q, r)
}
