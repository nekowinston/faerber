#![warn(
    // clippy::cargo,
    // clippy::pedantic,
    clippy::complexity,
    clippy::expect_used,
    clippy::nursery,
    clippy::perf,
    clippy::style,
    clippy::suspicious,
    clippy::unwrap_used,
)]

use faerber_lib::Lab;
use lazy_static::lazy_static;
use std::collections::BTreeMap;
use thiserror::Error;

type WezTermColorscheme = BTreeMap<String, Vec<String>>;
type SavedColorscheme = BTreeMap<String, BTreeMap<String, String>>;

#[derive(Clone, Default, Debug)]
pub struct Color {
    pub name: String,
    pub value: u32,
    pub enabled: bool,
}
impl Color {
    #[must_use]
    pub const fn new(name: String, value: u32) -> Self {
        Self {
            name,
            value,
            enabled: true,
        }
    }
    pub fn get_lab(&self) -> Lab {
        Lab::from_rgb(&[
            ((self.value >> 16) & 0xFF) as u8,
            ((self.value >> 8) & 0xFF) as u8,
            (self.value & 0xFF) as u8,
        ])
    }
}
pub type Palette = BTreeMap<String, Color>;
#[derive(Clone, Default, Debug)]
pub struct Flavor {
    pub name: String,
    pub palette: Palette,
    pub enabled: bool,
}
impl Flavor {
    #[must_use]
    pub fn new(name: String, palette: Palette) -> Self {
        Self {
            name,
            palette,
            enabled: true,
        }
    }
    pub fn get(&self, name: &str) -> Option<&Color> {
        self.palette.get(name)
    }
    pub fn get_colors(&self) -> Vec<Color> {
        self.palette.values().cloned().collect::<Vec<_>>()
    }
    pub fn get_labs(&self) -> Vec<Lab> {
        self.palette
            .values()
            .filter(|c| c.enabled)
            .map(|c| c.get_lab())
            .collect::<Vec<_>>()
    }
    pub fn get_u32s(&self) -> Vec<u32> {
        self.palette
            .values()
            .filter(|c| c.enabled)
            .map(|c| c.value)
            .collect::<Vec<_>>()
    }
}

pub type ColorScheme = BTreeMap<String, Flavor>;
pub type Library = BTreeMap<String, ColorScheme>;

#[derive(Clone, Debug)]
pub struct LibraryManager {
    pub library: Library,
}

#[derive(Debug, Error)]
pub enum LibraryError {
    #[error("Colorscheme {0} does not exist")]
    NoSuchColorscheme(String),
    #[error("Flavor {0} does not exist")]
    NoSuchFlavor(String),
    #[error("Color {0} does not exist")]
    NoSuchColor(String),
    #[error("Failed to parse scheme: {0}")]
    ParseColorSchemeError(String),
    #[error("Failed to parse color: {0}")]
    ParseColorError(String),
}

impl LibraryManager {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
    /// Adds a colorscheme to this [`LibraryManager`].
    ///
    /// # Errors
    ///
    /// This function will return an error if the colorscheme cannot be parsed.
    pub fn add_colorscheme(&mut self, name: &str, cs: &str) -> Result<ColorScheme, LibraryError> {
        let colorscheme = Self::parse_colorscheme(name, cs)?;
        self.library.insert(name.to_string(), colorscheme.clone());
        Ok(colorscheme)
    }
    /// Parses a colorscheme.
    ///
    /// # Errors
    ///
    /// This function will return an error if the colorscheme cannot be parsed.
    pub fn parse_colorscheme(name: &str, cs: &str) -> Result<ColorScheme, LibraryError> {
        serde_json::from_str::<SavedColorscheme>(cs).map_or_else(
            |_| Err(LibraryError::ParseColorSchemeError(name.to_owned())),
            |saved_cs| {
                let mut colorscheme = ColorScheme::new();

                for (flavor_name, flavor) in saved_cs {
                    let mut flavor_palette = Palette::new();

                    for (name, value) in flavor {
                        let value = value.trim_start_matches('#');
                        if let Ok(val) = u32::from_str_radix(value, 16) {
                            let color = Color::new(name, val);
                            flavor_palette.insert(color.name.clone(), color);
                        }
                    }

                    colorscheme.insert(name.to_owned(), Flavor::new(flavor_name, flavor_palette));
                }

                Ok(colorscheme)
            },
        )
    }
    /// Sets the color of this [`LibraryManager`].
    ///
    /// # Errors
    ///
    /// This function will return an error if .
    pub fn set_color(
        &mut self,
        cs: &str,
        flavor: &str,
        color: &str,
        status: bool,
    ) -> Result<bool, LibraryError> {
        self.library
            .get_mut(cs)
            .ok_or_else(|| LibraryError::NoSuchColorscheme(cs.to_owned()))?
            .get_mut(flavor)
            .ok_or_else(|| LibraryError::NoSuchFlavor(flavor.to_owned()))?
            .palette
            .get_mut(color)
            .ok_or_else(|| LibraryError::NoSuchColor(color.to_owned()))?
            .enabled = status;
        Ok(status)
    }
    pub fn get(&self, name: &str) -> Option<&ColorScheme> {
        self.library.get(name)
    }
}

lazy_static! {
    pub static ref DEFAULT_LIBRARY: Library = {
        #[allow(clippy::unwrap_used)]
        let vendored_colorschemes =
            // clippy::ignore
            parse_wezterm_colorscheme(include_str!("../vendor/wezterm.json")).unwrap();
        vendored_colorschemes
    };
    pub static ref DEFAULT_LIBRARY_MANAGER: LibraryManager = {
        let mut lm = LibraryManager::new();
        lm.library = DEFAULT_LIBRARY.clone();
        lm
    };
}

impl Default for LibraryManager {
    fn default() -> Self {
        Self {
            library: DEFAULT_LIBRARY.clone(),
        }
    }
}

/// Utility function to parse wezterm colorschemes.
fn parse_wezterm_colorscheme(content: &str) -> Result<Library, &'static str> {
    serde_json::from_str::<WezTermColorscheme>(content).map_or(
        Err("failed to parse wezterm colorscheme"),
        |wezterm_cs| {
            let mut library = Library::new();
            let mut wezterm = ColorScheme::new();

            for (name, palette) in wezterm_cs {
                let mut result = Palette::new();

                palette.into_iter().enumerate().for_each(|(i, color)| {
                    let color = color.trim_start_matches('#');
                    if let Ok(color) = u32::from_str_radix(color, 16) {
                        let color = Color::new(format!("color{i}"), color);
                        result.insert(color.name.clone(), color);
                    }
                });
                wezterm.insert(name.clone(), Flavor::new(name, result));
            }

            library.insert("wezterm".to_string(), wezterm);
            Ok(library)
        },
    )
}

#[cfg(test)]
mod tests {
    use std::fs::File;
    use std::io::prelude::*;

    use crate::{parse_wezterm_colorscheme, LibraryManager};

    #[test]
    fn test_parse_colorscheme() {
        let mut file = File::open("palettes/catppuccin.json").expect("file not found");
        let mut contents = String::new();
        file.read_to_string(&mut contents)
            .expect("something went wrong reading the file");

        let mut library = LibraryManager::new();
        library.add_colorscheme("catpuccin", &contents).unwrap();
        println!("{library:?}");
    }

    #[test]
    fn test_parse_wezterm_colorscheme() {
        let data = parse_wezterm_colorscheme(include_str!("../vendor/wezterm.json")).unwrap();
        let count = data.len();
        for (name, flavor) in data {
            println!("{}: {}", name, flavor.len());
        }
        println!("{count} colorschemes parsed");
    }

    #[test]
    fn get_colorscheme() {
        let library = LibraryManager::new();
        let cs = library
            .library
            .get("wezterm")
            .unwrap()
            .get("Catppuccin Mocha")
            .unwrap();
        println!("{cs:?}");
    }
}
