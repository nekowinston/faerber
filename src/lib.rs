pub mod custom_lab;

pub use crate::custom_lab::Lab;
use base64::{decode, encode};
use css_color::Srgb;
pub use deltae::DEMethod;
use deltae::DeltaE;
use image::buffer::Pixels;
use image::{Rgba, RgbaImage};
use quick_xml::events::Event;
use quick_xml::reader::Reader;
use quick_xml::writer::Writer;
use quick_xml::{events::attributes::Attribute, name::QName};
use rayon::prelude::*;
use std::borrow::Cow;
use std::io::Cursor;

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

pub fn convert_vector(source: &str, convert_method: DEMethod, labs: &Vec<Lab>) -> String {
    let mut reader = Reader::from_str(source);
    reader.trim_text(true);
    let mut writer = Writer::new(Cursor::new(Vec::new()));

    loop {
        let event = reader.read_event();
        match &event {
            Ok(Event::Start(e)) | Ok(Event::Empty(e)) => {
                let mut elem = e.to_owned();
                let mod_attr = e.attributes().map(|attr| {
                    let attr = attr.unwrap();
                    match attr.key {
                        QName(b"fill")
                        | QName(b"stroke")
                        | QName(b"stop-color")
                        | QName(b"flood-color")
                        | QName(b"lighting-color") => {
                            let value = String::from_utf8(attr.value.to_vec()).unwrap();
                            let p = value.parse::<Srgb>().unwrap();
                            let lab = Lab::from_rgb(&[
                                (p.red * 255.0) as u8,
                                (p.green * 255.0) as u8,
                                (p.blue * 255.0) as u8,
                            ]);
                            let converted = convert_color(convert_method, labs, &lab);
                            let rgba_color = format!(
                                "rgba({}, {}, {}, {})",
                                converted[0], converted[1], converted[2], converted[3]
                            );
                            Attribute {
                                key: attr.key,
                                value: Cow::Owned(rgba_color.as_bytes().to_vec()),
                            }
                        }
                        QName(b"href") => {
                            let value = String::from_utf8(attr.value.to_vec()).unwrap();
                            if value.starts_with("data:image/") {
                                let data = value.split(",").collect::<Vec<&str>>()[1];
                                let decoded = decode(data).unwrap();
                                let image: RgbaImage =
                                    image::load_from_memory(&decoded).unwrap().to_rgba8();
                                let converted = convert(image.clone(), convert_method, labs);
                                let mut buffer = Cursor::new(Vec::new());
                                let _ = image::write_buffer_with_format(
                                    &mut buffer,
                                    &converted,
                                    image.width(),
                                    image.height(),
                                    image::ColorType::Rgba8,
                                    image::ImageFormat::Png,
                                );
                                let encoded = encode(buffer.get_ref());
                                let href = format!("data:image/png;base64,{}", encoded);
                                Attribute {
                                    key: attr.key,
                                    value: Cow::Owned(href.as_bytes().to_vec()),
                                }
                            } else {
                                attr
                            }
                        }
                        _ => attr,
                    }
                });
                elem.clear_attributes();
                elem.extend_attributes(mod_attr);
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
                println!("closing {:?}", e);
                writer.write_event(e).is_ok()
            }),
            Err(e) => panic!("Error at position {}: {:?}", reader.buffer_position(), e),
        }
    }
    let result = writer.into_inner().into_inner();
    String::from_utf8(result).unwrap()
}

pub fn convert(img: RgbaImage, convert_method: DEMethod, labs: &Vec<Lab>) -> Vec<u8> {
    // convert the RGBA pixels in the image to LAB values
    let img_labs = rgba_pixels_to_labs(img.pixels());

    // loop over each LAB in the LAB-converted image:
    // benchmarks have shown that only DeltaE 2000 benefits from parallel processing with rayon
    return if convert_method != DEMethod::DE2000 {
        img_labs
            .iter()
            .flat_map(|lab| convert_color(convert_method, labs, lab))
            .collect()
    } else {
        img_labs
            .par_iter()
            .flat_map(|lab| convert_color(convert_method, labs, lab))
            .collect()
    };
}

pub fn rgba_pixels_to_labs(img_pixels: Pixels<Rgba<u8>>) -> Vec<Lab> {
    img_pixels.map(|pixel| Lab::from_rgba(&pixel.0)).collect()
}

pub fn convert_color(convert_method: DEMethod, palette: &Vec<Lab>, lab: &Lab) -> [u8; 4] {
    // keep track of the closest color
    let mut closest_color: Lab = Default::default();
    // keep track of the closest distance measured, initially set as high as possible
    let mut closest_distance: f32 = f32::MAX;

    // loop over each LAB in the user's palette, and find the closest color
    for color in palette {
        let delta = DeltaE::new(*lab, *color, convert_method);

        if delta.value() < &closest_distance {
            closest_color = *color;
            closest_color.alpha = lab.alpha;
            closest_distance = *delta.value()
        }
    }

    // convert the LAB back to RGBA
    closest_color.to_rgba()
}
