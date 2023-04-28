use crate::color::rgb::RGB;
use crate::color::Palette;

/// all the colors in the rainbow. or at least the rainbow in a ~24 crayon box.
pub const ALL: &Palette = &[
    YELLOW,
    BLUE,
    BLACK,
    VIOLET,
    BLUE_GREEN,
    RED_VIOLET,
    RED_ORANGE,
    YELLOW_GREEN,
    RED,
    ORANGE,
    DANDELION,
    CERULEAN,
    WHITE,
    VIOLET_RED,
    GRAY,
    INDIGO,
    APRICOT,
    SCARLET,
    CARNATION_PINK,
    GREEN,
    BLUE_VIOLET,
    BROWN,
    GREEN_YELLOW,
    TRUE_BLACK,
    TRUE_WHITE,
];

pub const YELLOW: RGB<u8> = RGB(0xFC, 0xE8, 0x83);
pub const BLUE: RGB<u8> = RGB(0x1F, 0x75, 0xFE);
pub const BLACK: RGB<u8> = RGB(0x23, 0x23, 0x23);
pub const VIOLET: RGB<u8> = RGB(0x92, 0x6E, 0xAE);
pub const BLUE_GREEN: RGB<u8> = RGB(0x19, 0x9E, 0xBD);
pub const RED_VIOLET: RGB<u8> = RGB(0xC0, 0x44, 0x8F);
pub const RED_ORANGE: RGB<u8> = RGB(0xFF, 0x53, 0x49);
pub const YELLOW_GREEN: RGB<u8> = RGB(0xC5, 0xE3, 0x84);
pub const RED: RGB<u8> = RGB(0xEE, 0x20, 0x4D);
pub const ORANGE: RGB<u8> = RGB(0xFF, 0x75, 0x38);
pub const DANDELION: RGB<u8> = RGB(0xFD, 0xDB, 0x6D);
pub const CERULEAN: RGB<u8> = RGB(0x1D, 0xAC, 0xD6);
pub const WHITE: RGB<u8> = RGB(0xED, 0xED, 0xED);
pub const VIOLET_RED: RGB<u8> = RGB(0xF7, 0x53, 0x94);
pub const GRAY: RGB<u8> = RGB(0x95, 0x91, 0x8C);
pub const INDIGO: RGB<u8> = RGB(0x5D, 0x76, 0xCB);
pub const APRICOT: RGB<u8> = RGB(0xFD, 0xD9, 0xB5);
pub const SCARLET: RGB<u8> = RGB(0xFC, 0x28, 0x47);
pub const CARNATION_PINK: RGB<u8> = RGB(0xFF, 0xAA, 0xCC);
pub const GREEN: RGB<u8> = RGB(0x1C, 0xAC, 0x78);
pub const BLUE_VIOLET: RGB<u8> = RGB(0x73, 0x66, 0xBD);
pub const BROWN: RGB<u8> = RGB(0xB4, 0x67, 0x4D);
pub const GREEN_YELLOW: RGB<u8> = RGB(0xF0, 0xE8, 0x91);
pub const TRUE_BLACK: RGB<u8> = RGB(0x00, 0x00, 0x00);
pub const TRUE_WHITE: RGB<u8> = RGB(0xff, 0xff, 0xff);
