//! Logic for dithering a loaded, preprocessed [Img][crate::img::Img].
//! See [tanner helland's excellent writeup on dithering algorithms](http://www.tannerhelland.com/4660/dithering-eleven-algorithms-source-code/) for details.
use super::Img;
use std::ops::{Add, Div, Mul};

/// dither a 2d matrix.
/// `P`  is the type of pixel; in practice, it is either [f64] or [`RGB<f64>`][RGB]
pub trait Dither<P> {
    fn dither(&self, img: Img<P>, quantize: impl FnMut(P) -> (P, P)) -> Img<P>;
}
/// A type of Dither. See the documentation for the constants (i.e, [ATKINSON]) for the dither matrices themselves.
/// A ditherer carries error from quantiation to nearby pixels after dividing by `div` and multiplying by the given scalar in offset; "spreading" the error,
/// eg, take floyd-steinberg dithering: `div=16`
/// - ` . x   7 `
/// - ` 7 5  1`
///
/// Suppose we have an grayscale image where the `f64` pixel "`p`"" at `(x=3, y=0)` is 100., and we quantize to black (0.0) and white(255.0)
/// The ditherer sets `p=0` and has a carried error of 100.
/// The spread error is then:
/// -   ` . ----  . ---    43.75`
/// -   `43.75   31.25   6.25`
///
///
/// See [tanner helland's excellent writeup on dithering algorithms](http://www.tannerhelland.com/4660/dithering-eleven-algorithms-source-code/)
/// for details.
#[derive(Clone, Debug)]
pub struct Ditherer<'a> {
    div: f64,
    /// offsets represents a triplet (dx, dy, mul)
    offsets: &'a [(isize, isize, f64)],
    name: Option<&'a str>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// An unknown ditherer name during parsing.
pub struct ErrorUnknownDitherer(String);

impl<'a> Ditherer<'a> {
    pub const fn new(div: f64, offsets: &'a [(isize, isize, f64)]) -> Self {
        Ditherer {
            div,
            offsets,
            name: None,
        }
    }
}

impl<'a, P> Dither<P> for Ditherer<'a>
where
    P: Add<Output = P> + Clone + Default,           // vec addition
    P: Mul<f64, Output = P> + Div<f64, Output = P>, // scalar multiplication
{
    /// dither an image using the specified offsets and divisor.
    /// `P` is the type of pixel; in practice, it is either [f64] or [RGB<f64]
    fn dither(&self, mut img: Img<P>, mut quantize: impl FnMut(P) -> (P, P)) -> super::Img<P> {
        let width = img.width() as isize;
        let mut spillover = vec![P::default(); img.len()];
        for (i, p) in img.iter_mut().enumerate() {
            let (quantized, spill) = quantize(p.clone() + spillover[i].clone());
            *p = quantized;

            // add spillover matrices
            for (dx, dy, mul) in self.offsets.iter().cloned() {
                let j = i as isize + (dy * width) + dx;

                if let Some(stored_spill) = spillover.get_mut(j as usize) {
                    // this cast is OK, since if we go past the edges, we get zero
                    *stored_spill = stored_spill.clone() + (spill.clone() * mul) / self.div;
                }
            }
        }
        img
    }
}

impl std::str::FromStr for Ditherer<'static> {
    type Err = ErrorUnknownDitherer;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s.to_ascii_lowercase().as_ref() {
            "floyd" | "steinberg" | "floydsteinberg" | "floyd steinberg" => FLOYD_STEINBERG,
            "atkinson" => ATKINSON,
            "stucki" => STUCKI,
            "burkes" => BURKES,
            "jarvis" | "judice" | "ninke" => JARVIS_JUDICE_NINKE,
            "sierra" | "sierra3" => SIERRA_3,
            _ => return Err(ErrorUnknownDitherer(s.to_string())),
        })
    }
}

/// Atkinson dithering. Div=8.
/// - `.  x  1  1`
/// - `1  1  1  .`
/// - `.  1  .  .`
pub const ATKINSON: Ditherer = Ditherer {
    name: Some("atkinson"),
    div: 8.,
    offsets: &[
        // (dx, dy, mul)
        (1, 0, 1.),
        (2, 0, 1.),
        //
        (-1, 1, 1.),
        (0, 1, 1.),
        (1, 1, 1.),
        //
        (0, 2, 1.),
    ],
};

/// Burkes dithering. Div=32.
/// - ` .  .  x  8  4`
/// - ` 2  4  8  4  2`
pub const BURKES: Ditherer = Ditherer {
    name: Some("burkes"),
    div: 32.,
    offsets: &[
        // (dx, dy, mul)
        (1, 0, 8.),
        (2, 0, 4.),
        //
        (-2, 1, 2.),
        (-1, 1, 4.),
        (0, 1, 8.),
        (1, 1, 4.),
        (2, 1, 2.),
    ],
};

/// floyd-steinberg dithering. `div=16`
///
/// - ` . x   7 `
/// - ` 7 5  1`
pub const FLOYD_STEINBERG: Ditherer = Ditherer {
    name: Some("floyd"),
    div: 16.,
    offsets: &[(1, 0, 7.), (-1, 1, 3.), (0, 1, 5.), (1, 1, 1.)],
};

/// Stucki dithering. `div=42`
///
/// - ` .  .  x  8  4`
/// - ` 2  4  8  4  2`
/// - ` 1  2  4  2  1`
pub const STUCKI: Ditherer = Ditherer {
    name: Some("stucki"),
    div: 42.,
    offsets: &[
        // (dx, dy, mul)
        (1, 0, 8.),
        (2, 0, 4.),
        //
        (-2, 1, 2.),
        (-1, 1, 4.),
        (0, 1, 8.),
        (1, 1, 4.),
        (2, 1, 2.),
        //
        (-2, 2, 1.),
        (-1, 2, 2.),
        (0, 2, 4.),
        (1, 2, 2.),
        (2, 2, 1.),
    ],
};

/// jarvis-judice-ninke dithering`. div=48.
///
/// - `.  .  x  7  5`
/// - `3  5  7  5  3`
/// - `1  3  5  3  1`  
pub const JARVIS_JUDICE_NINKE: Ditherer = Ditherer {
    name: Some("jarvis"),
    div: 48.0,
    offsets: &[
        // (dx, dy, mul)
        (1, 0, 7.),
        (2, 0, 5.),
        //
        (-2, 1, 3.),
        (-1, 1, 5.),
        (0, 1, 7.),
        (1, 1, 5.),
        (2, 1, 3.),
        //
        (-2, 2, 1.),
        (-1, 2, 3.),
        (0, 2, 5.),
        (1, 2, 3.),
        (2, 2, 1.),
    ],
};

/// sierra 3 dithering. div=32
/// - `.  .  x  5  3`
/// - `2  4  5  4  2`
/// - `.  2  3  2  .`
pub const SIERRA_3: Ditherer = Ditherer {
    name: Some("sierra3"),
    div: 32.,
    offsets: &[
        // (dx, dy, mul)
        (1, 0, 5.),
        (2, 0, 3.),
        //
        (-2, 1, 2.),
        (-1, 1, 4.),
        (0, 1, 5.),
        (1, 1, 4.),
        (2, 1, 2.),
        //
        (-1, 2, 2.),
        (0, 2, 3.),
        (1, 2, 2.),
    ],
};
//
//             X   5   3
//     2   4   5   4   2
//         2   3   2
//           (1/32)

impl std::fmt::Display for ErrorUnknownDitherer {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "unknown ditherer: {}", self.0)
    }
}

impl<'a> std::fmt::Display for Ditherer<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "{}",
            if let Some(name) = self.name {
                name
            } else {
                "custom ditherer"
            }
        )
    }
}

impl std::error::Error for ErrorUnknownDitherer {}

impl<'a> Eq for Ditherer<'a> {}

impl<'a> PartialEq for Ditherer<'a> {
    fn eq(&self, other: &Self) -> bool {
        (self.div, self.offsets) == (other.div, other.offsets)
    }
}

impl<'a> Default for Ditherer<'a> {
    fn default() -> Self {
        FLOYD_STEINBERG
    }
}
