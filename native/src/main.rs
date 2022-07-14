mod colour_library;

use crate::colour_library::Library;
use clap::{Parser, ValueEnum};
use colour_library::Flavour;
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

    let img: RgbaImage = match image::open(args.input) {
        Ok(img) => img.to_rgba8(),
        Err(e) => {
            eprintln!("Could not open image: {}", e);
            std::process::exit(1);
        }
    };

    let library: Library = Library::default();

    // handle missing palettes
    let palette = match library.get_palette(&args.palette) {
        Some(palette) => palette,
        None => {
            let available_palettes = library
                .palettes
                .iter()
                .map(|p| p.name.clone())
                .collect::<Vec<_>>();
            eprintln!("Could not find palette {}", args.palette);
            eprintln!("Available palettes: {:?}", available_palettes);
            std::process::exit(1);
        }
    };
    if args.verbose {
        println!("Using palette {}", palette.name);
    }

    // get the first flavour if no flavour is specified
    let flavour: Flavour = if args.flavour.is_empty() {
        palette.flavours[0].clone()
    } else {
        // handle wrong flavour names
        match palette.get_flavour(&args.flavour) {
            Some(p) => p.clone(),
            None => {
                let available_flavours: Vec<&str> =
                    palette.flavours.iter().map(|f| f.name.as_str()).collect();

                eprintln!("Could not find flavour {}", args.flavour);
                eprintln!("Available flavours: {:?}", available_flavours);
                std::process::exit(1);
            }
        }
    };
    if args.verbose {
        println!("Using flavour: {}", flavour.name);
    }

    let width = img.width();
    let height = img.height();
    let result = faerber::convert(img, method, &flavour.get_labs());

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
