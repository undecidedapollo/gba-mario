use core::fmt::Write;

use crate::{
    color::PaletteColor,
    ewram_static,
    ewramstring::EwramString,
    screen_text::{ScreenTextManager, WriteTicket},
    static_init::StaticInitSafe,
};

pub struct ScoreManager {
    pub score: u32,
    new_score: Option<u32>,
    score_handle: Option<WriteTicket>,
}

unsafe impl StaticInitSafe for ScoreManager {
    fn init(&mut self) {
        self.reset_internal(0);
    }
}

ewram_static!(HIGHSCORE_STR: EwramString<10> = EwramString::new());
ewram_static!(Score: ScoreManager = ScoreManager::new());

impl ScoreManager {
    pub const fn new() -> Self {
        ScoreManager {
            score: 0,
            new_score: Some(0),
            score_handle: None,
        }
    }

    fn reset_internal(&mut self, score: u32) {
        self.score = score;
        self.write_score();

        let score_str = "Score:";
        ScreenTextManager::write_text(
            2,
            score_str,
            (30 - 6 - score_str.len() - 1, 0),
            PaletteColor::White,
            false,
        )
        .map(|x| x.forever());
    }

    fn write_score(&mut self) {
        self.score_handle.take();
        let str = HIGHSCORE_STR.get_or_init();
        str.clear();
        write!(str, "{:0>6}", self.score).unwrap();
        let start_idx = 30 - str.len;
        self.score_handle = ScreenTextManager::write_text(
            2,
            str.as_str(),
            (start_idx, 0),
            PaletteColor::Cyan,
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

    pub fn tick() {
        let manager = Score.get_or_init();
        let Some(mut score) = manager.new_score.take() else {
            return;
        };

        if score >= 1_000_000 {
            score = 999_999;
        }

        manager.score = score;
        manager.write_score();
    }
}
