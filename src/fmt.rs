use core::ops::Deref;

use crate::math::{div10_u16, div10_u32};

#[inline(always)]
pub fn to_dec_u32<const N: usize>(mut n: u32) -> [u8; N] {
    debug_assert!(N > 0 && N <= 10);
    let mut output = [b'0'; N];

    for i in (0..N).rev() {
        let (q, d) = div10_u32(n);
        output[i] = b'0' + (d as u8);
        if q == 0 {
            break;
        }
        n = q;
    }

    output
}

#[inline(always)]
pub fn to_dec_u16<const N: usize>(mut n: u16) -> DecStr<N> {
    debug_assert!(N > 0 && N <= 5);
    let mut output = [b'0'; N];

    for i in (0..N).rev() {
        let (q, d) = div10_u16(n);
        output[i] = b'0' + (d as u8);
        if q == 0 {
            break;
        }
        n = q;
    }

    DecStr { buf: output }
}

pub struct DecStr<const N: usize> {
    buf: [u8; N],
}

impl<const N: usize> Deref for DecStr<N> {
    type Target = str;
    fn deref(&self) -> &Self::Target {
        unsafe { core::str::from_utf8_unchecked(&self.buf) }
    }
}
