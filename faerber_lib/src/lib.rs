#![warn(
    // clippy::cargo,
    clippy::complexity,
    clippy::nursery,
    clippy::pedantic,
    clippy::perf,
    clippy::style,
    // clippy::unwrap_used,
    // clippy::expect_used,
)]
#![allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]

pub mod custom_lab;

pub use crate::custom_lab::Lab;
use base64::{engine::general_purpose::STANDARD as base64, Engine as _};
use css_color::Srgb;
pub use deltae::DEMethod;
use deltae::DeltaE;
use dither::color::palette;
use dither::prelude::*;
use image::buffer::Pixels;
use image::{Rgba, RgbaImage};
use quick_xml::events::Event;
use quick_xml::reader::Reader;
use quick_xml::writer::Writer;
use quick_xml::{events::attributes::Attribute, name::QName};
use rayon::prelude::*;
use std::borrow::Cow;
use std::io::Cursor;

#[derive(Clone, Copy, Eq, PartialEq, PartialOrd)]
pub enum ConversionMethod {
    De1976,
    De1994T,
    De1994G,
    De2000,
    DitherFloydSteinberg,
    DitherAtkinson,
    DitherStucki,
    DitherBurkes,
    DitherJarvisJudiceNinke,
    DitherSierra3,
}

impl TryFrom<ConversionMethod> for DEMethod {
    type Error = &'static str;

    fn try_from(val: ConversionMethod) -> Result<Self, &'static str> {
        match val {
            ConversionMethod::De1976 => Ok(Self::DE1976),
            ConversionMethod::De1994G => Ok(Self::DE1994G),
            ConversionMethod::De1994T => Ok(Self::DE1994T),
            ConversionMethod::De2000 => Ok(Self::DE2000),
            _ => Err("Invalid conversion method"),
        }
    }
}

impl TryFrom<ConversionMethod> for ditherer::Ditherer<'static> {
    type Error = &'static str;

    fn try_from(value: ConversionMethod) -> Result<Self, Self::Error> {
        match value {
            ConversionMethod::DitherAtkinson => Ok(ditherer::ATKINSON),
            ConversionMethod::DitherBurkes => Ok(ditherer::BURKES),
            ConversionMethod::DitherFloydSteinberg => Ok(ditherer::FLOYD_STEINBERG),
            ConversionMethod::DitherJarvisJudiceNinke => Ok(ditherer::JARVIS_JUDICE_NINKE),
            ConversionMethod::DitherSierra3 => Ok(ditherer::SIERRA_3),
            ConversionMethod::DitherStucki => Ok(ditherer::STUCKI),
            _ => Err("Invalid conversion method"),
        }
    }
}

impl std::str::FromStr for ConversionMethod {
    type Err = &'static str;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        Ok(match s.to_ascii_lowercase().as_ref() {
            "de1976" => Self::De1976,
            "de1994g" => Self::De1994G,
            "de1994t" => Self::De1994T,
            "de2000" => Self::De2000,
            "dither_atkinson" => Self::DitherAtkinson,
            "dither_burkes" => Self::DitherBurkes,
            "dither_floydsteinberg" => Self::DitherFloydSteinberg,
            "dither_jarvisjudiceninke" => Self::DitherJarvisJudiceNinke,
            "dither_sierra3" => Self::DitherSierra3,
            "dither_stucki" => Self::DitherStucki,
            _ => return Err("Invalid conversion method"),
        })
    }
}

// used for the WASM library to convert the HEX colors to CIELAB
#[must_use]
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
pub fn parse_delta_e_method(method: String) -> ConversionMethod {
    return method.parse().unwrap();
}

#[must_use]
pub fn convert_vector(
    source: &str,
    convert_method: DEMethod,
    labs: &Vec<Lab>,
) -> Result<String, &'static str> {
    let mut reader = Reader::from_str(source);
    reader.trim_text(true);
    let mut writer = Writer::new(Cursor::new(Vec::new()));

    loop {
        let event = reader.read_event();
        match &event {
            Ok(Event::Start(e) | Event::Empty(e)) => {
                let mut elem = e.to_owned();
                let mod_attr = e.attributes().map(|attr| match attr.key {
                    QName(
                        b"fill" | b"stroke" | b"stop-color" | b"flood-color" | b"lighting-color",
                    ) => {
                        if attr?.value.starts_with(b"url(") || attr?.value == b"none".as_slice() {
                            return attr;
                        }

                        let value = String::from_utf8(attr?.value.to_vec()).unwrap();
                        let p = value.parse::<Srgb>().unwrap();
                        let lab = Lab::from_rgb(&[
                            (p.red * 255.0) as u8,
                            (p.green * 255.0) as u8,
                            (p.blue * 255.0) as u8,
                        ]);
                        let converted = convert_color(convert_method, labs, &lab);

                        let new_color = if converted[3] == 255 {
                            format!(
                                "#{:02x}{:02x}{:02x}",
                                converted[0], converted[1], converted[2]
                            )
                        } else {
                            format!(
                                "#{:02x}{:02x}{:02x}{:02x}",
                                converted[0], converted[1], converted[2], converted[3]
                            )
                        };

                        Ok(Attribute {
                            key: attr?.key,
                            value: Cow::Owned(new_color.as_bytes().to_vec()),
                        })
                    }
                    QName(b"href") => {
                        let value = String::from_utf8(attr?.value.to_vec()).unwrap();
                        if value.starts_with("data:image/") {
                            let data = value.split(',').collect::<Vec<&str>>()[1];
                            let decoded = base64.decode(data).unwrap();
                            let image: RgbaImage =
                                image::load_from_memory(&decoded).unwrap().to_rgba8();
                            let converted = convert_naive(&image, ConversionMethod::De2000, labs);
                            let mut buffer = Cursor::new(Vec::new());
                            _ = image::write_buffer_with_format(
                                &mut buffer,
                                &converted.expect(""),
                                image.width(),
                                image.height(),
                                image::ColorType::Rgba8,
                                image::ImageFormat::Png,
                            );
                            let encoded = base64.encode(buffer.get_ref());
                            let href = format!("data:image/png;base64,{encoded}");

                            Ok(Attribute {
                                key: attr?.key,
                                value: Cow::Owned(href.as_bytes().to_vec()),
                            })
                        } else {
                            attr
                        }
                    }
                    _ => attr,
                });
                e.into_owned().clear_attributes();
                e.into_owned().extend_attributes(mod_attr);
                match &event {
                    Ok(Event::Empty(..)) => {
                        writer.write_event(Event::Empty(elem)).unwrap();
                    }
                    Ok(Event::Start(..)) => {
                        writer.write_event(Event::Start(elem)).unwrap();
                    }
                    _ => unreachable!(),
                }
            }
            Ok(Event::Eof) => break,
            // we can either move or borrow the event to write, depending on your use-case
            Ok(e) => assert!({
                println!("closing {e:?}");
                writer.write_event(e).is_ok()
            }),
            Err(e) => panic!("Error at position {}: {:?}", reader.buffer_position(), e),
        }
    }

    Ok(String::from_utf8(writer.into_inner().into_inner()).unwrap())
}

#[must_use]
pub fn convert_naive(
    img: &RgbaImage,
    convert_method: ConversionMethod,
    labs: &Vec<Lab>,
) -> Result<Vec<u8>, &'static str> {
    // convert the RGBA pixels in the image to LAB values
    let img_labs = rgba_pixels_to_labs(img.pixels());

    // loop over each LAB in the LAB-converted image:
    // benchmarks have shown that only DeltaE 2000 benefits from parallel processing with rayon

    return match convert_method {
        ConversionMethod::De1976 | ConversionMethod::De1994T | ConversionMethod::De1994G => {
            Ok(img_labs
                .iter()
                .map(|lab| convert_color(convert_method.try_into(), labs, lab))
                .collect())
        }
        ConversionMethod::De2000 => {
            let method = convert_method.try_into()?;
            img_labs
                .par_iter()
                .flat_map(|lab| convert_color(method, labs, lab))
                .collect()
        }
        ConversionMethod::DitherAtkinson
        | ConversionMethod::DitherBurkes
        | ConversionMethod::DitherFloydSteinberg
        | ConversionMethod::DitherJarvisJudiceNinke
        | ConversionMethod::DitherSierra3
        | ConversionMethod::DitherStucki => {
            let buf = Img::from_raw_buf(img.to_vec(), img.width()).convert_with(f64::from);
            let quantize = dither::create_quantize_n_bits_func(3).expect("stuff");

            Ok(Into::<Ditherer<'static>>::into(convert_method)
                .dither(buf, quantize)
                .convert_with(clamp_f64_to_u8)
                .into_vec())
        }
    };
}

pub fn convert_dither(
    img: &RgbaImage,
    convert_method: ConversionMethod,
    palette: Vec<u32>,
) -> Vec<u8> {
    let buf: Img<RGB<f64>> = Img::<RGB<u8>>::from_raw_buf(
        img.to_vec()
            .iter()
            .map(|a| RGB {
                0: (16 >> a) & 0xFF as u8,
                1: (8 >> a) & 0xF as u8,
                2: (8 >> a) & 0xFF as u8,
            })
            .collect(),
        img.width(),
    )
    .convert_with(|rgb| rgb.convert_with(f64::from));
    let quantize = dither::create_quantize_n_bits_func(3).expect("stuff");
    let colors = palette.into_iter().map(RGB::from_hex).collect();

    let x = Into::<Ditherer<'static>>::into(convert_method)
        .dither(buf, palette::quantize(colors))
        .convert_with(|rgb| rgb.convert_with(clamp_f64_to_u8))
        .into_vec();

    todo!()
}

#[must_use]
pub fn rgba_pixels_to_labs(img_pixels: Pixels<Rgba<u8>>) -> Vec<Lab> {
    img_pixels.map(|pixel| Lab::from_rgba(&pixel.0)).collect()
}

#[must_use]
pub fn convert_color(convert_method: DEMethod, palette: &Vec<Lab>, lab: &Lab) -> [u8; 4] {
    // keep track of the closest color
    let mut closest_color: Lab = custom_lab::Lab::default();
    // keep track of the closest distance measured, initially set as high as possible
    let mut closest_distance: f32 = f32::MAX;

    // loop over each LAB in the user's palette, and find the closest color
    for color in palette {
        let delta = DeltaE::new(*lab, *color, convert_method);

        if delta.value() < &closest_distance {
            closest_color = *color;
            closest_color.alpha = lab.alpha;
            closest_distance = *delta.value();
        }
    }

    // convert the LAB back to RGBA
    closest_color.to_rgba()
}
