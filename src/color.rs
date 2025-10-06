use gba::{fixed::i32fx8, video::Color};

pub const fn make_color(r: u16, g: u16, b: u16) -> Color {
    return Color(r | (g << 5) | (b << 10));
}

const FMAX: u16 = 0b11111;
pub const TRANSPARENT: Color = make_color(0, 0, 0);
pub const WHITE: Color = make_color(FMAX, FMAX, FMAX);
pub const BLACK: Color = make_color(1, 1, 1); // Slightly off-black to distinguish from transparent
pub const RED: Color = make_color(FMAX, 0, 0);
pub const GREEN: Color = make_color(0, FMAX, 0);
pub const BLUE: Color = make_color(0, 0, FMAX);
pub const YELLOW: Color = make_color(FMAX, FMAX, 0);
pub const CYAN: Color = make_color(0, FMAX, FMAX);
pub const MAGENTA: Color = make_color(FMAX, 0, FMAX);
pub const ORANGE: Color = make_color(FMAX, 20, 0);
pub const PURPLE: Color = make_color(20, 0, 20);
pub const PINK: Color = make_color(FMAX, 20, 25);
pub const BROWN: Color = make_color(18, 9, 0);
pub const GRAY: Color = make_color(15, 15, 15);
pub const LIGHT_GRAY: Color = make_color(24, 24, 24);
pub const DARK_GREEN: Color = make_color(0, 15, 0);

#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum PaletteColor {
    Transparent = 0,
    White = 1,
    Red = 2,
    Green = 3,
    Blue = 4,
    Yellow = 5,
    Cyan = 6,
    Magenta = 7,
    Orange = 8,
    Purple = 9,
    Pink = 10,
    Brown = 11,
    Gray = 12,
    LightGray = 13,
    DarkGreen = 14,
    Black = 15,
}

pub const fn darken_rgb15(color: Color, factor: i32fx8) -> Color {
    let r = i32fx8::wrapping_from((color.0 & 0x1F) as i32);
    let g = i32fx8::wrapping_from(((color.0 >> 5) & 0x1F) as i32);
    let b = i32fx8::wrapping_from(((color.0 >> 10) & 0x1F) as i32);

    let r = (r.mul(factor).to_bits() >> 8) as u16;
    let g = (g.mul(factor).to_bits() >> 8) as u16;
    let b = (b.mul(factor).to_bits() >> 8) as u16;

    make_color(r, g, b)
}
