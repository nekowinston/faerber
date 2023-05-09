//! # Dither
//! # Written by Efron Licht. Available under the MIT license. Hire me!
//!
//!
//! Inspired by: <http://www.tannerhelland.com/4660/dithering-eleven-algorithms-source-code/>
//! and the game "Return of the Obra Dinn"

pub mod color;
pub mod ditherer;
mod error;
mod img;
pub mod prelude;
use self::error::DitherResult;
use self::error::Error;

use self::prelude::*;
#[cfg(test)]
mod tests;

/// quantize to n bits. See the [bit_depth][crate::opts::Opt] option.
/// ```
/// # use dither::prelude::*;
/// # use dither::create_quantize_n_bits_func;
/// let one_bit = create_quantize_n_bits_func(1).unwrap();
/// let want = 0;
/// assert_eq!(one_bit(100.), (0., 100.));
/// assert_eq!(one_bit(250.), (255., -5.));
///
/// ```
pub fn create_quantize_n_bits_func(n: u8) -> DitherResult<impl Fn(f64) -> (f64, f64)> {
    if n == 0 || n > 7 {
        Err(Error::BadBitDepth(n))
    } else {
        Ok(move |x: f64| {
            let step_size = 255. / f64::from(n);

            let floor = f64::floor(x / step_size) * step_size;
            let floor_rem = x - floor;

            let ceil = f64::ceil(x / step_size) * step_size;
            let ceil_rem = ceil - x;

            if floor_rem < ceil_rem {
                let quot = f64::max(floor, 0.0);
                (quot, floor_rem)
            } else {
                let quot = f64::min(255.0, ceil);
                (quot, -ceil_rem)
            }
        })
    }
}

/// clamp a f64 to the closest u8, rounding non-integers.
/// ```
/// # use dither::clamp_f64_to_u8;
/// assert_eq!(clamp_f64_to_u8(255.2), 255);
/// assert_eq!(clamp_f64_to_u8(2.8), 3);
/// ```
pub fn clamp_f64_to_u8(n: f64) -> u8 {
    match n {
        n if n > 255.0 => 255,
        n if n < 0.0 => 0,
        n => n.round() as u8,
    }
}
