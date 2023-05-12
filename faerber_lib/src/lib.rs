#![warn(
    // clippy::cargo,
    // clippy::pedantic,
    clippy::complexity,
    clippy::expect_used,
    clippy::nursery,
    clippy::perf,
    clippy::style,
    clippy::suspicious,
    clippy::suspicious,
    clippy::unwrap_used,
)]
#![allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]

pub mod custom_lab;

pub use crate::custom_lab::Lab;
use base64::{engine::general_purpose::STANDARD as base64, Engine as _};
use css_color::Srgb;
pub use deltae::DEMethod;
use deltae::DeltaE;
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
use strum::{Display, EnumIter, EnumString};
use thiserror::Error;

#[derive(Clone, Copy, Eq, PartialEq, PartialOrd, EnumIter, EnumString, Display)]
pub enum ConversionMethod {
    #[strum(serialize = "de1976")]
    De1976,
    #[strum(serialize = "de1994g")]
    De1994T,
    #[strum(serialize = "de1994t")]
    De1994G,
    #[strum(serialize = "de2000")]
    De2000,
    #[strum(serialize = "dither_floydsteinberg")]
    DitherFloydSteinberg,
    #[strum(serialize = "dither_atkinson")]
    DitherAtkinson,
    #[strum(serialize = "dither_stucki")]
    DitherStucki,
    #[strum(serialize = "dither_burkes")]
    DitherBurkes,
    #[strum(serialize = "dither_jarvisjudiceninke")]
    DitherJarvisJudiceNinke,
    #[strum(serialize = "dither_sierra3")]
    DitherSierra3,
}

#[derive(Error, Debug)]
pub enum ConversionError {
    #[error("Invalid attribute")]
    AttributeParseError,
    #[error("Error reading XML")]
    XMLReadError,
    #[error("Invalid conversion method")]
    InvalidConversionMethod,
}

impl TryFrom<ConversionMethod> for DEMethod {
    type Error = ConversionError;

    fn try_from(val: ConversionMethod) -> Result<Self, Self::Error> {
        match val {
            ConversionMethod::De1976 => Ok(Self::DE1976),
            ConversionMethod::De1994G => Ok(Self::DE1994G),
            ConversionMethod::De1994T => Ok(Self::DE1994T),
            ConversionMethod::De2000 => Ok(Self::DE2000),
            _ => Err(ConversionError::InvalidConversionMethod),
        }
    }
}

impl TryFrom<ConversionMethod> for Ditherer<'static> {
    type Error = ConversionError;

    fn try_from(value: ConversionMethod) -> Result<Self, Self::Error> {
        match value {
            ConversionMethod::DitherAtkinson => Ok(ditherer::ATKINSON),
            ConversionMethod::DitherBurkes => Ok(ditherer::BURKES),
            ConversionMethod::DitherFloydSteinberg => Ok(ditherer::FLOYD_STEINBERG),
            ConversionMethod::DitherJarvisJudiceNinke => Ok(ditherer::JARVIS_JUDICE_NINKE),
            ConversionMethod::DitherSierra3 => Ok(ditherer::SIERRA_3),
            ConversionMethod::DitherStucki => Ok(ditherer::STUCKI),
            _ => Err(ConversionError::InvalidConversionMethod),
        }
    }
}

// impl std::str::FromStr for ConversionMethod {
//     type Err = &'static str;
//
//     fn from_str(s: &str) -> Result<Self, Self::Err> {
//         Ok(match s.to_ascii_lowercase().as_ref() {
//             "de1976" => Self::De1976,
//             "de1994g" => Self::De1994G,
//             "de1994t" => Self::De1994T,
//             "de2000" => Self::De2000,
//             "dither_atkinson" => Self::DitherAtkinson,
//             "dither_burkes" => Self::DitherBurkes,
//             "dither_floydsteinberg" => Self::DitherFloydSteinberg,
//             "dither_jarvisjudiceninke" => Self::DitherJarvisJudiceNinke,
//             "dither_sierra3" => Self::DitherSierra3,
//             "dither_stucki" => Self::DitherStucki,
//             _ => return Err("Invalid conversion method"),
//         })
//     }
// }

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
pub fn parse_delta_e_method(method: String) -> ConversionMethod {
    return method.parse().unwrap();
}

fn convert_map<'a>(
    convert_method: ConversionMethod,
    labs: &[Lab],
    attr: Result<Attribute<'a>, quick_xml::events::attributes::AttrError>,
) -> Result<Attribute<'a>, Box<dyn std::error::Error>> {
    let attr = attr?;
    Ok(match attr.key {
        QName(b"fill" | b"stroke" | b"stop-color" | b"flood-color" | b"lighting-color") => {
            if attr.value.starts_with(b"url(") || attr.value == b"none".as_slice() {
                return Ok(attr);
            }

            let value = String::from_utf8(attr.value.to_vec())?;
            let p = value
                .parse::<Srgb>()
                .map_err(|_| ConversionError::AttributeParseError)?;

            let lab = Lab::from_rgb(&[
                (p.red * 255.0) as u8,
                (p.green * 255.0) as u8,
                (p.blue * 255.0) as u8,
            ]);
            let converted = convert_color(convert_method, labs, &lab)?;

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

            Attribute {
                key: attr.key,
                value: Cow::Owned(new_color.as_bytes().to_vec()),
            }
        }
        QName(b"href") => {
            let value = String::from_utf8(attr.value.to_vec())?;
            if value.starts_with("data:image/") {
                let data = value.split(',').collect::<Vec<&str>>()[1];
                let decoded = base64.decode(data)?;
                let image: RgbaImage = image::load_from_memory(&decoded)?.to_rgba8();
                let converted = convert_naive(&image, convert_method, labs)?;
                let mut buffer = Cursor::new(Vec::new());
                image::write_buffer_with_format(
                    &mut buffer,
                    &converted,
                    image.width(),
                    image.height(),
                    image::ColorType::Rgba8,
                    image::ImageFormat::Png,
                )
                .ok();
                let encoded = base64.encode(buffer.get_ref());
                let href = format!("data:image/png;base64,{encoded}");
                Attribute {
                    key: attr.key,
                    value: Cow::Owned(href.as_bytes().to_vec()),
                }
            } else {
                attr
            }
        }
        _ => attr,
    })
}

pub fn convert_vector(
    source: &str,
    convert_method: ConversionMethod,
    labs: &[Lab],
) -> Result<String, Box<dyn std::error::Error>> {
    let mut reader = Reader::from_str(source);
    reader.trim_text(true);
    let mut writer = Writer::new(Cursor::new(Vec::new()));

    loop {
        let event = reader.read_event();
        match &event? {
            event @ (Event::Start(e) | Event::Empty(e)) => {
                let mut elem = e.to_owned();
                let mod_attr = e
                    .attributes()
                    .map(|attr| convert_map(convert_method, labs, attr))
                    .collect::<Result<Vec<Attribute>, Box<dyn std::error::Error>>>()?;
                elem.clear_attributes();
                elem.extend_attributes(mod_attr);
                match event {
                    Event::Empty(..) => {
                        writer.write_event(Event::Empty(elem))?;
                    }
                    Event::Start(..) => {
                        writer.write_event(Event::Start(elem))?;
                    }
                    _ => unreachable!(),
                }
            }
            Event::Eof => break,
            // we can either move or borrow the event to write, depending on your use-case
            e => {
                println!("closing {e:?}");
                writer.write_event(e)?;
            }
        }
    }
    let result = writer.into_inner().into_inner();
    Ok(String::from_utf8(result)?)
}

pub fn convert_naive(
    img: &RgbaImage,
    method: ConversionMethod,
    labs: &[Lab],
) -> Result<Vec<u8>, ConversionError> {
    // convert the RGBA pixels in the image to LAB values
    let img_labs = rgba_pixels_to_labs(img.pixels());

    // loop over each LAB in the LAB-converted image:
    // benchmarks have shown that only DeltaE 2000 benefits from parallel processing with rayon
    match method {
        ConversionMethod::De2000 => Ok(img_labs
            .par_iter()
            .map(|lab| convert_color(method, labs, lab))
            .collect::<Result<Vec<_>, ConversionError>>()?
            .into_iter()
            .flatten()
            .collect()),
        ConversionMethod::De1976 | ConversionMethod::De1994G | ConversionMethod::De1994T => {
            Ok(img_labs
                .iter()
                .map(|lab| convert_color(method, labs, lab))
                .collect::<Result<Vec<_>, ConversionError>>()?
                .into_iter()
                .flatten()
                .collect())
        }
        _ => Err(ConversionError::InvalidConversionMethod),
    }
}

pub fn convert_dither(
    img: &RgbaImage,
    convert_method: ConversionMethod,
    palette: &[u32],
) -> Result<Vec<u8>, ConversionError> {
    let buf: Vec<RGB<u8>> = img.pixels().map(|p| RGB(p.0[0], p.0[1], p.0[2])).collect();
    let img: Img<RGB<f64>> = Img::new(buf, img.width())
        .unwrap()
        .convert_with(|rgb| rgb.convert_with(f64::from));

    let ditherer = match convert_method {
        ConversionMethod::DitherFloydSteinberg => Ok(dither::ditherer::FLOYD_STEINBERG),
        ConversionMethod::DitherAtkinson => Ok(dither::ditherer::ATKINSON),
        ConversionMethod::DitherStucki => Ok(dither::ditherer::STUCKI),
        ConversionMethod::DitherBurkes => Ok(dither::ditherer::BURKES),
        ConversionMethod::DitherJarvisJudiceNinke => Ok(dither::ditherer::JARVIS_JUDICE_NINKE),
        ConversionMethod::DitherSierra3 => Ok(dither::ditherer::SIERRA_3),
        _ => Err(ConversionError::InvalidConversionMethod),
    }?;

    let palette = palette
        .iter()
        .map(|color| dither::color::RGB::from_hex(*color))
        .collect();

    let result = ditherer
        .dither(img, dither::color::palette::quantize(palette))
        .convert_with(|rgb| rgb.convert_with(clamp_f64_to_u8));

    Ok(result
        .into_iter()
        .flat_map(|v| [v.0, v.1, v.2, 255])
        .collect())
}

pub fn rgba_pixels_to_labs(img_pixels: Pixels<Rgba<u8>>) -> Vec<Lab> {
    img_pixels.map(|pixel| Lab::from_rgba(&pixel.0)).collect()
}

pub fn convert_color(
    convert_method: ConversionMethod,
    palette: &[Lab],
    lab: &Lab,
) -> Result<[u8; 4], ConversionError> {
    // keep track of the closest color
    let mut closest_color: Lab = Lab::default();
    // keep track of the closest distance measured, initially set as high as possible
    let mut closest_distance: f32 = f32::MAX;

    // loop over each LAB in the user's palette, and find the closest color
    for color in palette {
        let delta = DeltaE::new(*lab, *color, convert_method.try_into()?);

        if delta.value() < &closest_distance {
            closest_color = *color;
            closest_color.alpha = lab.alpha;
            closest_distance = *delta.value();
        }
    }

    // convert the LAB back to RGBA
    Ok(closest_color.to_rgba())
}
