use core::ptr::copy_nonoverlapping;

use gba::{
    Align4,
    fixed::i32fx8,
    include_aligned_bytes,
    mmio::{BG_PALETTE, CHARBLOCK0_8BPP, OBJ_ATTR_ALL, OBJ_PALETTE, OBJ_TILES, TEXT_SCREENBLOCKS},
    prelude::{ObjAttr, ObjAttrWriteExt, ObjDisplayStyle},
    video::Color,
};

use crate::{
    color::darken_rgb15,
    ewram_static,
    levels::shared::{BRICK, Tile},
    static_init::StaticInitSafe,
};

pub static SHARED_PALETTE: Align4<[u8; 24]> = include_aligned_bytes!("../asset_out/shared.palette");

pub static BACKGROUND_TILES: Align4<[u8; 10240]> =
    include_aligned_bytes!("../asset_out/tileset.sprite");
pub const BACKGROUND_TILE_COLS_PER_ROW: usize = 16;
pub static COIN_TILE: Align4<[u8; 256]> = include_aligned_bytes!("../asset_out/coin.sprite");
pub static MARIO_TILE: Align4<[u8; 2048]> = include_aligned_bytes!("../asset_out/mario.sprite");

pub const COIN_TILE_IDX_START: usize = 1;
pub const MARIO_TILE_IDX_START: usize = COIN_TILE_IDX_START + COIN_TILE.0.len() / 64;
pub const BRICK_IDX_START: usize = MARIO_TILE_IDX_START + MARIO_TILE.0.len() / 64;
// Affine 2 is about the same size per stride as text, if we change affine background size (use something other than AFFINE2 we will need to change this)
pub const AFFINE2_SCREENBLOCK_START: usize = 16; // 0x0600_8000
pub const TEXT_SCREENBLOCK_START: usize = 24; // 0x0600_C000

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

pub struct AssetManager {
    change_magic: u8,
}

unsafe impl StaticInitSafe for AssetManager {
    fn init(&mut self) {
        self.reset_internal();
    }
}

ewram_static!(Asset: AssetManager = AssetManager::new());

const COLOR_MAGIC_MAX: Color = Color(0x127c);
const COLOR_MAGIC_1: Color = darken_rgb15(COLOR_MAGIC_MAX, i32fx8::from_bits(230));
const COLOR_MAGIC_2: Color = darken_rgb15(COLOR_MAGIC_MAX, i32fx8::from_bits(180));
const COLOR_MAGIC_3: Color = darken_rgb15(COLOR_MAGIC_MAX, i32fx8::from_bits(130));

unsafe fn copy_tile(tile: Tile, idx: usize) {
    unsafe {
        copy_nonoverlapping(
            CHARBLOCK0_8BPP.index(tile.top_left()).as_ptr() as *const u32,
            OBJ_TILES.index(idx * 2).as_usize() as *mut u32,
            16,
        );
        copy_nonoverlapping(
            CHARBLOCK0_8BPP.index(tile.top_right()).as_ptr() as *const u32,
            OBJ_TILES.index(idx * 2 + 2).as_usize() as *mut u32,
            16,
        );
        copy_nonoverlapping(
            CHARBLOCK0_8BPP.index(tile.bottom_left()).as_ptr() as *const u32,
            OBJ_TILES.index(idx * 2 + 4).as_usize() as *mut u32,
            16,
        );
        copy_nonoverlapping(
            CHARBLOCK0_8BPP.index(tile.bottom_right()).as_ptr() as *const u32,
            OBJ_TILES.index(idx * 2 + 6).as_usize() as *mut u32,
            16,
        );
    }
}

impl AssetManager {
    pub const fn new() -> Self {
        AssetManager { change_magic: 0 }
    }

    pub fn on_start() {
        Asset.init();
    }

    fn reset_internal(&mut self) {
        self.change_magic = 0;
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
            copy_nonoverlapping(
                BACKGROUND_TILES.0.as_ptr(),
                CHARBLOCK0_8BPP.index(1).as_usize() as *mut u8,
                BACKGROUND_TILES.0.len(),
            );
            copy_nonoverlapping(
                COIN_TILE.0.as_ptr(),
                OBJ_TILES.index(COIN_TILE_IDX_START * 2).as_usize() as *mut u8,
                COIN_TILE.0.len(),
            );
            copy_nonoverlapping(
                MARIO_TILE.0.as_ptr(),
                OBJ_TILES.index(MARIO_TILE_IDX_START * 2).as_usize() as *mut u8,
                MARIO_TILE.0.len(),
            );
            copy_tile(BRICK, BRICK_IDX_START);
            // Cga8x8Thick.bitunpack_8bpp(CHARBLOCK1_8BPP.as_region(), 0);
        }
    }

    pub fn post_tick() {
        let asset = Asset.assume_init();

        if asset.change_magic == 0 {
            BG_PALETTE.index(1).write(COLOR_MAGIC_MAX);
        } else if asset.change_magic == 8 {
            BG_PALETTE.index(1).write(COLOR_MAGIC_1);
        } else if asset.change_magic == 16 {
            BG_PALETTE.index(1).write(COLOR_MAGIC_2);
        } else if asset.change_magic == 24 {
            BG_PALETTE.index(1).write(COLOR_MAGIC_3);
        } else if asset.change_magic == 32 {
            BG_PALETTE.index(1).write(COLOR_MAGIC_2);
        } else if asset.change_magic == 40 {
            BG_PALETTE.index(1).write(COLOR_MAGIC_1);
        } else if asset.change_magic == 48 {
            BG_PALETTE.index(1).write(COLOR_MAGIC_MAX);
        } else if asset.change_magic == 64 {
            asset.change_magic = 0;
        }

        asset.change_magic = asset.change_magic.wrapping_add(1);
    }
}
