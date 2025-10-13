use gba::prelude::*;

use crate::{
    assets::COIN_TILE_IDX_START,
    effects::{AnimationCtx, Effect, EffectImpl, tile_to_screenspace},
    screen::ScreenManager,
};

pub struct CoinUp {
    row: usize,
    col: usize,
    otr: ObjAttr,
}

impl CoinUp {
    pub fn new(row: usize, col: usize) -> Self {
        CoinUp {
            row,
            col,
            otr: ObjAttr::default(),
        }
    }

    pub fn as_effect(self) -> Effect {
        Effect::CoinUp(self)
    }
}

impl EffectImpl for CoinUp {
    fn tick(&mut self, ctx: AnimationCtx) -> bool {
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

    fn post_tick(&mut self, ctx: AnimationCtx) {
        let screen = ScreenManager::get_screen_info();
        let (x, base_y) = tile_to_screenspace(self.row, self.col, &screen);

        self.otr.set_x(x.clamp(-60, 240) as u16);

        let offset = ctx.animation_tick as i32 * 2;
        self.otr
            .set_y(base_y.saturating_sub(offset).clamp(0, 256) as u16);

        OBJ_ATTR_ALL.index(2).write(self.otr);
    }
}
