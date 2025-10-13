use gba::prelude::*;

use crate::{
    assets::{BRICK_IDX_START, USED_BLOCK_IDX_START},
    effects::{AnimationCtx, Effect, EffectImpl, tile_to_screenspace},
    level_manager::{clear_tile, draw_tile},
    levels::shared::{BRICK, QUESTION_BLOCK_USED, Tile},
    screen::ScreenManager,
};

pub enum BounceEffectTile {
    Brick,
    UsedBlock,
}

impl BounceEffectTile {
    fn obj_tile_id(&self) -> u16 {
        (match self {
            BounceEffectTile::Brick => BRICK_IDX_START * 2,
            BounceEffectTile::UsedBlock => USED_BLOCK_IDX_START * 2,
        }) as u16
    }

    fn tile(&self) -> Tile {
        match self {
            BounceEffectTile::Brick => BRICK,
            BounceEffectTile::UsedBlock => QUESTION_BLOCK_USED,
        }
    }
}

pub struct TileBounce {
    row: usize,
    col: usize,
    tile: BounceEffectTile,
    otr: ObjAttr,
}

impl TileBounce {
    pub fn new(row: usize, col: usize, tile: BounceEffectTile) -> Self {
        TileBounce {
            row,
            col,
            tile,
            otr: ObjAttr::default(),
        }
    }

    pub fn as_effect(self) -> Effect {
        Effect::TileBounce(self)
    }
}

impl EffectImpl for TileBounce {
    fn tick(&mut self, ctx: AnimationCtx) -> bool {
        if ctx.animation_tick >= 8 {
            draw_tile(self.row, self.col, self.tile.tile());
            OBJ_ATTR_ALL.index(1).write(ObjAttr::default());
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
            otr.1 = otr.1.with_size(1).with_affine_index(0);
            otr.2 = otr
                .2
                .with_tile_id(self.tile.obj_tile_id())
                .with_priority(0)
                .with_palbank(0);
            self.otr = otr;
            clear_tile(self.row, self.col);
        }

        return true;
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
