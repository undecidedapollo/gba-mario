use core::fmt::Write;

use crate::{
    assets::TEXT_SCREENBLOCK_START,
    color::PaletteColor,
    ewram_static,
    fixed_string::FixedString,
    screen_text::{ScreenTextManager, TextPalette},
    static_init::StaticInitSafe,
    tick::TickContext,
};

pub struct TopBarManager {
    pub score: u32,
    pub time: u16,
    pub time_tick: u8,
    new_score: Option<u32>,
    palette_handle: Option<TextPalette<4>>,
}

unsafe impl StaticInitSafe for TopBarManager {
    fn init(&mut self) {
        self.reset_internal(0);
    }
}

ewram_static!(HIGHSCORE_STR: FixedString<5> = FixedString::new());
ewram_static!(WORLD_STR: FixedString<3> = FixedString::new());
ewram_static!(TIME_STR: FixedString<3> = FixedString::new());
ewram_static!(TopBar: TopBarManager = TopBarManager::new());

const TIME_LOC: (usize, usize) = (26, 0);

impl TopBarManager {
    pub const fn new() -> Self {
        TopBarManager {
            score: 0,
            time: 400,
            time_tick: 0,
            new_score: Some(0),
            palette_handle: None,
        }
    }

    fn reset_internal(&mut self, score: u32) {
        self.time = 400;
        self.time_tick = 0;
        self.new_score = None;
        self.score = score;
        self.palette_handle = Some(ScreenTextManager::create_palette(
            "0123456789-Cx ",
            PaletteColor::White,
        ));
        self.write_score();
        self.write_time();

        self.palette_handle.as_mut().unwrap().write_text(
            0,
            TEXT_SCREENBLOCK_START,
            "Cx00",
            (10, 0),
            false,
        );
        self.palette_handle.as_mut().unwrap().write_text(
            1,
            TEXT_SCREENBLOCK_START,
            "1-1",
            (18, 0),
            false,
        );
    }

    fn write_score(&mut self) {
        let str = HIGHSCORE_STR.get_or_init();
        str.clear();
        write!(str, "{:0>5}", self.score).unwrap();
        self.palette_handle.as_mut().unwrap().write_text(
            2,
            TEXT_SCREENBLOCK_START,
            str.as_str(),
            (1, 0),
            false,
        );
    }

    fn write_time(&mut self) {
        let str = TIME_STR.get_or_init();
        str.clear();
        write!(str, "{:0>3}", self.time).unwrap();
        self.palette_handle.as_mut().unwrap().write_text(
            3,
            TEXT_SCREENBLOCK_START,
            str.as_str(),
            TIME_LOC,
            false,
        );
    }

    pub fn reset(score: u32) {
        TopBar.get_or_init().reset_internal(score);
    }

    pub fn reset_w_score() {
        let manager = TopBar.get_or_init();
        manager.reset_internal(manager.score);
    }

    pub fn update_score(score: u32) {
        let manager = TopBar.get_or_init();
        manager.new_score = Some(score);
    }

    pub fn add_to_score(score: u32) {
        let manager = TopBar.get_or_init();
        let mut cur_score = manager.score + score;
        if let Some(ok) = manager.new_score {
            cur_score += ok;
        }
        manager.new_score = Some(cur_score);
    }

    pub fn tick(_tick_context: TickContext) {
        let manager = TopBar.get_or_init();

        if manager.time > 0 && manager.time_tick >= 22 {
            manager.time -= 1;
            manager.time_tick = 0;
            manager.write_time();
        } else {
            manager.time_tick = manager.time_tick.wrapping_add(1);
        }

        let Some(mut score) = manager.new_score.take() else {
            return;
        };

        if score >= 1_000_000 {
            score = 0;
        }

        manager.score = score;
        manager.write_score();
    }
}
