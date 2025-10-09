use core::ops::Add;

use gba::prelude::*;

use crate::{
    ewram_static,
    fixed_bag::FixedBag,
    fixed_queue::FixedQueue,
    gba_warning,
    levels::shared::{
        BUSH_LEFT, BUSH_MIDDLE, BUSH_RIGHT, LEVEL_1_1, Level, LevelFloor, LevelItem,
        MOUNTAIL_BUTTONS, MOUNTAIL_EMPTY, MOUNTAIL_SLOPE_DOWN, MOUNTAIL_SLOPE_UP, MOUNTAIL_TOP,
        PIPE_BODY_LEFT, PIPE_BODY_RIGHT, PIPE_TOP_LEFT, PIPE_TOP_RIGHT, Tile,
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
    stand_matrix: FixedQueue<u32, 32>,
    queue_start: usize,
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
            stand_matrix: FixedQueue::new(),
            queue_start: 0,
        }
    }

    fn reset_internal(&mut self) {}

    pub fn on_start() {
        Level.init();
    }

    pub fn collision_mask(mut col: u16) -> u32 {
        let screen_details = ScreenManager::get_screen_info();
        let end = screen_details.onscreen_col_end();
        if col < screen_details.onscreen_col_start {
            col = screen_details.onscreen_col_start;
        } else if col > end {
            col = end;
        }

        let manager = Level.get_or_init();
        let diff_to_add = screen_details.onscreen_col_start - manager.queue_start as u16;
        let col_idx = col
            .saturating_sub(screen_details.onscreen_col_start)
            .add(diff_to_add);

        if let Some(mask) = manager.stand_matrix.get(col_idx as usize) {
            return *mask;
        }

        return 0;
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
        let render_end = end + 2;
        let reap = (start as u16).saturating_sub(8);
        // gba_warning!(
        //     "Start {}, end {}, render_end {}, reap {}",
        //     start,
        //     end,
        //     render_end,
        //     reap
        // );

        // Don't handle column operations on odd frames
        if self.rendered_col >= render_end || mod_mask_u32(start as u32, Powers::_2) != 0 {
            return;
        }

        for mut i in (self.rendered_col..render_end).step_by(2) {
            i = i >> 1;
            let mut standable_mask: u32 = 0;
            let screenblock_col: usize = mod_mask_u32(i as u32, Powers::_32) as usize;

            gba_warning!("Rendering column {} actual {}", i, screenblock_col);

            let floor_bottom_for_col = match self.current_level.floor {
                LevelFloor::Solid { tile: _, row } => row << 1,
            };

            let background_col = i % 48;

            let floor_bg = floor_bottom_for_col >> 1;

            let from_floor = |up_from_floor: usize| -> usize {
                (floor_bg - 1).saturating_sub(up_from_floor) << 1
            };

            let mut should_floor_be_visible = true;

            match background_col {
                0 => {
                    self.draw_tile(from_floor(0), screenblock_col, MOUNTAIL_SLOPE_UP);
                }
                1 => {
                    self.draw_tile(from_floor(1), screenblock_col, MOUNTAIL_SLOPE_UP);
                    self.draw_tile(from_floor(0), screenblock_col, MOUNTAIL_BUTTONS);
                }
                2 => {
                    self.draw_tile(from_floor(2), screenblock_col, MOUNTAIL_TOP);
                    self.draw_tile(from_floor(1), screenblock_col, MOUNTAIL_BUTTONS);
                    self.draw_tile(from_floor(0), screenblock_col, MOUNTAIL_EMPTY);
                }
                3 => {
                    self.draw_tile(from_floor(1), screenblock_col, MOUNTAIL_SLOPE_DOWN);
                    self.draw_tile(from_floor(0), screenblock_col, MOUNTAIL_BUTTONS);
                }
                4 => {
                    self.draw_tile(from_floor(0), screenblock_col, MOUNTAIL_SLOPE_DOWN);
                }
                11 => {
                    self.draw_tile(from_floor(0), screenblock_col, BUSH_LEFT);
                }
                12..=14 => {
                    self.draw_tile(from_floor(0), screenblock_col, BUSH_MIDDLE);
                }
                15 => {
                    self.draw_tile(from_floor(0), screenblock_col, BUSH_RIGHT);
                }
                16 => {
                    self.draw_tile(from_floor(0), screenblock_col, MOUNTAIL_SLOPE_UP);
                }
                17 => {
                    self.draw_tile(from_floor(1), screenblock_col, MOUNTAIL_TOP);
                    self.draw_tile(from_floor(0), screenblock_col, MOUNTAIL_BUTTONS);
                }
                18 => {
                    self.draw_tile(from_floor(0), screenblock_col, MOUNTAIL_SLOPE_DOWN);
                }
                23 => {
                    self.draw_tile(from_floor(0), screenblock_col, BUSH_LEFT);
                }
                24 => {
                    self.draw_tile(from_floor(0), screenblock_col, BUSH_MIDDLE);
                }
                25 => {
                    self.draw_tile(from_floor(0), screenblock_col, BUSH_RIGHT);
                }
                41 => {
                    self.draw_tile(from_floor(0), screenblock_col, BUSH_LEFT);
                }
                42 | 43 => {
                    self.draw_tile(from_floor(0), screenblock_col, BUSH_MIDDLE);
                }
                44 => {
                    self.draw_tile(from_floor(0), screenblock_col, BUSH_RIGHT);
                }
                _ => {}
            };

            while self.col_ptr <= (i as usize) && self.level_ptr < self.current_level.data.len() {
                let item: LevelItem = self.current_level.data[self.level_ptr];
                self.level_ptr += 1;
                match item {
                    LevelItem::NextCol { advance_by } => {
                        self.col_ptr += advance_by;
                    }
                    LevelItem::Tile { .. }
                    | LevelItem::Pipe { .. }
                    | LevelItem::HoleInFloor { .. } => {
                        self.stack_of_renders.push(ManagedItem {
                            item,
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
                        let row = row << 1;
                        standable_mask |= 0b11 << row;
                        if i as usize == managed.col_start {
                            self.draw_tile(row, screenblock_col, PIPE_TOP_LEFT);
                        } else {
                            self.draw_tile(row, screenblock_col, PIPE_TOP_RIGHT);
                        }
                        let diff = floor_bottom_for_col.saturating_sub(row + 2) >> 1;
                        for vert_row in 0..diff {
                            let row = (row + 2) + vert_row * 2;
                            standable_mask |= 0b11 << row;
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
                        let row = row << 1;
                        let col_in_item = i as usize - managed.col_start;
                        if col_in_item < len {
                            standable_mask |= 0b11 << row;
                            self.draw_tile(row, screenblock_col, tile);
                        } else {
                            self.stack_of_renders.remove(idx);
                        }
                    }
                    LevelItem::HoleInFloor { len } => {
                        let col_in_item = i as usize - managed.col_start;

                        if col_in_item >= len {
                            self.stack_of_renders.remove(idx);
                        } else {
                            should_floor_be_visible = false;
                        }
                    }
                }
            }

            if should_floor_be_visible
                && let LevelFloor::Solid { tile, row } = self.current_level.floor
            {
                let row = row << 1;
                standable_mask |= 0b1111 << row;
                self.draw_tile(row, screenblock_col, tile);
                self.draw_tile(row + 2, screenblock_col, tile);
            }

            // gba_warning!("Standable mask for col: {:032b}", standable_mask);
            // Push twice to account for 2-column tiles
            if self.stand_matrix.push_pop(standable_mask).is_some() {
                self.queue_start += 1;
            }
            if self.stand_matrix.push_pop(standable_mask).is_some() {
                self.queue_start += 1;
            }

            if self.col_ptr >= self.current_level.data.len() {
                continue;
            }
        }
        self.rendered_col = render_end;
        for mut i in self.reaped_col..reap {
            i = i >> 1;
            let screenblock_col = mod_mask_u32(i as u32, Powers::_32) as usize;
            // gba_warning!("Reaping column {} actual {}", i, screenblock_col);
            for i in 0..=34 {
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
