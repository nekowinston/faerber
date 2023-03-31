pub type ColorScheme = HashMap<String, Palette>;
pub type Library = HashMap<String, ColorScheme>;
pub type Palette = HashMap<String, u32>;
use faerber_lib::custom_lab::Lab;
use serde_json::Value;
use std::collections::HashMap;

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

pub fn parse_colorscheme(json: Value) -> ColorScheme {
    let mut color_scheme: ColorScheme = HashMap::new();

    json.as_object().unwrap().iter().for_each(|(k, v)| {
        let palette: Palette = v
            .as_object()
            .unwrap()
            .iter()
            .map(|(k, v)| {
                let hex = v.as_str().unwrap().trim_start_matches('#');
                (k.to_string(), u32::from_str_radix(hex, 16).unwrap())
            })
            .collect();

        color_scheme.insert(k.to_string().replace(' ', "_").to_lowercase(), palette);
    });
    color_scheme
}
