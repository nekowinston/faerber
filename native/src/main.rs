mod library;

use crate::library::{get_labs, Palette};
use clap::{Parser, ValueEnum};
use faerber::DEMethod;
use image::{EncodableLayout, RgbaImage};
use library::LIBRARY;
use std::path::Path;

#[derive(ValueEnum, Clone)]
enum CliDeltaMethods {
    De76,
    De94t,
    De94g,
    De2000,
}

#[derive(Parser)]
#[clap(
    author = "farbenfroh.io",
    version,
    about = "Match images to your favourite colour schemes!"
)]
struct Args {
    #[clap(value_parser, value_hint = clap::ValueHint::FilePath, name="input")]
    input: String,

    #[clap(value_parser, value_hint = clap::ValueHint::FilePath, name="output")]
    output: String,

    /// colour comparison algorithm
    #[clap(long, short = 'm', value_enum, default_value = "de2000")]
    method: CliDeltaMethods,

    /// which palette to use
    #[clap(long, short = 'p', default_value = "catppuccin")]
    palette: String,

    /// which flavour of the palette to use, defaults to the first entry
    #[clap(long, short = 'f', default_value = "")]
    flavour: String,

    #[clap(long, short, action)]
    verbose: bool,
}

fn parse_de_method(method: CliDeltaMethods) -> DEMethod {
    match method {
        CliDeltaMethods::De76 => DEMethod::DE1976,
        CliDeltaMethods::De94t => DEMethod::DE1994T,
        CliDeltaMethods::De94g => DEMethod::DE1994G,
        CliDeltaMethods::De2000 => DEMethod::DE2000,
    }
}

fn main() {
    let args = Args::parse();

    let method = parse_de_method(args.method);

    let file_path = Path::new(&args.input);
    println!("Reading image from {:?}", file_path);
    let file_ext = file_path.extension().unwrap().to_str().unwrap();

    let colorscheme = LIBRARY.get(&args.palette).unwrap_or_else(|| {
        eprintln!("Could not find palette: {}", args.palette);
        eprintln!("Available palettes: {:?}", LIBRARY.keys());
        std::process::exit(1);
    });

    let palette: Palette = if args.flavour.is_empty() {
        colorscheme.values().next().unwrap().clone()
    } else {
        colorscheme
            .get(&args.flavour)
            .unwrap_or_else(|| {
                eprintln!("Could not find flavour: {}", args.flavour);
                eprintln!("Available flavours: {:?}", colorscheme.keys());
                std::process::exit(1);
            })
            .clone()
    };

    let img: RgbaImage = match image::open(args.input) {
        Ok(img) => img.to_rgba8(),
        Err(e) => {
            eprintln!("Could not open image: {}", e);
            std::process::exit(1);
        }
    };

    let width = img.width();
    let height = img.height();
    let result = faerber::convert(img, method, &get_labs(palette));

    match image::save_buffer(
        args.output,
        result.as_bytes(),
        width,
        height,
        image::ColorType::Rgba8,
    ) {
        Ok(_) => std::process::exit(0),
        Err(e) => eprintln!("Could not save image: {}", e),
    };
}
