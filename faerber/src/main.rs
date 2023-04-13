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

use clap::ArgGroup;
use clap::{value_parser, Arg, ArgAction, Command, ValueEnum, ValueHint};
use clap_complete::{generate, Generator, Shell};
use faerber::{get_labs, parse_colorscheme, ColorScheme, Palette, LIBRARY};
use faerber_lib::DEMethod;
use faerber_lib::Lab;
use image::RgbaImage;
use std::fs::{read_to_string, File};
use std::io;
use std::io::{Cursor, Write};
use std::path::Path;
use std::path::PathBuf;

extern crate oxipng;

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, ValueEnum)]
enum CliDeltaMethods {
    De76,
    De94t,
    De94g,
    De2000,
}

impl From<CliDeltaMethods> for DEMethod {
    fn from(val: CliDeltaMethods) -> Self {
        match val {
            CliDeltaMethods::De76 => Self::DE1976,
            CliDeltaMethods::De94t => Self::DE1994T,
            CliDeltaMethods::De94g => Self::DE1994G,
            CliDeltaMethods::De2000 => Self::DE2000,
        }
    }
}

fn build_cli() -> Command {
    let palettes = LIBRARY.keys().map(|s| s.to_lowercase()).collect::<Vec<_>>();
    let flavours = LIBRARY
        .iter()
        .flat_map(|(_k, v)| v.keys().map(|s| s.to_lowercase()).collect::<Vec<_>>())
        .collect::<Vec<_>>();

    Command::new("faerber")
        .subcommand_negates_reqs(true)
        .args([
            Arg::new("input")
                .required(true)
                .value_parser(value_parser!(PathBuf))
                .value_hint(ValueHint::FilePath),
            Arg::new("output")
                .value_parser(value_parser!(String))
                .value_hint(ValueHint::FilePath),
        ])
        .group(ArgGroup::new("palette_flavour"))
        .args([
            Arg::new("palette")
                .short('p')
                .long("palette")
                .value_parser(palettes)
                .default_value("catppuccin"),
            Arg::new("flavour")
                .short('f')
                .long("flavour")
                .value_parser(flavours),
        ])
        .arg(
            Arg::new("method")
                .short('m')
                .long("method")
                .value_parser(value_parser!(CliDeltaMethods))
                .default_value("de2000"),
        )
        .arg(
            Arg::new("verbose")
                .short('v')
                .long("verbose")
                .action(ArgAction::Count),
        )
        .subcommand(
            Command::new("completion")
                .about("Generate shell completion scripts")
                .arg_required_else_help(true)
                .arg(
                    Arg::new("shell")
                        .required(true)
                        .value_parser(value_parser!(Shell)),
                ),
        )
}

fn print_completions<G: Generator>(gen: G, cmd: &mut Command) {
    generate(gen, cmd, cmd.get_name().to_string(), &mut io::stdout());
}

fn slugify(s: &str) -> String {
    "_".to_owned() + &s.to_lowercase().replace([' ', '_'], "_")
}

fn main() {
    let matches = build_cli().get_matches();

    if let Some(completion) = matches.subcommand_matches("completion") {
        let shell = completion.get_one::<Shell>("shell").copied().unwrap();
        let mut cmd = build_cli();
        eprintln!("Generating completion file for {shell}...");
        print_completions(shell, &mut cmd);
        std::process::exit(0);
    }

    let input = matches.get_one::<PathBuf>("input").expect("required");
    let method: DEMethod = (*matches
        .get_one::<CliDeltaMethods>("method")
        .expect("default"))
    .into();
    let palette = matches.get_one::<String>("palette").expect("default");
    let flavour = matches.get_one::<String>("flavour");

    let file_path = Path::new(input);
    println!("Reading image from {}", file_path.display());
    let file_ext = file_path.extension().unwrap().to_str().unwrap();

    let mut custom_colorscheme: ColorScheme = ColorScheme::new();
    let colorscheme = LIBRARY.get(palette).unwrap_or_else(|| {
        let contents = read_to_string(palette).expect("something went wrong reading the file");

        custom_colorscheme = parse_colorscheme(serde_json::from_str(&contents).unwrap());
        &custom_colorscheme
    });

    let output = matches.get_one::<String>("output").map_or_else(
        || {
            let input = input.file_stem().unwrap().to_str().unwrap();
            let flavour = flavour.map_or_else(String::new, |flavour| slugify(flavour));
            let palette = slugify(palette);
            format!("{input}{palette}{flavour}.{file_ext}")
        },
        Clone::clone,
    );

    let labs: Vec<Lab> = flavour.map_or_else(
        || {
            let palette: Palette = colorscheme
                .values()
                .next()
                .expect("palette should have a flavour")
                .clone();
            get_labs(palette)
        },
        |flavour| {
            let mut labs: Vec<Lab> = vec![];
            if colorscheme.contains_key(flavour) {
                labs.append(&mut get_labs(colorscheme.get(flavour).unwrap().clone()));
            } else {
                eprintln!("Could not find flavour: {flavour}");
                eprintln!(
                    "Available flavours: {}",
                    colorscheme
                        .keys()
                        .map(|s| s.to_lowercase())
                        .collect::<Vec<_>>()
                        .join(", ")
                );
                std::process::exit(1);
            }
            labs
        },
    );

    if file_ext == "svg" {
        let contents = read_to_string(input).unwrap();
        let result = faerber_lib::convert_vector(&contents, method, &labs);
        println!("{result}");
        let mut fp = File::create(output).unwrap();
        fp.write_all(result.as_bytes()).unwrap();
    } else {
        let img: RgbaImage = match image::open(input) {
            Ok(img) => img.to_rgba8(),
            Err(e) => {
                eprintln!("Could not open image: {e}");
                std::process::exit(1);
            }
        };

        let result = faerber_lib::convert(img.clone(), method, &labs);
        let mut c = Cursor::new(Vec::new());
        image::write_buffer_with_format(
            &mut c,
            &result,
            img.width(),
            img.height(),
            image::ColorType::Rgba8,
            image::ImageFormat::Png,
        )
        .expect("Could not write to buffer");
        let compressed = oxipng::optimize_from_memory(&c.into_inner(), &oxipng::Options::default());
        let mut file = std::fs::File::create(output).expect("Could not create file");
        file.write_all(&compressed.expect("Could not compress file"))
            .expect("Could not write to file");
    }
}
