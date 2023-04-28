use crate::color::{rgb::RGB, Palette};

/// All the colors defined by the constants in this crate;
/// BLACK,BLUE,GREEN,CYAN,RED,MAGENTA,BROWN,LIGHT_GRAY,GRAY,LIGHT_BLUE,LIGHT_GREEN,LIGHT_CYAN,LIGHT_RED,LIGHT_MAGENTA,YELLOW,WHITE,
/// TRUE_BLACK, TRUE_WHITE

pub const ALL: &Palette = &[
    BLACK,
    BLUE,
    GREEN,
    CYAN,
    RED,
    MAGENTA,
    BROWN,
    LIGHT_GRAY,
    GRAY,
    LIGHT_BLUE,
    LIGHT_GREEN,
    LIGHT_CYAN,
    LIGHT_RED,
    LIGHT_MAGENTA,
    YELLOW,
    WHITE,
];
pub const BLACK: RGB<u8> = RGB(0x00, 0x00, 0x00);
pub const BLUE: RGB<u8> = RGB(0x00, 0x00, 0xAA);
pub const GREEN: RGB<u8> = RGB(0x00, 0xAA, 0x00);
pub const CYAN: RGB<u8> = RGB(0x00, 0xAA, 0xAA);
pub const RED: RGB<u8> = RGB(0xAA, 0x00, 0x00);
pub const MAGENTA: RGB<u8> = RGB(0xAA, 0x00, 0xAA);
pub const BROWN: RGB<u8> = RGB(0xAA, 0x55, 0x00);
pub const LIGHT_GRAY: RGB<u8> = RGB(0xAA, 0xAA, 0xAA);
pub const GRAY: RGB<u8> = RGB(0x55, 0x55, 0x55);
pub const LIGHT_BLUE: RGB<u8> = RGB(0x55, 0x55, 0xFF);
pub const LIGHT_GREEN: RGB<u8> = RGB(0x55, 0xFF, 0x55);
pub const LIGHT_CYAN: RGB<u8> = RGB(0x55, 0xFF, 0xFF);
pub const LIGHT_RED: RGB<u8> = RGB(0xFF, 0x55, 0x55);
pub const LIGHT_MAGENTA: RGB<u8> = RGB(0xFF, 0x55, 0xFF);
pub const YELLOW: RGB<u8> = RGB(0xFF, 0xFF, 0x55);
pub const WHITE: RGB<u8> = RGB(0xFF, 0xFF, 0xFF);
