use super::cga;
use crate::color::{Error, Mode, RGB};
use std::collections::HashSet;
use std::fs::File;
use std::io::Write;
#[test]
fn parse() {
    const GARBAGE: &str = "ASDASLKJAS";

    let tt: Vec<(&str, Result<Mode, Error>)> = vec![
        ("bw", Ok(Mode::BlackAndWhite)),
        ("c", Ok(Mode::Color)),
        ("color", Ok(Mode::Color)),
        ("RED", Ok(Mode::SingleColor(cga::RED))),
        ("blue", Ok(Mode::SingleColor(cga::BLUE))),
        ("LigHT_CYAN", Ok(Mode::SingleColor(cga::LIGHT_CYAN))),
        ("cga", Ok(Mode::CGA)),
        ("cRaYoN", Ok(Mode::CRAYON)),
        (GARBAGE, Err(Error::UnknownOption(GARBAGE.to_string()))),
    ];
    for (s, want) in tt {
        assert_eq!(s.parse::<Mode>(), want);
    }

    let mut input = std::env::current_dir().unwrap();
    input.push("temp_cga.plt");

    let mut file = File::create(&input).unwrap();
    write!(
        file,
        "
0x000000
0x0000AA
0x00AA00
0x00AAAA
0xAA00AA
0xAA0000
0xAA5500
0xAAAAAA
0x555555
0x5555FF
0x55FF55
0x55FFFF
0xFF5555
0xFF55FF
0xFFFF55
0xFFFFFF"
    )
    .unwrap();
    let want_palette: HashSet<RGB<u8>> = cga::ALL.iter().cloned().collect();
    if let Mode::Palette {
        palette: got_palette,
        ..
    } = input
        .to_string_lossy()
        .parse::<Mode>()
        .expect("should have no trouble parsing")
    {
        assert_eq!(want_palette, got_palette.iter().cloned().collect());
    } else {
        unreachable!()
    }
    std::fs::remove_file(input).unwrap();
}
