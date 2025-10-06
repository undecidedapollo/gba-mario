use core::ptr::copy_nonoverlapping;

use gba::{
    mmio::{CHARBLOCK1_4BPP, TEXT_SCREENBLOCKS},
    prelude::CGA_8X8_THICK,
    video::TextEntry,
};
use voladdress::{Safe, VolAddress};

use crate::{color::PaletteColor, ewram_static, static_init::StaticInitSafe};

struct CharBlockTicket {
    idx: u16,
    ticket: u16,
}

pub struct WriteTicket {
    pub ticket: u16,
}

impl WriteTicket {
    pub fn forever(self) {
        core::mem::forget(self);
    }

    pub fn clear(&mut self) {
        screen.get_or_init().unlock(self.ticket);
        while let idx = self.ticket.trailing_zeros() as u16
            && idx != 16
        {
            self.ticket &= !(1u16 << idx);
            let base_tile_idx = (idx * 32) as usize + 1;
            let tmp: [u32; 64] = core::array::repeat(0);
            unsafe {
                for idx in 0..4 {
                    copy_nonoverlapping(
                        tmp.as_ptr(),
                        CHARBLOCK1_4BPP
                            .index(base_tile_idx + (8 * idx as usize))
                            .as_usize() as *mut u32,
                        tmp.len(),
                    );
                }
            }
        }
    }
}

impl Drop for WriteTicket {
    fn drop(&mut self) {
        self.clear();
    }
}

pub struct ScreenTextManager {
    char_free_bits: u16,
}

impl ScreenTextManager {
    const fn new() -> Self {
        ScreenTextManager { char_free_bits: 0 }
    }

    fn reset_internal(&mut self) {
        self.char_free_bits = 0;
        let zeros: [u32; 64] = core::array::repeat(0);
        unsafe {
            for idx in 0..16 {
                copy_nonoverlapping(
                    zeros.as_ptr(),
                    CHARBLOCK1_4BPP.index((idx * 32) as usize + 1).as_usize() as *mut u32,
                    zeros.len(),
                );
            }
        }
    }

    fn try_lock_first_zero(&mut self) -> Option<CharBlockTicket> {
        if self.char_free_bits == u16::MAX {
            return None;
        }
        let inv = !self.char_free_bits;
        let idx = inv.trailing_zeros() as u16; // 0..15
        let ticket = 1u16 << idx;
        self.char_free_bits |= ticket; // set the bit (lock)
        Some(CharBlockTicket { idx, ticket })
    }

    fn unlock(&mut self, ticket: u16) {
        self.char_free_bits &= !ticket;
    }

    pub fn unlock_all() {
        screen.get_or_init().unlock(0xFFFF);
    }

    pub fn write_text(
        screenblock_idx: usize,
        str: &str,
        loc: (usize, usize),
        color: PaletteColor,
        overflow: bool,
    ) -> Option<WriteTicket> {
        let manager = screen.get_or_init();
        let tile_idx = manager.try_lock_first_zero().unwrap();
        let mut base_tile_idx = (tile_idx.idx * 32) as usize + 1;
        let cb: VolAddress<[u32; 8], Safe, Safe> = CHARBLOCK1_4BPP.index(base_tile_idx);
        let menu = TEXT_SCREENBLOCKS.get_frame(screenblock_idx)?;

        // assert!(b.len() >= 256);
        let mut tmp: [u32; 64] = core::array::repeat(0);
        for (idx, ch) in str.chars().into_iter().take(32).enumerate() {
            let mut x = loc.0 + idx;
            let mut y = loc.1;
            if x >= 30 {
                if !overflow {
                    break;
                }
                x -= 30;
                y += 1;
            }

            if let Some(text_entry_spot) = menu.get(x, y) {
                text_entry_spot.write(
                    TextEntry::new()
                        .with_tile(base_tile_idx as u16)
                        .with_palbank(15),
                );
            }
            let base_idx = idx * 2;
            let base_char = ch as usize * 2;
            tmp[base_idx] = CGA_8X8_THICK[base_char];
            tmp[base_idx + 1] = CGA_8X8_THICK[base_char + 1];
            base_tile_idx += 1;
        }
        // let src = unsafe { CGA_8X8_THICK.as_ptr().add((char * 2) as usize) };
        let info = gba::bios::BitUnpackInfo {
            src_byte_len: size_of_val(&tmp) as u16,
            src_elem_width: 1,
            dest_elem_width: 4,
            offset_and_touch_zero: (color as u8).saturating_sub(1) as u32,
        };
        unsafe {
            gba::bios::BitUnPack(tmp.as_ptr() as *const u8, cb.as_usize() as *mut u32, &info)
        };
        Some(WriteTicket {
            ticket: tile_idx.ticket,
        })
    }
}

ewram_static!(screen: ScreenTextManager = ScreenTextManager::new());

unsafe impl StaticInitSafe for ScreenTextManager {
    fn init(&mut self) {
        self.reset_internal();
    }
}
