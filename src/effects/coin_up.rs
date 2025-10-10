use gba::prelude::*;

use crate::{
    assets::COIN_TILE_IDX_START,
    effects::{AnimationCtx, Effect, tile_to_screenspace},
    screen::ScreenManager,
};

pub struct CoinUpEffect {
    row: usize,
    col: usize,
    otr: ObjAttr,
}

impl CoinUpEffect {
    pub fn new(row: usize, col: usize) -> Self {
        CoinUpEffect {
            row,
            col,
            otr: ObjAttr::default(),
        }
    }

    pub fn as_effect(self) -> Effect {
        Effect::CoinUp(self)
    }

    pub fn tick(&mut self, ctx: AnimationCtx) -> bool {
        if ctx.animation_tick >= 16 {
            OBJ_ATTR_ALL.index(2).write(ObjAttr::default());
            return false;
        }

        if ctx.animation_tick == 0 {
            let mut otr = ObjAttr::new();
            otr.set_x(32);
            otr.set_y(32);
            otr.set_style(ObjDisplayStyle::Affine);
            otr.0 = otr
                .0
                .with_shape(ObjShape::Square)
                .with_mode(ObjEffectMode::Normal)
                .with_bpp8(true);
            otr.1 = otr.1.with_size(1).with_affine_index(1);
            otr.2 = otr
                .2
                .with_tile_id((COIN_TILE_IDX_START * 2) as u16)
                .with_priority(0)
                .with_palbank(0);
            self.otr = otr;
        }

        return true;
    }

    pub fn is_duplicate(&self, other: &Self) -> bool {
        self.row == other.row && self.col == other.col
    }

    pub fn post_tick(&mut self, ctx: AnimationCtx) {
        let screen = ScreenManager::get_screen_info();
        let (x, base_y) = tile_to_screenspace(self.row, self.col, &screen);

        self.otr.set_x(x.clamp(-60, 240) as u16);

        let offset = ctx.animation_tick as i32 * 2;
        self.otr
            .set_y(base_y.saturating_sub(offset).clamp(0, 256) as u16);

        OBJ_ATTR_ALL.index(2).write(self.otr);
    }
}
