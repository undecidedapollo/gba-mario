use gba::prelude::*;

use crate::{ewram_static, static_init::StaticInitSafe};

pub struct KeysManager {
    frame_keys: KeyInput,
    prev_keys: KeyInput,
}

unsafe impl StaticInitSafe for KeysManager {
    fn init(&mut self) {
        self.reset_internal();
    }
}

ewram_static!(Keys: KeysManager = KeysManager::new());

impl KeysManager {
    const fn new() -> Self {
        KeysManager {
            frame_keys: KeyInput::new(),
            prev_keys: KeyInput::new(),
        }
    }

    fn reset_internal(&mut self) {
        self.frame_keys = KeyInput::new();
        self.prev_keys = KeyInput::new();
    }

    pub fn on_vblank() -> KeysResponse {
        let manager = Keys.get_or_init();
        manager.prev_keys = manager.frame_keys;
        manager.frame_keys = KEYINPUT.read();
        KeysResponse {
            keys: manager.frame_keys,
            prev_keys: manager.prev_keys,
        }
    }

    pub fn keys() -> KeysResponse {
        let manager = Keys.get_or_init();
        KeysResponse {
            keys: manager.frame_keys,
            prev_keys: manager.prev_keys,
        }
    }
}

#[derive(Clone, Copy)]
pub struct KeysResponse {
    pub keys: KeyInput,
    prev_keys: KeyInput,
}

impl KeysResponse {
    pub fn is_just_pressed(&self, key: KeyInput) -> bool {
        (self.keys & key) == key && (self.prev_keys & key) == KeyInput::new()
    }

    pub fn is_just_released(&self, key: KeyInput) -> bool {
        (self.keys & key) == KeyInput::new() && (self.prev_keys & key) == key
    }

    pub fn is_held(&self, key: KeyInput) -> bool {
        (self.keys & key) == (self.prev_keys & key) && (self.keys & key) != KeyInput::new()
    }

    pub fn is_up(&self, key: KeyInput) -> bool {
        (self.keys & key) == KeyInput::new()
    }

    pub fn is_down(&self, key: KeyInput) -> bool {
        (self.keys & key) == key
    }

    pub fn left(&self) -> bool {
        self.keys.left()
    }
    pub fn right(&self) -> bool {
        self.keys.right()
    }
    pub fn up(&self) -> bool {
        self.keys.up()
    }
    pub fn down(&self) -> bool {
        self.keys.down()
    }
    pub fn a(&self) -> bool {
        self.keys.a()
    }
    pub fn b(&self) -> bool {
        self.keys.b()
    }
    pub fn start(&self) -> bool {
        self.keys.start()
    }
    pub fn select(&self) -> bool {
        self.keys.select()
    }
    pub fn r(&self) -> bool {
        self.keys.r()
    }
    pub fn l(&self) -> bool {
        self.keys.l()
    }
}
