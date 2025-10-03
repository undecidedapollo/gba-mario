use core::ptr::copy_nonoverlapping;

use gba::{
    Align4, include_aligned_bytes,
    mmio::{BG_PALETTE, OBJ_ATTR_ALL, OBJ_PALETTE, TEXT_SCREENBLOCKS},
    prelude::{ObjAttr, ObjDisplayStyle},
    video::Color,
};

pub static SHARED_PALETTE: Align4<[u8; 24]> = include_aligned_bytes!("../asset_out/shared.palette");

pub static BACKGROUND_TILES: Align4<[u8; 10240]> =
    include_aligned_bytes!("../asset_out/tileset.sprite");
pub const BACKGROUND_TILE_COLS_PER_ROW: usize = 16;
pub static COIN_TILE: Align4<[u8; 256]> = include_aligned_bytes!("../asset_out/coin.sprite");
pub static MARIO_TILE: Align4<[u8; 2048]> = include_aligned_bytes!("../asset_out/mario.sprite");

pub const COIN_TILE_IDX_START: usize = 1;
pub const MARIO_TILE_IDX_START: usize = COIN_TILE_IDX_START + COIN_TILE.0.len() / 64;

pub fn zero_screenblock(frame: usize) {
    // Zero out the screenblock
    let zeros: [u32; 32] = core::array::repeat(0);
    for i in 0..16 {
        unsafe {
            copy_nonoverlapping(
                zeros.as_ptr(),
                TEXT_SCREENBLOCKS
                    .get_frame(frame)
                    .unwrap()
                    .get_row(i * 2)
                    .unwrap()
                    .as_usize() as *mut u32,
                zeros.len(),
            );
        }
    }
}

pub fn reset_data() {
    let mut ottr = ObjAttr::new();
    ottr.0 = ottr.0.with_style(ObjDisplayStyle::NotDisplayed);
    OBJ_ATTR_ALL.iter().for_each(|va| va.write(ottr));

    // zero_screenblock(0);
    // zero_screenblock(1);
    // zero_screenblock(2);
    // zero_screenblock(3);

    // let zeros: [u32; 32] = core::array::repeat(0);

    // Make the zero-th tile transparent
    unsafe {
        // copy_nonoverlapping(
        //     zeros.as_ptr(),
        //     CHARBLOCK0_8BPP.index(0).as_usize() as *mut u32,
        //     16,
        // );
        copy_nonoverlapping(
            SHARED_PALETTE.0.as_ptr(),
            OBJ_PALETTE.as_usize() as *mut u8,
            SHARED_PALETTE.0.len(),
        );
        copy_nonoverlapping(
            SHARED_PALETTE.0.as_ptr(),
            BG_PALETTE.as_usize() as *mut u8,
            SHARED_PALETTE.0.len(),
        );
        BG_PALETTE.index(0).write(Color(0x7E73));
        let colors: [gba::video::Color; 16] = [
            crate::color::TRANSPARENT, // Can't be accessed by the mapping function being used
            crate::color::WHITE,
            crate::color::RED,
            crate::color::GREEN,
            crate::color::BLUE,
            crate::color::YELLOW,
            crate::color::CYAN,
            crate::color::MAGENTA,
            crate::color::ORANGE,
            crate::color::PURPLE,
            crate::color::PINK,
            crate::color::BROWN,
            crate::color::GRAY,
            crate::color::LIGHT_GRAY,
            crate::color::DARK_GREEN,
            crate::color::BLACK,
        ];
        copy_nonoverlapping(
            colors.as_ptr(),
            BG_PALETTE.index(16 * 15).as_usize() as *mut Color,
            colors.len(),
        );
        // Cga8x8Thick.bitunpack_8bpp(CHARBLOCK1_8BPP.as_region(), 0);
    }
}
