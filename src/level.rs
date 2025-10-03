use gba::prelude::*;

use crate::{
    ewram_static, gba_warning,
    math::{Powers, mod_mask_u32},
    screen::ScreenManager,
    static_init::StaticInitSafe,
    tick::TickContext,
};

pub struct LevelManager {
    rendered_col: u16,
    reaped_col: u16,
}

impl LevelManager {
    pub const fn new() -> Self {
        LevelManager {
            rendered_col: 0,
            reaped_col: 0,
        }
    }

    fn reset_internal(&mut self) {}

    pub fn init() {
        Level.init();
    }

    pub fn is_standable(mut row: u16, col: u16) -> bool {
        row = mod_mask_u32(row as u32, Powers::_32) as u16;
        let screen_details = ScreenManager::get_screen_info();
        let end = screen_details.onscreen_col_end();
        if col < screen_details.onscreen_col_start || col > end {
            gba_warning!(
                "Checking standable out of screen bounds: {},{} (screen {}-{})",
                col,
                row,
                screen_details.onscreen_col_start,
                end
            );
            return false;
        }

        let screenblock_col: usize = mod_mask_u32(col as u32, Powers::_32) as usize;

        let Some(tile) = AFFINE2_SCREENBLOCKS
            .get_frame(16)
            .unwrap()
            .get(screenblock_col, row.into())
            .map(|x| x.read())
        else {
            gba_warning!("Failed to read tile at {},{}", col, row);
            return false;
        };
        tile.high() != 0 || tile.low() != 0
    }

    fn process_screen(&mut self) {
        let screen_details = ScreenManager::get_screen_info();
        let start = screen_details.onscreen_col_start;
        let end: u16 = screen_details.onscreen_col_end();
        let render_end = end + 4;
        let reap = (start as u16).saturating_sub(4);
        if self.rendered_col >= render_end {
            return;
        }

        for i in self.rendered_col..render_end {
            let screenblock_col: usize = mod_mask_u32(i as u32, Powers::_32) as usize;

            if screenblock_col == 21 || screenblock_col == 22 {
                continue;
            }

            if screenblock_col > 14 && screenblock_col < 18 {
                AFFINE2_SCREENBLOCKS
                    .get_frame(16)
                    .unwrap()
                    .index(screenblock_col, 8)
                    .write(u8x2::default().with_high(1).with_low(2));
                AFFINE2_SCREENBLOCKS
                    .get_frame(16)
                    .unwrap()
                    .index(screenblock_col, 9)
                    .write(u8x2::default().with_high(17).with_low(18));
            }

            // gba_warning!("Rendering column {} actual {}", i, screenblock_col);
            AFFINE2_SCREENBLOCKS
                .get_frame(16)
                .unwrap()
                .index(screenblock_col, 16)
                .write(u8x2::default().with_high(3).with_low(4));
            AFFINE2_SCREENBLOCKS
                .get_frame(16)
                .unwrap()
                .index(screenblock_col, 17)
                .write(u8x2::default().with_high(19).with_low(20));
            AFFINE2_SCREENBLOCKS
                .get_frame(16)
                .unwrap()
                .index(screenblock_col, 18)
                .write(u8x2::default().with_high(3).with_low(4));
            AFFINE2_SCREENBLOCKS
                .get_frame(16)
                .unwrap()
                .index(screenblock_col, 19)
                .write(u8x2::default().with_high(19).with_low(20));
        }
        self.rendered_col = render_end;
        for i in self.reaped_col..reap {
            let screenblock_col = mod_mask_u32(i as u32, Powers::_32) as usize;
            // gba_warning!("Reaping column {} actual {}", i, screenblock_col);
            for i in 0..32 {
                AFFINE2_SCREENBLOCKS
                    .get_frame(16)
                    .unwrap()
                    .index(screenblock_col, i)
                    .write(u8x2::default().with_high(0).with_low(0));
            }
        }
        self.reaped_col = reap;
    }

    pub fn tick(_tick: TickContext) {
        let manager = Level.get_or_init();
        if _tick.tick_count != 0 && _tick.tick_count % 10 == 0 {
            // ScreenManager::translate_x(i32fx8::wrapping_from(8));
        }

        manager.process_screen();
    }
}

unsafe impl StaticInitSafe for LevelManager {
    fn init(&mut self) {
        self.reset_internal();
    }
}

ewram_static!(Level: LevelManager = LevelManager::new());
