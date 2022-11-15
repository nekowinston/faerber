use serde_json::Value;
use std::collections::HashMap;

pub type Palette = HashMap<String, u32>;
pub type ColorScheme = HashMap<String, Palette>;
pub type Library = HashMap<String, ColorScheme>;
use faerber::custom_lab::Lab;

lazy_static::lazy_static! {
    pub static ref LIBRARY: Library = {
        let mut library: Library = HashMap::new();

        let paths = std::fs::read_dir("palettes").unwrap();

        for path in paths {
            let path = path.unwrap().path();
            let file = std::fs::read_to_string(&path).unwrap();
            let name = path.file_stem().unwrap().to_str().unwrap().to_string();

            let json: Value = serde_json::from_str(&file).unwrap();
            let mut color_scheme: ColorScheme = HashMap::new();

            json.as_object().unwrap().iter().for_each(|(k, v)| {
                let palette: Palette = v.as_object().unwrap().iter().map(|(k, v)| {
                    let hex = v.as_str().unwrap().trim_start_matches('#');
                    (k.to_string(), u32::from_str_radix(hex, 16).unwrap())
                }).collect();

                color_scheme.insert(k.to_string().replace(" ", "_").to_lowercase(), palette);
            });

            library.insert(name.to_string().replace(" ", "_").to_lowercase(), color_scheme);
        }

        return library;
    };
}

pub fn get_labs(palette: Palette) -> Vec<Lab> {
    return palette
        .values()
        .map(|c| {
            Lab::from_rgb(&[
                ((c >> 16) & 0xFF) as u8,
                ((c >> 8) & 0xFF) as u8,
                (c & 0xFF) as u8,
            ])
        })
        .collect();
}
