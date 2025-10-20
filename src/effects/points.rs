use gba::prelude::*;

use crate::{
    assets::POINT_TILE_IDX_START,
    effects::{AnimationCtx, Effect, EffectImpl, tile_to_screenspace},
    screen::ScreenManager,
};

#[repr(u16)]
#[allow(unused)]
#[derive(Debug, PartialEq, Eq)]
enum PointsAnimationTileIdx {
    Ten = 0,
    Twenty = 1,
    Fourty = 2,
    Fifty = 3,
    Eighty = 4,
    TrailingZero = 5,
    OneUpLeft = 6,
    OneUpRight = 7,
}

impl PointsAnimationTileIdx {
    fn tile_id(self) -> u16 {
        let self_id = self as u16;
        (POINT_TILE_IDX_START as u16 * 2 + self_id * 2) as u16
    }
}

pub enum ScoreAmount {
    OneHundred,
    TwoHundred,
    FourHundred,
    FiveHundred,
    EightHundred,
    OneUp,
}

pub struct Points {
    row: usize,
    col: usize,
    amount: ScoreAmount,
    otr_left: ObjAttr,
    otr_right: ObjAttr,
}

impl Points {
    pub fn new(row: usize, col: usize, amount: ScoreAmount) -> Self {
        Points {
            row,
            col,
            amount,
            otr_left: ObjAttr::default(),
            otr_right: ObjAttr::default(),
        }
    }

    pub fn as_effect(self) -> Effect {
        Effect::Points(self)
    }

    fn get_tiles(&self) -> (PointsAnimationTileIdx, PointsAnimationTileIdx) {
        match self.amount {
            ScoreAmount::OneHundred => (
                PointsAnimationTileIdx::Ten,
                PointsAnimationTileIdx::TrailingZero,
            ),
            ScoreAmount::TwoHundred => (
                PointsAnimationTileIdx::Twenty,
                PointsAnimationTileIdx::TrailingZero,
            ),
            ScoreAmount::FourHundred => (
                PointsAnimationTileIdx::Fourty,
                PointsAnimationTileIdx::TrailingZero,
            ),
            ScoreAmount::FiveHundred => (
                PointsAnimationTileIdx::Fifty,
                PointsAnimationTileIdx::TrailingZero,
            ),
            ScoreAmount::EightHundred => (
                PointsAnimationTileIdx::Eighty,
                PointsAnimationTileIdx::TrailingZero,
            ),
            ScoreAmount::OneUp => (
                PointsAnimationTileIdx::OneUpLeft,
                PointsAnimationTileIdx::OneUpRight,
            ),
        }
    }
}

impl EffectImpl for Points {
    fn tick(&mut self, ctx: AnimationCtx) -> bool {
        if ctx.animation_tick >= 16 {
            OBJ_ATTR_ALL.index(3).write(ObjAttr::default());
            OBJ_ATTR_ALL.index(4).write(ObjAttr::default());
            return false;
        }

        if ctx.animation_tick == 0 {
            let (left_tile, right_tile) = self.get_tiles();
            let mut otr_left = ObjAttr::new();
            otr_left.set_x(32);
            otr_left.set_y(32);
            otr_left.set_style(ObjDisplayStyle::Normal);
            otr_left.0 = otr_left
                .0
                .with_shape(ObjShape::Square)
                .with_mode(ObjEffectMode::Normal)
                .with_bpp8(true);
            otr_left.1 = otr_left.1.with_size(0);
            otr_left.2 = otr_left
                .2
                .with_tile_id(left_tile.tile_id())
                .with_priority(0)
                .with_palbank(0);

            let mut otr_right = otr_left.clone();
            otr_right.2 = otr_right.2.with_tile_id(right_tile.tile_id());

            self.otr_left = otr_left;
            self.otr_right = otr_right;
        }

        return true;
    }

    fn post_tick(&mut self, ctx: AnimationCtx) {
        let screen = ScreenManager::get_screen_info();
        let (x, base_y) = tile_to_screenspace(self.row, self.col, &screen);

        let offset = ctx.animation_tick as i32 * 2;
        let y = base_y.saturating_sub(offset).clamp(0, 256) as u16;

        self.otr_left.set_x(x.clamp(-60, 240) as u16);
        self.otr_left.set_y(y);
        self.otr_right
            .set_x(x.saturating_add(8).clamp(-60, 240) as u16);
        self.otr_right.set_y(y);

        OBJ_ATTR_ALL.index(3).write(self.otr_left);
        OBJ_ATTR_ALL.index(4).write(self.otr_right);
    }
}
