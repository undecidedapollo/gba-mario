use crate::assets::BACKGROUND_TILE_COLS_PER_ROW;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum LevelFloor {
    Solid { tile: Tile, row: usize },
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Tile(usize);

impl Tile {
    pub const fn new(id: usize) -> Self {
        Tile(id)
    }

    pub fn id(&self) -> usize {
        self.0
    }

    pub fn top_left(&self) -> usize {
        self.0
    }
    pub fn top_right(&self) -> usize {
        self.0 + 1
    }
    pub fn bottom_left(&self) -> usize {
        self.0 + BACKGROUND_TILE_COLS_PER_ROW
    }
    pub fn bottom_right(&self) -> usize {
        self.0 + BACKGROUND_TILE_COLS_PER_ROW + 1
    }
}

const fn get_tile_idx(row: usize, col: usize) -> usize {
    (row * 2 * BACKGROUND_TILE_COLS_PER_ROW + col * 2) + 1
}

pub static BRICK: Tile = Tile::new(get_tile_idx(0, 0));
pub static ROCK: Tile = Tile::new(get_tile_idx(0, 1));
pub static PIPE_TOP_LEFT: Tile = Tile::new(get_tile_idx(0, 6));
pub static PIPE_TOP_RIGHT: Tile = Tile::new(get_tile_idx(0, 7));
pub static PIPE_BODY_LEFT: Tile = Tile::new(get_tile_idx(1, 6));
pub static PIPE_BODY_RIGHT: Tile = Tile::new(get_tile_idx(1, 7));

pub struct Level {
    pub floor: LevelFloor,
    pub data: &'static [LevelItem],
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum LevelItem {
    Tile { tile: Tile, row: usize, len: usize },
    Pipe { row: usize },
    NextCol { advance_by: usize },
}

pub const LEVEL_1_1_DATA: [LevelItem; 8] = [
    LevelItem::NextCol { advance_by: 8 },
    LevelItem::Tile {
        tile: PIPE_BODY_RIGHT,
        row: 14,
        len: 3,
    },
    LevelItem::NextCol { advance_by: 4 },
    LevelItem::Pipe { row: 16 },
    LevelItem::NextCol { advance_by: 4 },
    LevelItem::Tile {
        tile: BRICK,
        row: 14,
        len: 4,
    },
    LevelItem::NextCol { advance_by: 8 },
    LevelItem::Tile {
        tile: BRICK,
        row: 14,
        len: 3,
    },
];

pub const LEVEL_1_1: Level = Level {
    floor: LevelFloor::Solid {
        tile: ROCK,
        row: 22,
    },
    data: &LEVEL_1_1_DATA,
};
