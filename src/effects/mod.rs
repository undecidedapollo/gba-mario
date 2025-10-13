use core::ops::Shr;

use enum_dispatch::enum_dispatch;
use gba::prelude::*;

use crate::{
    effects::{coin_up::CoinUp, points::Points, tile_bounce::TileBounce},
    ewram_static,
    fixed_bag::FixedBag,
    fixed_queue::FixedQueue,
    gba_warning,
    math::mod_mask_u32,
    screen::ScreenInfo,
    static_init::StaticInitSafe,
    tick::TickContext,
};

pub mod coin_up;
pub mod points;
pub mod tile_bounce;

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
#[enum_dispatch]
pub trait EffectImpl {
    fn tick(&mut self, ctx: AnimationCtx) -> bool;
    fn post_tick(&mut self, ctx: AnimationCtx);
}

#[enum_dispatch(EffectImpl)]
pub enum Effect {
    TileBounce,
    CoinUp,
    Points,
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
    pending_effects: FixedQueue<AnimationEffect, 6>,
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
        AFFINE_PARAM_A.index(1).write(i16fx8::from_bits(1 << 8));
        AFFINE_PARAM_B.index(1).write(i16fx8::from_bits(0));
        AFFINE_PARAM_C.index(1).write(i16fx8::from_bits(0));
        AFFINE_PARAM_D.index(1).write(i16fx8::from_bits(1 << 8));
    }

    pub fn on_start() {
        Effects.init();
    }

    pub fn tick(_tick: TickContext) {
        let manager: &mut EffectsManager = Effects.assume_init();
        for _ in 0..manager.pending_effects.len() {
            let Some(effect) = manager.pending_effects.pop() else {
                break;
            };
            if effect.tick_start > 0 {
                // gba_warning!("Delaying effect by 1 tick: {}", effect.tick_start);
                manager.pending_effects.push_pop(AnimationEffect {
                    tick_start: effect.tick_start.saturating_sub(1),
                    effect: effect.effect,
                });
                continue;
            }

            let anim_effect = AnimationEffect {
                tick_start: _tick.tick_count,
                effect: effect.effect,
            };

            if let Err(effect) = manager.active_effects.push(anim_effect) {
                // No space, put it back
                manager.pending_effects.push_pop(effect);
                break;
            }
        }

        manager
            .active_effects
            .iter_filter(|effect| effect.tick(&_tick));
    }

    pub fn post_tick(_tick: TickContext) {
        let manager: &mut EffectsManager = Effects.assume_init();
        let mut has_effects = false;
        manager.active_effects.iter_mut().for_each(|effect| {
            has_effects = true;
            effect.1.post_tick(&_tick)
        });

        if has_effects {
            let mod_tick = mod_mask_u32(_tick.tick_count, crate::math::Powers::_16);
            if mod_tick == 0 {
                AFFINE_PARAM_A.index(1).write(i16fx8::from_bits(4 << 8));
            } else if mod_tick == 2 || mod_tick == 14 {
                AFFINE_PARAM_A.index(1).write(i16fx8::from_bits(2 << 8));
            } else if mod_tick == 4 || mod_tick == 12 {
                AFFINE_PARAM_A.index(1).write(i16fx8::from_bits(1 << 8));
            } else if mod_tick == 6 || mod_tick == 10 {
                AFFINE_PARAM_A.index(1).write(-i16fx8::from_bits(2 << 8));
            } else if mod_tick == 8 {
                AFFINE_PARAM_A.index(1).write(-i16fx8::from_bits(4 << 8));
            }
        }
    }

    pub fn add_effect(effect: Effect, delay_ticks: u32) {
        let manager = Effects.assume_init();
        manager.pending_effects.push_pop(AnimationEffect {
            tick_start: delay_ticks,
            effect,
        });
    }
}

unsafe impl StaticInitSafe for EffectsManager {
    fn init(&mut self) {
        self.reset_internal();
    }
}

ewram_static!(Effects: EffectsManager = EffectsManager::new());
