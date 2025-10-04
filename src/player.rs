use core::ptr::copy_nonoverlapping;

use gba::prelude::*;

use crate::{
    assets::{MARIO_TILE, MARIO_TILE_IDX_START},
    ewram_static,
    keys::FRAME_KEYS,
    level_manager::LevelManager,
    math::mod_mask_u32,
    obj::VolAddressExt,
    screen::{ScreenInfo, ScreenManager},
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
#[derive(Debug, PartialEq, Eq)]
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

// We fall faster down then up
const VERT_DIFF_UP: i32fx8 = i32fx8::from_bits(64);
const VERT_DIFF_DOWN: i32fx8 = i32fx8::from_bits(128);

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

    fn is_moving_left(&self) -> bool {
        self.vel_x < i32fx8::wrapping_from(0)
    }

    fn is_moving_right(&self) -> bool {
        self.vel_x > i32fx8::wrapping_from(0)
    }

    fn is_horizontally_stationary(&self) -> bool {
        self.vel_x == i32fx8::wrapping_from(0)
    }

    fn is_vertically_stationary(&self) -> bool {
        self.vel_y == i32fx8::wrapping_from(0)
    }

    fn is_moving_up(&self) -> bool {
        self.vel_y < i32fx8::wrapping_from(0)
    }

    fn is_moving_down(&self) -> bool {
        self.vel_y > i32fx8::wrapping_from(0)
    }

    // 8 to remove fractional and 2 to convert to rows instead of raw pixels
    fn row(&self) -> u16 {
        (self.player_y.to_bits() >> (8 + 3)) as u16
    }

    fn col(&self) -> u16 {
        (self.player_x.to_bits() >> (8 + 3)) as u16
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
        self.next_anim_tick = 0;
        self.vel_x = i32fx8::wrapping_from(0);
        self.vel_y = i32fx8::wrapping_from(0);
        self.player_x = i32fx8::wrapping_from(32);
        self.player_y = i32fx8::wrapping_from(32);
        self.facing_dir = true;

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

    fn die_state_handler(&mut self) {
        if self.next_anim_tick == 0 {
            self.vel_y = i32fx8::from_bits(-1792);
            self.next_anim_tick = 1;
            return;
        }

        let vel_adjuster = if self.is_moving_up() {
            VERT_DIFF_UP
        } else {
            VERT_DIFF_DOWN
        };

        self.vel_y = self.vel_y.add(vel_adjuster);
        self.player_y = self.player_y.add(self.vel_y);

        if self.row() > 23 {
            self.reset_internal();
            // todo: reset level / game state of sorts
        }
    }

    fn default_movement_handler(&mut self, screen: ScreenInfo) {
        let keys = FRAME_KEYS.read();

        let is_player_sorta_to_the_right = mod_mask_u32(
            (self.player_x.to_bits() >> 8) as u32,
            crate::math::Powers::_8,
        ) >= 3;

        let row: u16 = self.row();
        let bottom_of_player = row + 2;
        let mask_above = 0b1 << row.saturating_sub(1);
        let mask_body = 0b11 << row;
        let mask_under = 0b1 << bottom_of_player;

        let left_air = LevelManager::collision_mask(self.col().saturating_sub(1));
        let left_collision = LevelManager::collision_mask(self.col());
        let right_collision = LevelManager::collision_mask(self.col() + 1);
        let right_air = LevelManager::collision_mask(self.col() + 2);

        let collision_bottom = (left_collision & mask_under != 0)
            || (right_collision & mask_under != 0)
            || (is_player_sorta_to_the_right && (right_air & mask_under != 0));

        let collision_top =
            (left_collision & mask_above != 0) || (right_collision & mask_above != 0);
        let collide_left = left_air & mask_body != 0;
        let collide_right = right_air & mask_body != 0;

        let is_new_direction_opposite_cur_dir = keys.left() && self.vel_x > i32fx8::default()
            || keys.right() && self.vel_x < i32fx8::default();

        let is_fast_enough_run = keys.b()
            && (keys.left() || keys.right())
            && !is_new_direction_opposite_cur_dir
            && self.vel_x.abs() > i32fx8::wrapping_from(1);

        if (!collision_bottom && (!self.is_moving_up())) || (!collision_top && self.is_moving_up())
        {
            let vel_adjuster = if self.is_moving_up() {
                VERT_DIFF_UP
            } else {
                VERT_DIFF_DOWN
            };

            let mut need_dec_vel_y = true;
            if keys.a() {
                let max_tick = if is_fast_enough_run { 24 } else { 18 };
                if self.next_anim_tick < max_tick {
                    self.next_anim_tick += 1;
                    if self.next_anim_tick & 0b1 == 0 {
                        need_dec_vel_y = false;
                    }
                }
            }

            if need_dec_vel_y {
                self.vel_y = self.vel_y.add(vel_adjuster);
            }
            self.player_y = self.player_y.add(self.vel_y);
        } else if collision_bottom && self.is_moving_down() {
            self.player_y = i32fx8::wrapping_from((self.row() << 3) as i32 + 1);
            self.vel_y = i32fx8::wrapping_from(0);
        } else if collision_top && self.is_moving_up() {
            self.player_y = i32fx8::wrapping_from((self.row() << 3) as i32);
            self.vel_y = i32fx8::wrapping_from(0);
        } else if self.is_vertically_stationary() {
            self.player_y = i32fx8::wrapping_from((self.row() << 3) as i32 + 1);
            self.vel_y = i32fx8::wrapping_from(0);
            if keys.a() {
                self.next_anim_tick = 0;
                self.vel_y = i32fx8::from_bits(if is_fast_enough_run { -1200 } else { -1100 });
                self.player_y = self.player_y.add(self.vel_y);
                self.set_tile(MarioAnimationTileIdx::Jumping1);
            } else if self.get_tile() == MarioAnimationTileIdx::Jumping1 {
                self.set_tile(MarioAnimationTileIdx::Standing);
                self.next_anim_tick = 0;
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

        let mut stopping_conditions: bool = false;
        if collide_left && (self.is_moving_left() || keys.left()) {
            self.vel_x = i32fx8::default();
            self.player_x = i32fx8::wrapping_from((self.col() << 3) as i32);
        } else if collide_right && (self.is_moving_right() || keys.right()) {
            self.vel_x = i32fx8::default();
            self.player_x = i32fx8::wrapping_from((self.col() << 3) as i32 + 2);
        } else if keys.left() {
            let was_above_min = self.vel_x >= -max_x_speed;
            if was_above_min {
                self.vel_x = self.vel_x.sub(x_mod_on_move);
                if self.vel_x < -max_x_speed {
                    self.vel_x = -max_x_speed;
                } else if self.is_moving_right() && self.is_vertically_stationary() {
                    // Changed direction, play stopping animation
                    self.set_tile(MarioAnimationTileIdx::Stopping);
                    stopping_conditions = true;
                }
            }
        } else if keys.right() {
            let was_below_max = self.vel_x <= max_x_speed;
            if was_below_max {
                self.vel_x = self.vel_x.add(x_mod_on_move);
                if self.vel_x > max_x_speed {
                    self.vel_x = max_x_speed;
                } else if self.is_moving_left() && self.is_vertically_stationary() {
                    // Changed direction, play stopping animation
                    self.set_tile(MarioAnimationTileIdx::Stopping);
                    stopping_conditions = true;
                }
            }
        } else {
            let adj = i32fx8::from_bits(1 << 4);
            if self.is_moving_right() {
                self.vel_x = self.vel_x.sub(adj);
            } else if self.is_moving_left() {
                self.vel_x = self.vel_x.add(adj);
            }
        }

        let max_y_speed = 1600;

        if self.vel_x > max_x_speed {
            self.vel_x -= i32fx8::from_bits(1 << 3);
        } else if self.vel_x < -max_x_speed {
            self.vel_x += i32fx8::from_bits(1 << 3);
        }

        if self.vel_y > i32fx8::from_bits(max_y_speed) {
            self.vel_y = i32fx8::from_bits(max_y_speed);
        } else if self.vel_y < i32fx8::from_bits(-max_y_speed) {
            self.vel_y = i32fx8::from_bits(-max_y_speed);
        }

        let is_walking_animation_valid_horizontally: bool =
            !self.is_horizontally_stationary() || (keys.left() || keys.right());

        if collision_bottom
            && is_walking_animation_valid_horizontally
            && self.is_vertically_stationary()
        {
            self.next_anim_tick = self.next_anim_tick.saturating_sub(1);
            if self.next_anim_tick == 0 {
                self.next_anim_tick =
                    10 - (self.vel_x.abs().mul(i32fx8::wrapping_from(3)).to_bits() >> 8) as u8;
                if self.next_anim_tick < 2 {
                    self.next_anim_tick = 2;
                }
                let new_tile = match self.get_tile() {
                    MarioAnimationTileIdx::Stopping if stopping_conditions => {
                        MarioAnimationTileIdx::Stopping
                    }
                    MarioAnimationTileIdx::Standing | MarioAnimationTileIdx::Stopping => {
                        MarioAnimationTileIdx::Walking1
                    }
                    MarioAnimationTileIdx::Walking1 => MarioAnimationTileIdx::Walking2,
                    MarioAnimationTileIdx::Walking2 => MarioAnimationTileIdx::Walking3,
                    MarioAnimationTileIdx::Walking3 => MarioAnimationTileIdx::Walking1,
                    _ => MarioAnimationTileIdx::Walking1,
                };
                self.set_tile(new_tile);
            }
            // On the ground and moving
        } else if collision_bottom
            && self.is_horizontally_stationary()
            && self.is_vertically_stationary()
        {
            self.set_tile(MarioAnimationTileIdx::Standing);
            self.next_anim_tick = 0;
        }

        // gba_warning!("Player {:?}, {:?}", self.vel_x, self.vel_y,);

        self.player_x = self.player_x.add(self.vel_x);

        if self.player_x < screen.affn_x {
            self.player_x = screen.affn_x;
            self.vel_x = i32fx8::default();
        }

        if self.vel_x.abs() < i32fx8::from_bits(1 << 4) {
            self.vel_x = i32fx8::default();
        }

        if self.row() >= 32 {
            self.set_tile(MarioAnimationTileIdx::DieState);
            self.next_anim_tick = 0;
        }

        if self.is_moving_left() {
            self.facing_dir = false;
        } else if self.is_moving_right() {
            self.facing_dir = true;
        }
    }

    pub fn tick(tick_context: TickContext) {
        let manager = Player.get_or_init();
        let screen = ScreenManager::get_screen_info();

        if manager.get_tile() == MarioAnimationTileIdx::DieState {
            manager.die_state_handler();
        } else {
            manager.default_movement_handler(screen);
        }

        let middle_screen_px = screen.affn_x.add(i32fx8::wrapping_from(10 * 8));

        let player_min_y = screen.affn_y.add(i32fx8::wrapping_from(45));
        let player_max_y = screen.affn_y.add(i32fx8::wrapping_from(120));

        // gba_warning!("velx {:?}, vely {:?}", manager.vel_x, manager.vel_y);

        let y_diff = if manager.player_y < player_min_y {
            manager.player_y.sub(player_min_y)
        } else if manager.player_y > player_max_y {
            manager.player_y.sub(player_max_y)
        } else {
            i32fx8::default()
        };

        let mut to_far = manager.player_x.sub(middle_screen_px);
        if to_far <= i32fx8::default() {
            to_far = i32fx8::default();
        }
        if to_far != i32fx8::default() || y_diff != i32fx8::default() {
            ScreenManager::translate(to_far, y_diff);
        }

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
