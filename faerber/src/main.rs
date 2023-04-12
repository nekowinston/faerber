use clap::{value_parser, Arg, ArgAction, Command, ValueEnum, ValueHint};
use clap_complete::{generate, Generator, Shell};
use faerber::{get_labs, parse_colorscheme, ColorScheme, Palette, LIBRARY};
use faerber_lib::DEMethod;
use faerber_lib::Lab;
use image::RgbaImage;
use std::fs::File;
use std::io;
use std::io::prelude::*;
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
            CliDeltaMethods::De76 => DEMethod::DE1976,
            CliDeltaMethods::De94t => DEMethod::DE1994T,
            CliDeltaMethods::De94g => DEMethod::DE1994T,
            CliDeltaMethods::De2000 => DEMethod::DE2000,
        }
    }
}

fn build_cli() -> Command {
    Command::new("faerber")
        .arg(
            Arg::new("input")
                .required_unless_present("completion")
                .value_parser(value_parser!(PathBuf))
                .value_hint(ValueHint::FilePath),
        )
        .arg(
            Arg::new("output")
                .required_unless_present("completion")
                .value_parser(value_parser!(String))
                .value_hint(ValueHint::FilePath),
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
                .value_parser(["catppuccin", "dracula", "gruvbox", "nord", "solarized"])
                .default_value("catppuccin"),
        )
        .arg(Arg::new("flavour").short('f').long("flavour").num_args(1..))
        .arg(
            Arg::new("verbose")
                .short('v')
                .long("verbose")
                .action(ArgAction::Count),
        )
        .arg(
            Arg::new("completion")
                .long("completion")
                .value_parser(value_parser!(Shell))
                .value_hint(ValueHint::Other),
        )
}

fn print_completions<G: Generator>(gen: G, cmd: &mut Command) {
    generate(gen, cmd, cmd.get_name().to_string(), &mut io::stdout());
}

fn main() {
    let matches = build_cli().get_matches();

    if let Some(completion) = matches.get_one::<Shell>("completion").copied() {
        let mut cmd = build_cli();
        eprintln!("Generating completion file for {completion}...");
        print_completions(completion, &mut cmd);
        return;
    }

    let input = matches.get_one::<PathBuf>("input").expect("required");
    let output = matches.get_one::<String>("output").expect("required");
    let method: DEMethod = matches
        .get_one::<CliDeltaMethods>("method")
        .expect("required")
        .to_owned()
        .into();
    let palette = matches.get_one::<String>("palette").expect("required");
    let flavour = matches.get_many::<String>("flavour");

    let file_path = Path::new(input);
    println!("Reading image from {file_path:?}");
    let file_ext = file_path.extension().unwrap().to_str().unwrap();

    let mut custom_colorscheme: ColorScheme = ColorScheme::new();
    let colorscheme = LIBRARY.get(palette).unwrap_or_else(|| {
        let mut file = File::open(palette).expect("file not found");
        let mut contents = String::new();
        file.read_to_string(&mut contents)
            .expect("something went wrong reading the file");

        custom_colorscheme = parse_colorscheme(serde_json::from_str(&contents).unwrap());
        &custom_colorscheme
    });

    let labs: Vec<Lab> = match flavour {
        Some(flavour) => {
            let mut labs: Vec<Lab> = vec![];
            for f in flavour {
                if colorscheme.contains_key(f) {
                    labs.append(&mut get_labs(colorscheme.get(f).unwrap().to_owned()));
                } else {
                    eprintln!("Could not find flavour: {f}");
                    eprintln!(
                        "Available flavours: {:?}",
                        colorscheme
                            .keys()
                            .map(|s| s.to_lowercase())
                            .collect::<Vec<_>>()
                    );
                    std::process::exit(1);
                }
            }
            labs
        }
        None => {
            let palette: Palette = colorscheme
                .values()
                .next()
                .expect("palette should have a flavour")
                .to_owned();
            get_labs(palette)
        }
    };

    if file_ext != "svg" {
        let img: RgbaImage = match image::open(input) {
            Ok(img) => img.to_rgba8(),
            Err(e) => {
                eprintln!("Could not open image: {e}");
                std::process::exit(1);
            }
        };

        let result = faerber_lib::convert(img.to_owned(), method.to_owned(), &labs);
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
    } else {
        let mut fp = File::open(input).unwrap();
        let mut contents = String::new();
        fp.read_to_string(&mut contents).unwrap();
        let result = faerber_lib::convert_vector(&contents, method.to_owned(), &labs);
        println!("{result}");
        let mut fp = File::create(output).unwrap();
        fp.write_all(result.as_bytes()).unwrap();
    }
}
