use core::fmt::{self, Write};

use gba::Align4;

use crate::static_init::StaticInitSafe;

/// A fixed-capacity string buffer in EWRAM.
///
/// Not heap-allocated. All bytes live in `.ewram`, and you use
/// `write!` / `writeln!` macros to append to it.
pub struct FixedString<const N: usize> {
    pub buf: Align4<[u8; N]>,
    pub len: usize,
}

impl<const N: usize> FixedString<N> {
    pub const fn new() -> Self {
        FixedString {
            buf: gba::Align4([0; N]),
            len: 0,
        }
    }

    pub fn from_str(s: &str) -> Self {
        let mut fs = Self::new();
        let _ = write!(fs, "{}", s);
        fs
    }

    /// Clears the buffer (just resets length, doesn’t zero the bytes).
    pub fn clear(&mut self) {
        self.len = 0;
    }

    /// Returns the current contents as a &str.
    pub fn as_str(&self) -> &str {
        // SAFETY: only writes come from valid UTF-8 sources.
        unsafe { core::str::from_utf8_unchecked(&self.buf.0[..self.len]) }
    }
}

impl<const N: usize> Write for FixedString<N> {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        let dst = self.buf.0.as_mut_slice();
        let rem = dst.len().saturating_sub(self.len);
        let take = rem.min(s.len());
        dst[self.len..self.len + take].copy_from_slice(&s.as_bytes()[..take]);
        self.len += take;
        Ok(())
    }
}

unsafe impl<const N: usize> StaticInitSafe for FixedString<N> {
    // Uses default no-op init
}
