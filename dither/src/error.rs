//! Error and result types for runtime error.
use crate::prelude::*;
use image::ImageError;
use std::path::{Path, PathBuf};
/// Handling of runtime errors in main.
#[derive(Debug)]
pub enum Error {
    /// An error saving an image from file.
    Output(IOError),
    // An error loading an image.
    Input(IOError),
    /// A bit depth that's not in the [range][std::ops::Range] `0..8`
    BadBitDepth(u8),
    /// An error creating a [color::Mode]
    Color(color::Error),
    /// The user has specified both [color::Mode::CustomPalette] and the bit depth [Opt]
    CustomPaletteIncompatibleWithDepth,
}

/// Result type for [Error]
pub type DitherResult<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub struct IOError {
    err: ImageError,
    path: PathBuf,
}
impl Error {
    pub fn input(err: impl Into<ImageError>, path: impl AsRef<Path>) -> Self {
        Error::Input(IOError::new(err, path))
    }
    pub fn output(err: impl Into<ImageError>, path: impl AsRef<Path>) -> Self {
        Error::Output(IOError::new(err, path))
    }
}

impl IOError {
    pub fn new(err: impl Into<ImageError>, path: impl AsRef<Path>) -> Self {
        IOError {
            path: path.as_ref().to_owned(),
            err: err.into(),
        }
    }
}

impl std::fmt::Display for IOError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let IOError { path, err } = self;
        write!(f, "on path \"{}\": {}", path.display(), err)
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Error::Input(err) => write!(f, "input error: loading image: {}", err),
            Error::Output(err) => write!(f, "output error: output: {}", err),
            Error::BadBitDepth(n) => write!(
                f,
                "configuration error: bit depth must be between 1 and 7, but was {}",
                n
            ),
            Error::Color(err) => write!(f, "configuration error for: {}", err),
            Error::CustomPaletteIncompatibleWithDepth => f.write_str(
                "error: the custom palette --color option is incompatible with the --depth option",
            ),
        }
    }
}

impl From<color::Error> for Error {
    fn from(err: color::Error) -> Self {
        Error::Color(err)
    }
}

impl std::error::Error for Error {}
