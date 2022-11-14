mod library;

use crate::library::{get_labs, Palette};
use clap::{arg, command, value_parser, Arg, ArgAction, ValueEnum};
use faerber::DEMethod;
use image::{EncodableLayout, RgbaImage};
use library::LIBRARY;
use std::path::Path;
use std::path::PathBuf;

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, ValueEnum)]
enum CliDeltaMethods {
    De76,
    De94t,
    De94g,
    De2000,
}

impl Into<DEMethod> for CliDeltaMethods {
    fn into(self) -> DEMethod {
        match self {
            CliDeltaMethods::De76 => DEMethod::DE1976,
            CliDeltaMethods::De94t => DEMethod::DE1994T,
            CliDeltaMethods::De94g => DEMethod::DE1994T,
            CliDeltaMethods::De2000 => DEMethod::DE2000,
        }
    }
}

fn main() {
    let matches = command!()
        .arg(
            arg!([input] "Input file")
                .required(true)
                .value_parser(value_parser!(PathBuf)),
        )
        .arg(
            arg!([output] "Output file")
                .required(true)
                .value_parser(value_parser!(PathBuf)),
        )
        .arg(
            Arg::new("method")
                .short('m')
                .long("method")
                .value_parser(value_parser!(CliDeltaMethods))
                .default_value("de2000"),
        )
        .arg(
            Arg::new("palette")
                .short('p')
                .long("palette")
                .value_parser(value_parser!(String))
                .default_value("catppuccin"),
        )
        .arg(
            Arg::new("flavour")
                .short('f')
                .long("flavour")
                .default_value("mocha"),
        )
        .arg(
            Arg::new("verbose")
                .short('v')
                .long("verbose")
                .action(ArgAction::Count),
        )
        .get_matches();

    let input = matches.get_one::<PathBuf>("input").expect("required");
    let output = matches.get_one::<PathBuf>("output").expect("required");
    let method: DEMethod = matches
        .get_one::<CliDeltaMethods>("method")
        .expect("required")
        .to_owned()
        .into();
    let palette = matches.get_one::<String>("palette").expect("required");
    let flavour = matches.get_one::<String>("flavour").expect("required");

    let file_path = Path::new(input);
    println!("Reading image from {:?}", file_path);
    let file_ext = file_path.extension().unwrap().to_str().unwrap();

    let colorscheme = LIBRARY.get(palette).unwrap_or_else(|| {
        eprintln!("Could not find palette: {}", palette);
        eprintln!("Available palettes: {:?}", LIBRARY.keys());
        std::process::exit(1);
    });

    let palette: Palette = if flavour.is_empty() {
        colorscheme.values().next().unwrap().clone()
    } else {
        colorscheme
            .get(flavour)
            .unwrap_or_else(|| {
                eprintln!("Could not find flavour: {}", flavour);
                eprintln!("Available flavours: {:?}", colorscheme.keys());
                std::process::exit(1);
            })
            .clone()
    };

    let img: RgbaImage = match image::open(input) {
        Ok(img) => img.to_rgba8(),
        Err(e) => {
            eprintln!("Could not open image: {}", e);
            std::process::exit(1);
        }
    };

    let width = img.width();
    let height = img.height();
    let result = faerber::convert(img, method.to_owned(), &get_labs(palette));

    match image::save_buffer(
        output,
        result.as_bytes(),
        width,
        height,
        image::ColorType::Rgba8,
    ) {
        Ok(_) => std::process::exit(0),
        Err(e) => eprintln!("Could not save image: {}", e),
    };
}
