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
    pub fn new(name: String, value: u32) -> Self {
        Self {
            name,
            value,
            enabled: true,
        }
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
    pub fn new(name: String, palette: Palette) -> Self {
        Self {
            name,
            palette,
            enabled: true,
        }
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
    #[error("Failed to parse color: {0}")]
    ParseColorError(String),
}

impl LibraryManager {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn add_colorscheme(&mut self, name: &str, cs: &str) -> Result<ColorScheme, &'static str> {
        let colorscheme = Self::parse_colorscheme(name, cs)?;
        self.library.insert(name.to_string(), colorscheme.clone());
        Ok(colorscheme)
    }
    pub fn parse_colorscheme(name: &str, cs: &str) -> Result<ColorScheme, &'static str> {
        if let Ok(saved_cs) = serde_json::from_str::<SavedColorscheme>(cs) {
            let mut colorscheme = ColorScheme::new();

            saved_cs.into_iter().for_each(|(flavor_name, flavor)| {
                let mut flavor_palette = Palette::new();

                flavor.into_iter().for_each(|(name, value)| {
                    let value = value.trim_start_matches('#');
                    let color = Color::new(name, u32::from_str_radix(value, 16).unwrap());
                    flavor_palette.insert(color.name.clone(), color);
                });

                colorscheme.insert(name.to_owned(), Flavor::new(flavor_name, flavor_palette));
            });

            Ok(colorscheme)
        } else {
            Err("failed to parse colorscheme")
        }
    }
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
}

lazy_static! {
    pub static ref DEFAULT_LIBRARY: Library = {
        let vendored_colorschemes =
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
        LibraryManager {
            library: DEFAULT_LIBRARY.clone(),
        }
    }
}

/// Utility function to parse wezterm colorschemes.
fn parse_wezterm_colorscheme(content: &str) -> Result<Library, &'static str> {
    if let Ok(wezterm_cs) = serde_json::from_str::<WezTermColorscheme>(content) {
        let mut library = Library::new();
        let mut wezterm = ColorScheme::new();

        wezterm_cs.into_iter().for_each(|(name, palette)| {
            let mut result = Palette::new();

            palette.into_iter().enumerate().for_each(|(i, color)| {
                let color = color.trim_start_matches('#');
                let color =
                    Color::new(format!("color{i}"), u32::from_str_radix(color, 16).unwrap());
                result.insert(color.name.clone(), color);
            });
            wezterm.insert(name.clone(), Flavor::new(name, result));
        });

        library.insert("wezterm".to_string(), wezterm);
        Ok(library)
    } else {
        Err("failed to parse wezterm colorscheme")
    }
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
        println!("{:?}", library);
    }

    #[test]
    fn test_parse_wezterm_colorscheme() {
        let data = parse_wezterm_colorscheme(include_str!("../vendor/wezterm.json")).unwrap();
        let count = data.len();
        data.into_iter().for_each(|(name, flavor)| {
            println!("{}: {}", name, flavor.len());
        });
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
        println!("{:?}", cs);
    }
}
