mod colour_library;

use crate::colour_library::Library;
use clap::{Parser, ValueEnum};
use faerber::DEMethod;
use image::{EncodableLayout, RgbaImage};

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
    #[clap(long, short = 'p', default_value = "Catppuccin")]
    palette: String,

    /// which flavour of the palette to use, defaults to the first entry
    #[clap(long, short = 'f', default_value = "")]
    flavour: String,
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

    let img: RgbaImage = image::open(args.input)
        .expect("Should be able to open image")
        .to_rgba8();

    let library: Library = Library::default();

    let palette = library
        .get_palette(&args.palette)
        .expect("Should be able to get palette");

    let flavour;
    // get the first flavour if no flavour is specified
    if args.flavour.is_empty() {
        flavour = palette.flavours[0].clone();
    } else {
        flavour = palette
            .get_flavour(&args.flavour)
            .expect("Should be able to get flavour")
            .clone();
    }

    let width = img.width();
    let height = img.height();
    let result = faerber::convert(img, method, &flavour.get_labs());

    image::save_buffer(
        args.output,
        result.as_bytes(),
        width,
        height,
        image::ColorType::Rgba8,
    )
    .expect("Should be able to save image");
}
