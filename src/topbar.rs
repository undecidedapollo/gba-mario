use core::fmt::Write;

use crate::{
    assets::TEXT_SCREENBLOCK_START,
    color::PaletteColor,
    ewram_static,
    ewramstring::EwramString,
    math::mod_mask_u32,
    screen_text::{ScreenTextManager, WriteTicket},
    static_init::StaticInitSafe,
    tick::TickContext,
};

pub struct TopBarManager {
    pub score: u32,
    pub time: u16,
    pub time_tick: u8,
    new_score: Option<u32>,
    score_handle: Option<WriteTicket>,
    coin_handle: Option<WriteTicket>,
    world_handle: Option<WriteTicket>,
    time_handle: Option<WriteTicket>,
}

unsafe impl StaticInitSafe for TopBarManager {
    fn init(&mut self) {
        self.reset_internal(0);
    }
}

ewram_static!(HIGHSCORE_STR: EwramString<5> = EwramString::new());
ewram_static!(WORLD_STR: EwramString<3> = EwramString::new());
ewram_static!(TIME_STR: EwramString<3> = EwramString::new());
ewram_static!(Score: TopBarManager = TopBarManager::new());

const TIME_LOC: (usize, usize) = (26, 0);

impl TopBarManager {
    pub const fn new() -> Self {
        TopBarManager {
            score: 0,
            time: 400,
            time_tick: 0,
            new_score: Some(0),
            score_handle: None,
            coin_handle: None,
            world_handle: None,
            time_handle: None,
        }
    }

    fn reset_internal(&mut self, score: u32) {
        self.time = 400;
        self.time_tick = 0;
        self.new_score = None;
        self.score = score;
        self.write_score();
        self.write_time();

        ScreenTextManager::write_text(
            TEXT_SCREENBLOCK_START,
            "Cx00",
            (10, 0),
            PaletteColor::White,
            false,
        )
        .map(|x| x.forever());

        ScreenTextManager::write_text(
            TEXT_SCREENBLOCK_START,
            "1-1",
            (18, 0),
            PaletteColor::White,
            false,
        )
        .map(|x| x.forever());
    }

    fn write_score(&mut self) {
        self.score_handle.take();
        let str = HIGHSCORE_STR.get_or_init();
        str.clear();
        write!(str, "{:0>5}", self.score).unwrap();
        let start_idx = 30 - str.len;
        self.score_handle = ScreenTextManager::write_text(
            TEXT_SCREENBLOCK_START,
            str.as_str(),
            (1, 0),
            PaletteColor::White,
            false,
        );
    }

    fn write_time(&mut self) {
        self.time_handle.take();
        let str = TIME_STR.get_or_init();
        str.clear();
        write!(str, "{:0>3}", self.time).unwrap();
        self.time_handle = ScreenTextManager::write_text(
            TEXT_SCREENBLOCK_START,
            str.as_str(),
            TIME_LOC,
            PaletteColor::White,
            false,
        );
    }

    pub fn reset(score: u32) {
        Score.get_or_init().reset_internal(score);
    }

    pub fn reset_w_score() {
        let manager = Score.get_or_init();
        manager.reset_internal(manager.score);
    }

    pub fn update_score(score: u32) {
        let manager = Score.get_or_init();
        manager.new_score = Some(score);
    }

    pub fn add_to_score(score: u32) {
        let manager = Score.get_or_init();
        let mut cur_score = manager.score + score;
        if let Some(ok) = manager.new_score {
            cur_score += ok;
        }
        manager.new_score = Some(cur_score);
    }

    pub fn tick(_tick_context: TickContext) {
        let manager = Score.get_or_init();

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
