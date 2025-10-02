use core::ptr::copy_nonoverlapping;

use gba::prelude::*;

use crate::{
    assets::{MARIO_TILE, MARIO_TILE_IDX_START},
    ewram_static, gba_warning,
    keys::FRAME_KEYS,
    level::LevelManager,
    math::mod_mask_u32,
    obj::VolAddressExt,
    screen::ScreenManager,
    static_init::StaticInitSafe,
    tick::TickContext,
};

pub struct PlayerManager {
    otr: ObjAttr,
    player_x: i32fx8,
    vel_x: i32fx8,
    player_y: i32fx8,
    vel_y: i32fx8,
    next_anim_tick: u8,
    facing_dir: bool, // true is right, false is left
}

unsafe impl StaticInitSafe for PlayerManager {
    fn init(&mut self) {
        self.reset_internal();
    }
}

ewram_static!(Player: PlayerManager = PlayerManager::new());

#[repr(u16)]
#[allow(unused)]
#[derive(Debug)]
enum MarioAnimationTileIdx {
    Standing = 0,
    Walking1 = 1 * 4,
    Walking2 = 2 * 4,
    Walking3 = 3 * 4,
    Stopping = 4 * 4,
    Jumping1 = 5 * 4,
    DieState = 6 * 4,
    SlidePole = 7 * 4,
}

impl PlayerManager {
    pub const fn new() -> Self {
        PlayerManager {
            otr: ObjAttr::new(),
            next_anim_tick: 0,
            facing_dir: true,
            player_x: i32fx8::wrapping_from(32),
            vel_x: i32fx8::wrapping_from(0),
            player_y: i32fx8::wrapping_from(32),
            vel_y: i32fx8::wrapping_from(0),
        }
    }

    fn set_tile(&mut self, tile: MarioAnimationTileIdx) {
        self.otr
            .set_tile_id((MARIO_TILE_IDX_START as u16 + (tile as u16)) * 2);
    }

    fn get_tile(&self) -> MarioAnimationTileIdx {
        let idx = (self.otr.2.tile_id() / 2) - (MARIO_TILE_IDX_START as u16);
        match idx {
            0 => MarioAnimationTileIdx::Standing,
            4 => MarioAnimationTileIdx::Walking1,
            8 => MarioAnimationTileIdx::Walking2,
            12 => MarioAnimationTileIdx::Walking3,
            16 => MarioAnimationTileIdx::Stopping,
            20 => MarioAnimationTileIdx::Jumping1,
            24 => MarioAnimationTileIdx::DieState,
            28 => MarioAnimationTileIdx::SlidePole,
            _ => MarioAnimationTileIdx::Standing, // Default case, should not happen
        }
    }

    fn update_face_dir(&mut self) {
        AFFINE_PARAM_A.index(0).write(if self.facing_dir {
            i16fx8::from_bits(1 << 8)
        } else {
            -i16fx8::from_bits(1 << 8)
        });
    }

    fn reset_internal(&mut self) {
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
        OBJ_ATTR_ALL.index(0).write(otr);

        AFFINE_PARAM_A.index(0).write(i16fx8::from_bits(1 << 8));
        AFFINE_PARAM_B.index(0).write(i16fx8::from_bits(0));
        AFFINE_PARAM_C.index(0).write(i16fx8::from_bits(0));
        AFFINE_PARAM_D.index(0).write(i16fx8::from_bits(1 << 8));
        self.otr = otr;
        self.set_tile(MarioAnimationTileIdx::Standing);

        unsafe {
            copy_nonoverlapping(
                MARIO_TILE.0.as_ptr(),
                OBJ_TILES.index(MARIO_TILE_IDX_START * 2).as_usize() as *mut u8,
                MARIO_TILE.0.len(),
            );
        }
    }

    pub fn init() {
        Player.init();
    }

    pub fn on_start() {
        let _player = Player.get_or_init();
    }

    pub fn tick(tick_context: TickContext) {
        let manager = Player.get_or_init();
        let keys = FRAME_KEYS.read();
        let screen = ScreenManager::get_screen_info();

        let row: u16 = (manager.player_y.to_bits() >> (8 + 3)) as u16; // 8 to remove fractional and 3 to convert to rows instead of raw pixels
        let col: u16 = (manager.player_x.to_bits() >> (8 + 3)) as u16;

        // gba_warning!(
        //     "Player at row {}, col {}, plr {}",
        //     row,
        //     col,
        //     manager.player_x.to_bits() >> 11
        // );
        let standable = LevelManager::is_standable(row + 2, col >> 1);
        let is_new_direction_opposite_cur_dir = keys.left() && manager.vel_x > i32fx8::default()
            || keys.right() && manager.vel_x < i32fx8::default();
        if !standable {
            let vel_adjuster = if manager.vel_y < i32fx8::wrapping_from(0) {
                i32fx8::from_bits(1 << 5)
            } else {
                i32fx8::from_bits(1 << 6)
            };

            manager.vel_y = manager.vel_y.add(vel_adjuster);
            manager.player_y = manager.player_y.add(manager.vel_y);
        } else if manager.vel_y > i32fx8::wrapping_from(0) {
            manager.player_y = i32fx8::wrapping_from((row << 3) as i32 + 1);
            manager.vel_y = i32fx8::wrapping_from(0);
        } else if manager.vel_y < i32fx8::wrapping_from(0) {
            // On the ground and needs vertical acceleration applied, not needed for anything at the moment
            manager.player_y = manager.player_y.add(manager.vel_y);
        } else if manager.vel_y == i32fx8::wrapping_from(0) {
            manager.player_y = i32fx8::wrapping_from((row << 3) as i32 + 1);
            manager.vel_y = i32fx8::wrapping_from(0);
            if keys.a() {
                manager.vel_y = if keys.b()
                    && (keys.left() || keys.right())
                    && !is_new_direction_opposite_cur_dir
                    && manager.vel_x.abs() > i32fx8::wrapping_from(1)
                {
                    i32fx8::from_bits((-1 << 10))
                } else {
                    i32fx8::from_bits((-1 << 9) - 384)
                };
                manager.player_y = manager.player_y.add(manager.vel_y);
                manager.set_tile(MarioAnimationTileIdx::Jumping1);
            }
        }

        let x_mod_on_move = if is_new_direction_opposite_cur_dir {
            i32fx8::from_bits(1 << 5)
        } else {
            i32fx8::from_bits(1 << 4)
        };

        let max_x_speed = if keys.b() {
            i32fx8::from_bits((1 << 9) + (1 << 8))
        } else {
            i32fx8::from_bits(1 << 9)
        };

        if keys.left() {
            let was_above_min = manager.vel_x >= -max_x_speed;
            if was_above_min {
                manager.vel_x = manager.vel_x.sub(x_mod_on_move);
                if manager.vel_x < -max_x_speed {
                    manager.vel_x = -max_x_speed;
                } else if manager.vel_x > i32fx8::default()
                    && manager.vel_y == i32fx8::wrapping_from(0)
                {
                    // Changed direction, play stopping animation
                    manager.set_tile(MarioAnimationTileIdx::Stopping);
                }
            }
        } else if keys.right() {
            let was_below_max = manager.vel_x <= max_x_speed;
            if was_below_max {
                manager.vel_x = manager.vel_x.add(x_mod_on_move);
                if manager.vel_x > max_x_speed {
                    manager.vel_x = max_x_speed;
                } else if manager.vel_x < i32fx8::default()
                    && manager.vel_y == i32fx8::wrapping_from(0)
                {
                    // Changed direction, play stopping animation
                    manager.set_tile(MarioAnimationTileIdx::Stopping);
                }
            }
        } else {
            let adj = i32fx8::from_bits(1 << 4);
            if manager.vel_x > i32fx8::wrapping_from(0) {
                manager.vel_x = manager.vel_x.sub(adj);
            } else if manager.vel_x < i32fx8::wrapping_from(0) {
                manager.vel_x = manager.vel_x.add(adj);
            }
        }

        let max_y_speed = 1 << 10;

        if manager.vel_x > max_x_speed {
            manager.vel_x -= i32fx8::from_bits(1 << 3);
        } else if manager.vel_x < -max_x_speed {
            manager.vel_x += i32fx8::from_bits(1 << 3);
        }

        if manager.vel_y > i32fx8::from_bits(max_y_speed) {
            manager.vel_y = i32fx8::from_bits(max_y_speed);
        } else if manager.vel_y < i32fx8::from_bits(-max_y_speed) {
            manager.vel_y = i32fx8::from_bits(-max_y_speed);
        }

        if standable
            && manager.vel_y == i32fx8::wrapping_from(0)
            && manager.vel_x != i32fx8::wrapping_from(0)
        {
            manager.next_anim_tick = manager.next_anim_tick.saturating_sub(1);
            if manager.next_anim_tick == 0 {
                manager.next_anim_tick =
                    10 - (manager.vel_x.abs().mul(i32fx8::wrapping_from(3)).to_bits() >> 8) as u8;
                if manager.next_anim_tick < 2 {
                    manager.next_anim_tick = 2;
                }
                let new_tile = match manager.get_tile() {
                    MarioAnimationTileIdx::Standing | MarioAnimationTileIdx::Stopping => {
                        MarioAnimationTileIdx::Walking1
                    }
                    MarioAnimationTileIdx::Walking1 => MarioAnimationTileIdx::Walking2,
                    MarioAnimationTileIdx::Walking2 => MarioAnimationTileIdx::Walking3,
                    MarioAnimationTileIdx::Walking3 => MarioAnimationTileIdx::Walking1,
                    _ => MarioAnimationTileIdx::Walking1,
                };
                manager.set_tile(new_tile);
            }
            // On the ground and moving
        } else if standable
            && manager.vel_y == i32fx8::wrapping_from(0)
            && manager.vel_x == i32fx8::wrapping_from(0)
        {
            manager.set_tile(MarioAnimationTileIdx::Standing);
            manager.next_anim_tick = 0;
        }

        // gba_warning!("Player {:?}, {:?}", manager.vel_x, manager.vel_y,);

        manager.player_x = manager.player_x.add(manager.vel_x);

        if manager.player_x < screen.affn_x {
            manager.player_x = screen.affn_x;
            manager.vel_x = i32fx8::default();
        }

        if manager.vel_x.abs() < i32fx8::from_bits(1 << 4) {
            manager.vel_x = i32fx8::default();
        }

        if manager.vel_x < i32fx8::default() {
            manager.facing_dir = false;
        } else if manager.vel_x > i32fx8::default() {
            manager.facing_dir = true;
        }

        let player_min_y = screen.affn_y.add(i32fx8::wrapping_from(45));
        let player_max_y = screen.affn_y.add(i32fx8::wrapping_from(120));

        gba_warning!("velx {:?}, vely {:?}", manager.vel_x, manager.vel_y);

        let y_diff = if manager.player_y < player_min_y {
            manager.player_y.sub(player_min_y)
        } else if manager.player_y > player_max_y {
            manager.player_y.sub(player_max_y)
        } else {
            i32fx8::default()
        };

        let middle_screen_px = screen.affn_x.add(i32fx8::wrapping_from(10 * 8));

        let mut to_far = manager.player_x.sub(middle_screen_px);
        if to_far <= i32fx8::default() {
            to_far = i32fx8::default();
        }
        if to_far != i32fx8::default() || y_diff != i32fx8::default() {
            ScreenManager::translate(to_far, y_diff);
        }

        // gba_warning!(
        //     "Player row {}, {}, {}",
        //     row,
        //     standable,
        //     (manager.player_x.to_bits() >> 8) as u16
        // );
        // manager.otr.set_y((manager.p.to_bits() >> 8) as u16);
        manager
            .otr
            .set_x((manager.player_x.sub(screen.affn_x).to_bits() >> 8) as u16);
        manager
            .otr
            .set_y((manager.player_y.sub(screen.affn_y).to_bits() >> 8) as u16);
        manager.update_face_dir();
        OBJ_ATTR_ALL.index(0).write_consecutive(&[manager.otr]);
        // manager.otr.write(OBJ_ATTR_ALL.index(0));
    }
}
