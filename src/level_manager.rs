use gba::prelude::*;

use crate::{
    ewram_static,
    fixed_bag::FixedBag,
    gba_warning,
    levels::shared::{
        LEVEL_1_1, Level, LevelFloor, LevelItem, PIPE_BODY_LEFT, PIPE_BODY_RIGHT, PIPE_TOP_LEFT,
        PIPE_TOP_RIGHT, Tile,
    },
    math::{Powers, mod_mask_u32},
    screen::ScreenManager,
    static_init::StaticInitSafe,
    tick::TickContext,
};

pub struct LevelManager {
    rendered_col: u16,
    reaped_col: u16,
    level_ptr: usize,
    col_ptr: usize,
    current_level: &'static Level,
    stack_of_renders: FixedBag<ManagedItem, 8>,
}

#[derive(Clone, Copy)]
struct ManagedItem {
    item: LevelItem,
    col_start: usize,
}

impl LevelManager {
    pub const fn new() -> Self {
        LevelManager {
            rendered_col: 0,
            reaped_col: 0,
            level_ptr: 0,
            col_ptr: 0,
            current_level: &LEVEL_1_1,
            stack_of_renders: FixedBag::new(),
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

    fn draw_tile(&mut self, row: usize, col: usize, tile: Tile) {
        AFFINE2_SCREENBLOCKS
            .get_frame(16)
            .unwrap()
            .index(col, row)
            .write(
                u8x2::default()
                    .with_high(tile.top_right() as u8)
                    .with_low(tile.top_left() as u8),
            );
        AFFINE2_SCREENBLOCKS
            .get_frame(16)
            .unwrap()
            .index(col, row + 1)
            .write(
                u8x2::default()
                    .with_high(tile.bottom_right() as u8)
                    .with_low(tile.bottom_left() as u8),
            );
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

            let floor_bottom_for_col = match self.current_level.floor {
                LevelFloor::Solid { tile: _, row } => row,
            };

            while self.col_ptr <= (i as usize) && self.level_ptr < self.current_level.data.len() {
                let item = self.current_level.data[self.level_ptr];
                self.level_ptr += 1;
                match item {
                    LevelItem::NextCol { advance_by } => {
                        self.col_ptr += advance_by;
                    }
                    tile @ LevelItem::Tile {
                        tile: _,
                        len: _,
                        row: _,
                    } => {
                        self.stack_of_renders.push(ManagedItem {
                            item: tile,
                            col_start: i as usize,
                        });
                    }
                    pipe @ LevelItem::Pipe { row: _ } => {
                        self.stack_of_renders.push(ManagedItem {
                            item: pipe,
                            col_start: i as usize,
                        });
                    }
                };
            }

            for (idx, managed) in self.stack_of_renders.clone().iter() {
                match managed.item {
                    LevelItem::NextCol { .. } => {
                        // This shouldn't happen but just in case
                        self.stack_of_renders.remove(idx);
                        continue;
                    }
                    LevelItem::Pipe { row } => {
                        if i as usize == managed.col_start {
                            self.draw_tile(row, screenblock_col, PIPE_TOP_LEFT);
                        } else {
                            self.draw_tile(row, screenblock_col, PIPE_TOP_RIGHT);
                        }
                        let diff = floor_bottom_for_col.saturating_sub(row + 2) >> 1;
                        for vert_row in 0..diff {
                            let row = (row + 2) + vert_row * 2;
                            if i as usize == managed.col_start {
                                self.draw_tile(row, screenblock_col, PIPE_BODY_LEFT);
                            } else {
                                self.draw_tile(row, screenblock_col, PIPE_BODY_RIGHT);
                            }
                        }

                        if i as usize != managed.col_start {
                            self.stack_of_renders.remove(idx);
                        }
                        continue;
                    }
                    LevelItem::Tile { len, row, tile } => {
                        let col_in_item = i as usize - managed.col_start;
                        if col_in_item < len {
                            self.draw_tile(row, screenblock_col, tile);
                        } else {
                            self.stack_of_renders.remove(idx);
                        }
                    }
                }
                if let LevelItem::Tile { tile, row, len } = managed.item {
                    let col_in_item = i as usize - managed.col_start;
                    if col_in_item < len {
                        self.draw_tile(row, screenblock_col, tile);
                    } else {
                        self.stack_of_renders.remove(idx);
                    }
                }
            }

            if let LevelFloor::Solid { tile, row } = self.current_level.floor {
                self.draw_tile(row, screenblock_col, tile);
                self.draw_tile(row + 2, screenblock_col, tile);
            }

            if self.col_ptr >= self.current_level.data.len() {
                continue;
            }

            // gba_warning!("Rendering column {} actual {}", i, screenblock_col);
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
