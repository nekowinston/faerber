//! Re-exports of the most common traits and types.
pub use super::*;

pub use self::{
    color::{palette::Palette, RGB},
    ditherer::{Dither, Ditherer},
    error::{DitherResult, Error, IOError},
    img::Img,
};
