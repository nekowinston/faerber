//! handling of color modes & [RGB].

mod rgb;

pub use palette::Palette;
use palette::{cga, crayon};
pub use rgb::RGB;

pub mod palette;
use std::borrow::Cow;
use std::path::Path;
use std::str::FromStr;
#[derive(Clone, Debug, PartialEq, Eq)]
/// Mode is the color mode the program runs in. Corresponds to [Opt][crate::Opt] `--color`
#[derive(Default)]
pub enum Mode {
    /// A single known [RGB] color.
    /// -  `--color="RED"`
    SingleColor(RGB<u8>),

    /// Color dithering to the user-specified bit depth.
    /// - `--color="color"`
    Color,
    /// Grayscale dithering to the user-specified bit depth.
    /// - `-color="bw"`(default)
    #[default]
    BlackAndWhite,
    /// A user-specified palette, read from a file or the CGA option. See [parse-palette] and the readme for details on palette files.
    /// - `color==$FILENAME`
    /// - `color==cga`
    Palette {
        palette: Cow<'static, Palette>,
        name: Cow<'static, str>,
    },
}

impl Mode {
    /// the built-in CGA palette. see the [cga] module for a list of contained colors.
    const CGA: Self = Mode::Palette {
        palette: Cow::Borrowed(cga::ALL),
        name: Cow::Borrowed("CGA"),
    };
    /// the built in CRAYON palette. see the [crayon] module for a list of contained colors.
    const CRAYON: Self = Mode::Palette {
        palette: Cow::Borrowed(crayon::ALL),
        name: Cow::Borrowed("CRAYON"),
    };
}

#[derive(Debug)]
/// An error handling the `--color` input option.
pub enum Error {
    /// Error parsing the palette as a hexidecimal unsigned integer
    RGBParse,
    /// The custom palette only has one (or zero! colors)
    PaletteTooSmall,
    /// An unknown user option.
    UnknownOption(String),

    /// An error accessing a file
    BadFile { path: String, err: std::io::Error },
}

impl std::fmt::Display for Mode {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Mode::Palette { name, .. } => write!(f, "custom_palette_{}", name),
            Mode::Color => write!(f, "color"),
            Mode::SingleColor(color) => write!(f, "single_color_{:x}", color),
            Mode::BlackAndWhite => write!(f, "bw"),
        }
    }
}

impl FromStr for Mode {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s.trim().to_ascii_uppercase().as_ref() {
            "WHITE" | "BLACK" | "BW" => Mode::BlackAndWhite,
            "C" | "COLOR" => Mode::Color,
            "CGA" => Mode::CGA,
            "CRAYON" => Mode::CRAYON,

            "BLUE" => Mode::SingleColor(cga::BLUE),
            "GREEN" => Mode::SingleColor(cga::GREEN),
            "CYAN" => Mode::SingleColor(cga::CYAN),
            "RED" => Mode::SingleColor(cga::RED),
            "MAGENTA" => Mode::SingleColor(cga::MAGENTA),
            "BROWN" => Mode::SingleColor(cga::BROWN),
            "LIGHT_GRAY" => Mode::SingleColor(cga::LIGHT_GRAY),
            "GRAY" => Mode::SingleColor(cga::GRAY),
            "LIGHT_BLUE" => Mode::SingleColor(cga::LIGHT_BLUE),
            "LIGHT_GREEN" => Mode::SingleColor(cga::LIGHT_GREEN),
            "LIGHT_CYAN" => Mode::SingleColor(cga::LIGHT_CYAN),
            "LIGHT_RED" => Mode::SingleColor(cga::LIGHT_RED),
            "LIGHT_MAGENTA" => Mode::SingleColor(cga::LIGHT_MAGENTA),
            "YELLOW" => Mode::SingleColor(cga::YELLOW),

            _ if Path::new(s).is_file() => {
                let path = s.to_string();
                return match std::fs::read_to_string(s) {
                    // note: we don't use
                    Ok(contents) => Ok(Mode::Palette {
                        palette: palette::parse(&contents)?,
                        name: Cow::Owned(path),
                    }),
                    Err(err) => Err(Error::BadFile { err, path }),
                };
            }

            _ => return Err(Error::UnknownOption(s.to_string())),
        })
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Error::UnknownOption(opt) => writeln!(f, "unknown color option {}", opt),
            Error::PaletteTooSmall => writeln!(
                f,
                "user-specified palette has 0 or 1 color; must have at least two"
            ),
            Error::RGBParse =>        write!(f, "could not parse to a RGB value: bad format. must be exactly six hexidecimal characters, with optional 0x prefix"),
            Error::BadFile{path, err} => write!(f, "could not load color palette from file at path \"{}\": {}", path, err),
        }
    }
}

impl PartialEq for Error {
    fn eq(&self, other: &Self) -> bool {
        use self::Error::*;
        match (self, other) {
            (RGBParse, RGBParse) | (PaletteTooSmall, PaletteTooSmall) => true,
            (UnknownOption(a), UnknownOption(b)) => a == b,
            (BadFile { path: p0, err: e0 }, BadFile { path: p1, err: e1 }) => {
                p0 == p1 && e0.to_string() == e1.to_string()
            }

            _ => false,
        }
    }
}

impl Eq for Error {}
