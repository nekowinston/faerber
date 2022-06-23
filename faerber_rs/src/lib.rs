#![allow(unused)]

pub mod custom_lab;

use crate::custom_lab::Lab;
use clap::Parser;
use deltae::{DEMethod, DeltaE};
use image::buffer::Pixels;
use image::io::Reader as ImageReader;
use image::{EncodableLayout, ImageBuffer, RgbImage, Rgba, RgbaImage};
use rayon::prelude::*;


/*
fn main() {
    //run palletize on the wallpaper.png image
    palettize("wallpaper.jpg", "mocha", "result.png");
}
*/
pub fn palettize(opt_path: Option<&str>, palette: &str, output: &str) {
    //let args = Args::parse();
    let image_path = opt_path.unwrap();
    let img: RgbaImage = image::open(image_path)
        .expect("Should be able to open image")
        .to_rgba8();

    let latte: &[u32] = &[
        0xF5E0DC, 0xF2CDCD, 0xF5C2E7, 0xCBA6F7, 0xF38BA8, 0xEBA0AC, 0xFAB387, 0xF9E2AF, 0xA6E3A1,
        0x94E2D5, 0x89DCEB, 0x74C7EC, 0x89B4FA, 0xB4BEFE, 0xCDD6F4, 0xBAC2DE, 0xA6ADC8, 0x9399B2,
        0x7F849C, 0x6C7086, 0x585B70, 0x45475A, 0x313244, 0x1E1E2E, 0x181825, 0x11111B,
    ];
    let frappe: &[u32] = &[
        0xF5E0DC, 0xF2CDCD, 0xF5C2E7, 0xCBA6F7, 0xF38BA8, 0xEBA0AC, 0xFAB387, 0xF9E2AF, 0xA6E3A1,
        0x94E2D5, 0x89DCEB, 0x74C7EC, 0x89B4FA, 0xB4BEFE, 0xCDD6F4, 0xBAC2DE, 0xA6ADC8, 0x9399B2,
        0x7F849C, 0x6C7086, 0x585B70, 0x45475A, 0x313244, 0x1E1E2E, 0x181825, 0x11111B,
    ];
    let macchiato: &[u32] = &[
        0xF5E0DC, 0xF2CDCD, 0xF5C2E7, 0xCBA6F7, 0xF38BA8, 0xEBA0AC, 0xFAB387, 0xF9E2AF, 0xA6E3A1,
        0x94E2D5, 0x89DCEB, 0x74C7EC, 0x89B4FA, 0xB4BEFE, 0xCDD6F4, 0xBAC2DE, 0xA6ADC8, 0x9399B2,
        0x7F849C, 0x6C7086, 0x585B70, 0x45475A, 0x313244, 0x1E1E2E, 0x181825, 0x11111B,
    ];
    let mocha: &[u32] = &[
        0xF5E0DC, 0xF2CDCD, 0xF5C2E7, 0xCBA6F7, 0xF38BA8, 0xEBA0AC, 0xFAB387, 0xF9E2AF, 0xA6E3A1,
        0x94E2D5, 0x89DCEB, 0x74C7EC, 0x89B4FA, 0xB4BEFE, 0xCDD6F4, 0xBAC2DE, 0xA6ADC8, 0x9399B2,
        0x7F849C, 0x6C7086, 0x585B70, 0x45475A, 0x313244, 0x1E1E2E, 0x181825, 0x11111B,
    ];
    let labs = convert_palette_to_lab(latte);

    let width = img.width();
    let height = img.height();
    let result = convert(img, DEMethod::DE2000, &labs);

    image::save_buffer(
        output,
        result.clone().as_bytes(),
        width,
        height,
        image::ColorType::Rgba8,
    );
}

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

pub fn parse_delta_e_method(method: String) -> DEMethod {
    return match method.as_str() {
        "76" => deltae::DE1976,
        "94t" => deltae::DE1976,
        "94g" => deltae::DE1976,
        "2000" => deltae::DE1976,
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
            .map(|lab| convert_loop(convert_method, labs, lab))
            .flatten()
            .collect()
    } else {
        img_labs
            .par_iter()
            .map(|lab| convert_loop(convert_method, labs, lab))
            .flatten()
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
        let delta = DeltaE::new(lab.clone(), color.clone(), convert_method);

        if delta.value() < &closest_distance {
            closest_color = color.clone();
            closest_distance = delta.value().clone()
        }
    }

    // convert the LAB back to RGBA
    closest_color.to_rgba()
}
