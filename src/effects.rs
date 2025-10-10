use core::{
    intrinsics::copy_nonoverlapping,
    ops::{Add, Shl, Shr, Sub},
    u16,
};

use gba::prelude::*;

use crate::{
    assets::UNUSED_TILE_IDX_START,
    ewram_static,
    fixed_bag::FixedBag,
    fixed_queue::FixedQueue,
    gba_warning,
    level_manager::{clear_tile, draw_tile},
    levels::shared::Tile,
    screen::{ScreenInfo, ScreenManager},
    static_init::StaticInitSafe,
    tick::TickContext,
};

pub struct TileBounceEffect {
    row: usize,
    col: usize,
    tile: Tile,
    otr: ObjAttr,
}

const ONE_HALF: i32fx8 = i32fx8::wrapping_from(1).div(i32fx8::wrapping_from(2));

/// Converts tile grid coordinates (row/col) to sprite screenspace coordinates
/// Returns (x, y) coordinates for sprite positioning
///
/// TODO: Row and col work at different scales, col is half the scale of row
pub fn tile_to_screenspace(row: usize, col: usize, screen: &ScreenInfo) -> (i32, i32) {
    let difference: i32 = screen
        .affn_x
        .add(ONE_HALF)
        .sub(i32fx8::wrapping_from(screen.onscreen_col_start as i32 * 8))
        .to_bits()
        .shr(8);
    let x = (col * 2 - screen.onscreen_col_start as usize) as i32 * 8 - difference;

    let onscreen_row_start: i32 = screen.affn_y.to_bits().shr(11);
    let y_difference: i32 = screen
        .affn_y
        .sub(i32fx8::wrapping_from(onscreen_row_start * 8))
        .to_bits()
        .shr(8)
        - 1;
    let y = row as i32 * 8 - onscreen_row_start * 8 - y_difference;

    (x, y)
}

impl TileBounceEffect {
    pub fn new(row: usize, col: usize, tile: Tile) -> Self {
        TileBounceEffect {
            row,
            col,
            tile,
            otr: ObjAttr::default(),
        }
    }

    pub fn as_effect(self) -> Effect {
        Effect::TileBounce(self)
    }

    fn tick(&mut self, ctx: AnimationCtx) -> bool {
        if ctx.animation_tick >= 8 {
            draw_tile(self.row, self.col, self.tile);
            OBJ_ATTR_ALL.index(1).write(ObjAttr::default());
            return false;
        }

        if ctx.animation_tick == 0 {
            unsafe {
                copy_nonoverlapping(
                    CHARBLOCK0_8BPP.index(self.tile.top_left()).as_ptr() as *const u32,
                    OBJ_TILES.index(UNUSED_TILE_IDX_START * 2).as_usize() as *mut u32,
                    16,
                );
                copy_nonoverlapping(
                    CHARBLOCK0_8BPP.index(self.tile.top_right()).as_ptr() as *const u32,
                    OBJ_TILES.index(UNUSED_TILE_IDX_START * 2 + 2).as_usize() as *mut u32,
                    16,
                );
                copy_nonoverlapping(
                    CHARBLOCK0_8BPP.index(self.tile.bottom_left()).as_ptr() as *const u32,
                    OBJ_TILES.index(UNUSED_TILE_IDX_START * 2 + 4).as_usize() as *mut u32,
                    16,
                );
                copy_nonoverlapping(
                    CHARBLOCK0_8BPP.index(self.tile.bottom_right()).as_ptr() as *const u32,
                    OBJ_TILES.index(UNUSED_TILE_IDX_START * 2 + 6).as_usize() as *mut u32,
                    16,
                );
            }
            let mut otr = ObjAttr::new();
            otr.set_x(32);
            otr.set_y(32);
            otr.set_style(ObjDisplayStyle::Affine);
            otr.0 = otr
                .0
                .with_shape(ObjShape::Square)
                .with_mode(ObjEffectMode::Normal)
                .with_bpp8(true);
            otr.1 = otr.1.with_size(1).with_affine_index(0);
            otr.2 = otr
                .2
                .with_tile_id((UNUSED_TILE_IDX_START * 2) as u16)
                .with_priority(0)
                .with_palbank(0);
            self.otr = otr;
            clear_tile(self.row, self.col);
        }

        return true;
    }

    fn is_duplicate(&self, other: &Self) -> bool {
        self.row == other.row && self.col == other.col
    }

    fn post_tick(&mut self, ctx: AnimationCtx) {
        let screen = ScreenManager::get_screen_info();
        let (x, base_y) = tile_to_screenspace(self.row, self.col, &screen);

        self.otr.set_x(x.clamp(-60, 240) as u16);

        let offset = 4 - (ctx.animation_tick as i32 - 4).abs();
        self.otr
            .set_y(base_y.saturating_sub(offset).clamp(0, 256) as u16);

        // gba_warning!(
        //     "{} {} {} {} {} {}",
        //     self.col * 2,
        //     screen.onscreen_col_start,
        //     (self.col * 2 - screen.onscreen_col_start as usize),
        //     (self.col * 2 - screen.onscreen_col_start as usize) * 8,
        //     difference,
        //     x
        // );
        OBJ_ATTR_ALL.index(1).write(self.otr);
    }
}

pub enum Effect {
    TileBounce(TileBounceEffect),
}

impl Effect {
    pub fn tick(&mut self, ctx: AnimationCtx) -> bool {
        match self {
            Effect::TileBounce(effect) => effect.tick(ctx),
        }
    }

    pub fn post_tick(&mut self, ctx: AnimationCtx) {
        match self {
            Effect::TileBounce(effect) => effect.post_tick(ctx),
        }
    }

    pub fn is_same_type_and_position(&self, other: &Effect) -> bool {
        match (self, other) {
            (Effect::TileBounce(a), Effect::TileBounce(b)) => a.is_duplicate(b),
        }
    }
}

pub struct AnimationCtx {
    pub tick: u32,
    pub animation_tick: u32,
}

pub struct AnimationEffect {
    tick_start: u32,
    effect: Effect,
}

impl AnimationEffect {
    pub fn tick(&mut self, ctx: &TickContext) -> bool {
        let animation_tick = ctx.tick_count.saturating_sub(self.tick_start);
        let anim_ctx = AnimationCtx {
            tick: ctx.tick_count,
            animation_tick,
        };
        self.effect.tick(anim_ctx)
    }

    pub fn post_tick(&mut self, ctx: &TickContext) {
        let animation_tick = ctx.tick_count.saturating_sub(self.tick_start);
        let anim_ctx = AnimationCtx {
            tick: ctx.tick_count,
            animation_tick,
        };
        self.effect.post_tick(anim_ctx);
    }
}

pub struct EffectsManager {
    active_effects: FixedBag<AnimationEffect, 8>,
    pending_effects: FixedQueue<Effect, 4>,
}

impl EffectsManager {
    pub const fn new() -> Self {
        EffectsManager {
            active_effects: FixedBag::new(),
            pending_effects: FixedQueue::new(),
        }
    }

    fn reset_internal(&mut self) {
        for (_, opt) in self.active_effects.iter_mut_opt() {
            *opt = None;
        }
    }

    pub fn on_start() {
        Effects.init();
    }

    pub fn tick(_tick: TickContext) {
        let manager: &mut EffectsManager = Effects.assume_init();
        while let Some(effect) = manager.pending_effects.pop() {
            let anim_effect = AnimationEffect {
                tick_start: _tick.tick_count,
                effect,
            };

            let is_matching_pos = manager.active_effects.iter().any(|(_idx, active_effect)| {
                active_effect
                    .effect
                    .is_same_type_and_position(&anim_effect.effect)
            });

            if is_matching_pos {
                continue;
            }

            if let Err(effect) = manager.active_effects.push(anim_effect) {
                // No space, put it back
                manager.pending_effects.push_pop(effect.effect);
                break;
            }
        }

        manager
            .active_effects
            .iter_filter(|effect| effect.tick(&_tick));
    }

    pub fn post_tick(_tick: TickContext) {
        let manager: &mut EffectsManager = Effects.assume_init();
        manager
            .active_effects
            .iter_mut()
            .for_each(|effect| effect.1.post_tick(&_tick));
    }

    pub fn add_effect(effect: Effect) {
        let manager = Effects.assume_init();
        manager.pending_effects.push_pop(effect);
    }
}

unsafe impl StaticInitSafe for EffectsManager {
    fn init(&mut self) {
        self.reset_internal();
    }
}

ewram_static!(Effects: EffectsManager = EffectsManager::new());
