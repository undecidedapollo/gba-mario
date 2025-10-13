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

pub const BRICK: Tile = Tile::new(get_tile_idx(0, 0));
pub const QUESTION_BLOCK_UNUSED: Tile = Tile::new(get_tile_idx(0, 2));
pub const QUESTION_BLOCK_USED: Tile = Tile::new(get_tile_idx(0, 3));
pub const ROCK: Tile = Tile::new(get_tile_idx(0, 1));
pub const PIPE_TOP_LEFT: Tile = Tile::new(get_tile_idx(0, 6));
pub const PIPE_TOP_RIGHT: Tile = Tile::new(get_tile_idx(0, 7));
pub const PIPE_BODY_LEFT: Tile = Tile::new(get_tile_idx(1, 6));
pub const PIPE_BODY_RIGHT: Tile = Tile::new(get_tile_idx(1, 7));

pub const BUSH_LEFT: Tile = Tile::new(get_tile_idx(1, 2));
pub const BUSH_MIDDLE: Tile = Tile::new(get_tile_idx(1, 3));
pub const BUSH_RIGHT: Tile = Tile::new(get_tile_idx(3, 7));

pub const MOUNTAIL_TOP: Tile = Tile::new(get_tile_idx(3, 3));
pub const MOUNTAIL_SLOPE_UP: Tile = Tile::new(get_tile_idx(4, 3));
pub const MOUNTAIL_BUTTONS: Tile = Tile::new(get_tile_idx(4, 4));
pub const MOUNTAIL_EMPTY: Tile = Tile::new(get_tile_idx(4, 5));
pub const MOUNTAIL_SLOPE_DOWN: Tile = Tile::new(get_tile_idx(4, 7));

pub struct Level {
    pub floor: LevelFloor,
    pub data: &'static [LevelItem],
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum LevelItem {
    Tile { tile: Tile, row: usize, len: usize },
    HoleInFloor { len: usize },
    Pipe { row: usize },
    NextCol { advance_by: usize },
}

const FLOOR: usize = 15;
const SCREEN_WIDTH: usize = 16;

const fn from_floor(up_from_floor: usize) -> usize {
    (FLOOR - 1).saturating_sub(up_from_floor)
}

struct MultilayerSprite<const W: usize, const H: usize> {
    tiles: [[Tile; W]; H],
}

pub const LEVEL_1_1_DATA: &[LevelItem] = &[
    LevelItem::NextCol { advance_by: 4 },
    LevelItem::Tile {
        tile: QUESTION_BLOCK_UNUSED,
        row: from_floor(3),
        len: 4,
    },
    LevelItem::NextCol {
        advance_by: SCREEN_WIDTH - 4,
    },
    LevelItem::Tile {
        tile: QUESTION_BLOCK_UNUSED,
        row: from_floor(3),
        len: 1,
    },
    // LevelItem::HoleInFloor { len: 2 },
    LevelItem::NextCol { advance_by: 4 },
    LevelItem::Tile {
        tile: BRICK,
        row: from_floor(3),
        len: 5,
    },
    LevelItem::NextCol { advance_by: 1 },
    LevelItem::Tile {
        tile: QUESTION_BLOCK_UNUSED,
        row: from_floor(3),
        len: 1,
    },
    LevelItem::NextCol { advance_by: 1 },
    LevelItem::Tile {
        tile: BRICK,
        row: from_floor(7),
        len: 1,
    },
    LevelItem::NextCol { advance_by: 1 },
    LevelItem::Tile {
        tile: QUESTION_BLOCK_UNUSED,
        row: from_floor(3),
        len: 1,
    },
    LevelItem::NextCol { advance_by: 5 },
    LevelItem::Pipe { row: from_floor(1) },
    LevelItem::NextCol { advance_by: 10 },
    LevelItem::Pipe { row: from_floor(2) },
    LevelItem::NextCol { advance_by: 8 },
    LevelItem::Pipe { row: from_floor(3) },
    LevelItem::NextCol { advance_by: 11 },
    LevelItem::Pipe { row: from_floor(3) },
    LevelItem::NextCol { advance_by: 12 },
    LevelItem::HoleInFloor { len: 2 },
];

pub const LEVEL_1_1: Level = Level {
    floor: LevelFloor::Solid {
        tile: ROCK,
        row: FLOOR,
    },
    data: &LEVEL_1_1_DATA,
};
