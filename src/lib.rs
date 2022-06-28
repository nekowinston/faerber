pub mod custom_lab;

pub use crate::custom_lab::Lab;
pub use deltae::DEMethod;
use deltae::DeltaE;
use image::buffer::Pixels;
use image::{Rgba, RgbaImage};
use rayon::prelude::*;

// used for the WASM library to convert the HEX colors to CIELAB
pub fn convert_palette_to_lab(palette: &[u32]) -> Vec<Lab> {
    palette
        .iter()
        .map(|color| {
            let r = ((color >> 16) & 0xFF) as u8;
            let g = ((color >> 8) & 0xFF) as u8;
            let b = (color & 0xFF) as u8;
            Lab::from_rgb(&[r, g, b])
        })
        .collect()
}

// used for the WASM library to convert a String to a DeltaE method
#[cfg(target_family = "wasm")]
pub fn parse_delta_e_method(method: String) -> DEMethod {
    return match method.as_str() {
        "76" => deltae::DE1976,
        "94t" => deltae::DE1994T,
        "94g" => deltae::DE1994G,
        "2000" => deltae::DE2000,
        _ => deltae::DE1976,
    };
}

pub fn convert(img: RgbaImage, convert_method: DEMethod, labs: &Vec<Lab>) -> Vec<u8> {
    // convert the RGBA pixels in the image to LAB values
    let img_labs = rgba_pixels_to_labs(img.pixels());

    // loop over each LAB in the LAB-converted image:
    // benchmarks have shown that only DeltaE 2000 benefits from parallel processing with rayon
    return if convert_method != DEMethod::DE2000 {
        img_labs
            .iter()
            .flat_map(|lab| convert_loop(convert_method, labs, lab))
            .collect()
    } else {
        img_labs
            .par_iter()
            .flat_map(|lab| convert_loop(convert_method, labs, lab))
            .collect()
    };
}

pub fn rgba_pixels_to_labs(img_pixels: Pixels<Rgba<u8>>) -> Vec<Lab> {
    img_pixels.map(|pixel| Lab::from_rgba(&pixel.0)).collect()
}

pub fn convert_loop(convert_method: DEMethod, palette: &Vec<Lab>, lab: &Lab) -> [u8; 4] {
    // keep track of the closest color
    let mut closest_color: Lab = Default::default();
    // keep track of the closest distance measured, initially set as high as possible
    let mut closest_distance: f32 = f32::MAX;

    // loop over each LAB in the user's palette, and find the closest color
    for color in palette {
        let delta = DeltaE::new(*lab, *color, convert_method);

        if delta.value() < &closest_distance {
            closest_color = *color;
            closest_distance = *delta.value()
        }
    }

    // convert the LAB back to RGBA
    closest_color.to_rgba()
}
