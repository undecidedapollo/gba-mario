use core::ptr::copy_nonoverlapping;

use gba::{
    mmio::{CHARBLOCK1_4BPP, TEXT_SCREENBLOCKS},
    prelude::CGA_8X8_THICK,
    video::TextEntry,
};
use voladdress::{Safe, VolAddress};

use crate::{color::PaletteColor, ewram_static, gba_warning, static_init::StaticInitSafe};

pub struct CharBlockTicket {
    idx: u16,
    ticket: u16,
}

impl CharBlockTicket {
    pub fn forever(self) {
        core::mem::forget(self);
    }

    pub fn clear(&mut self) {
        gba_warning!("Clearing CharBlockTicket idx {}", self.idx);
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

impl Drop for CharBlockTicket {
    fn drop(&mut self) {
        self.clear();
    }
}

pub struct TextPalette<const N: usize> {
    chars: [char; 32],
    ticket: CharBlockTicket,
    slots: [Option<TextHandle>; N],
}

pub struct TextHandle {
    screenblock_idx: usize,
    loc: (usize, usize),
    len: usize,
}

impl TextHandle {
    pub fn clear(&mut self) {
        gba_warning!("Clearing texthandle");

        let Some(menu) = TEXT_SCREENBLOCKS.get_frame(self.screenblock_idx) else {
            return;
        };

        for idx in 0..self.len {
            let x = self.loc.0 + idx;
            let y = self.loc.1;
            if x >= 30 {
                break;
            }

            if let Some(text_entry_spot) = menu.get(x, y) {
                text_entry_spot.write(TextEntry::new());
            }
        }
    }
}

impl<const N: usize> TextPalette<N> {
    pub fn new(input: &str, ticket: CharBlockTicket, color: PaletteColor) -> Self {
        let base_tile_idx = (ticket.idx * 32) as usize + 1;
        let cb: VolAddress<[u32; 8], Safe, Safe> = CHARBLOCK1_4BPP.index(base_tile_idx);

        let mut chars = [' '; 32];
        let mut tmp: [u32; 64] = core::array::repeat(0);
        for (idx, ch) in input.chars().enumerate().take(32) {
            chars[idx] = ch;
            let base_idx = idx * 2;
            let base_char = ch as usize * 2;
            tmp[base_idx] = CGA_8X8_THICK[base_char];
            tmp[base_idx + 1] = CGA_8X8_THICK[base_char + 1];
            let info = gba::bios::BitUnpackInfo {
                src_byte_len: size_of_val(&tmp) as u16,
                src_elem_width: 1,
                dest_elem_width: 4,
                offset_and_touch_zero: (color as u8).saturating_sub(1) as u32,
            };
            unsafe {
                gba::bios::BitUnPack(tmp.as_ptr() as *const u8, cb.as_usize() as *mut u32, &info)
            };
        }
        TextPalette {
            chars,
            ticket,
            slots: [const { None }; N],
        }
    }

    pub fn find_tile_idx(&self, ch: char) -> Option<u16> {
        self.chars.iter().position(|&c| c == ch).map(|i| {
            let base_tile_idx = (self.ticket.idx * 32) as usize + 1;
            (base_tile_idx + i) as u16
        })
    }

    pub fn write_text<'a>(
        &'a mut self,
        slot: usize,
        screenblock_idx: usize,
        str: &str,
        loc: (usize, usize),
        clear_first: bool,
    ) -> Option<&'a TextHandle> {
        let menu = TEXT_SCREENBLOCKS.get_frame(screenblock_idx)?;

        for (idx, ch) in str.chars().into_iter().take(32).enumerate() {
            if loc.0 + idx >= 30 {
                break;
            }

            menu.index(loc.0 + idx, loc.1).write(
                TextEntry::new()
                    .with_tile(self.find_tile_idx(ch)?)
                    .with_palbank(15),
            );
        }

        self.slots.get_mut(slot).and_then(|s| {
            if s.is_some() {
                if clear_first {
                    s.take().map(|mut x| x.clear());
                }
            }
            s.replace(TextHandle {
                screenblock_idx,
                loc,
                len: str.len().min(32),
            })
        });
        self.slots.get(slot).and_then(|s| s.as_ref())
    }

    pub fn clear_text(&mut self, slot: usize) {
        if let Some(Some(handle)) = self.slots.get_mut(slot) {
            handle.clear();
            self.slots[slot] = None;
        }
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

    pub fn create_palette<const N: usize>(chars: &str, color: PaletteColor) -> TextPalette<N> {
        let manager = screen.get_or_init();
        let tile_idx = manager.try_lock_first_zero().unwrap();
        TextPalette::new(chars, tile_idx, color)
    }
}

ewram_static!(screen: ScreenTextManager = ScreenTextManager::new());

unsafe impl StaticInitSafe for ScreenTextManager {
    fn init(&mut self) {
        self.reset_internal();
    }
}
