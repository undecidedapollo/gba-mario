use gba::{
    fixed::i32fx8,
    mmio::{BG2X, BG2Y},
};

use crate::{ewram_static, static_init::StaticInitSafe};

pub struct ScreenManager {
    pub affn_x: i32fx8,
    pub affn_y: i32fx8,
}

unsafe impl StaticInitSafe for ScreenManager {
    fn init(&mut self) {
        self.reset_internal();
    }
}

ewram_static!(Screen: ScreenManager = ScreenManager::new());

#[derive(Clone, Copy)]
pub struct ScreenInfo {
    pub affn_x: i32fx8,
    pub affn_y: i32fx8,
    pub onscreen_col_start: u16,
}

impl ScreenInfo {
    pub fn onscreen_col_end(&self) -> u16 {
        self.onscreen_col_start + 30
    }
}

impl ScreenManager {
    pub const fn new() -> Self {
        ScreenManager {
            affn_x: i32fx8::wrapping_from(0),
            affn_y: i32fx8::wrapping_from(0),
        }
    }

    pub fn on_start() {
        Screen.init();
    }

    fn reset_internal(&mut self) {}

    pub fn get_screen_info() -> ScreenInfo {
        let screen = Screen.assume_init();
        ScreenInfo {
            affn_x: screen.affn_x,
            affn_y: screen.affn_y,
            onscreen_col_start: (screen.affn_x.div(i32fx8::wrapping_from(8)).to_bits() >> 8) as u16,
        }
    }

    pub fn translate_x(amount: i32fx8) {
        let screen = Screen.assume_init();
        screen.affn_x = screen.affn_x.add(amount);
    }

    pub fn translate(x: i32fx8, y: i32fx8) {
        let screen = Screen.assume_init();
        screen.affn_x = screen.affn_x.add(x);
        screen.affn_y = screen.affn_y.add(y);
        if screen.affn_y > i32fx8::wrapping_from(10 * 10) {
            screen.affn_y = i32fx8::wrapping_from(10 * 10);
        }
    }

    pub fn post_tick() {
        let manager = Screen.assume_init();
        // manager.affn_y = manager.affn_y.add(i32fx8::from_bits(8));
        BG2X.write(manager.affn_x);
        BG2Y.write(manager.affn_y);
    }
}
