use core::fmt::Write;

use crate::{ewram_static, static_init::StaticInitSafe};

pub struct PlayerManager {}

unsafe impl StaticInitSafe for PlayerManager {
    fn init(&mut self) {
        self.reset_internal();
    }
}

ewram_static!(Player: PlayerManager = PlayerManager::new());

impl PlayerManager {
    pub const fn new() -> Self {
        PlayerManager {}
    }

    fn reset_internal(&mut self) {}

    pub fn tick() {}
}
