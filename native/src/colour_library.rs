#![allow(unused)]
#![allow(clippy::unreadable_literal)]

use faerber::custom_lab::Lab;

#[derive(Debug, Clone)]
pub struct Library {
    pub palettes: Vec<Palette>,
}

impl Library {
    pub fn new() -> Library {
        Library {
            palettes: Vec::default(),
        }
    }
    pub fn add_palette(&mut self, palette: Palette) {
        self.palettes.push(palette);
    }
    pub fn remove_palette(&mut self, name: &str) {
        self.palettes.retain(|p| p.name != name);
    }
    pub fn get_palette(&self, name: &str) -> Option<&Palette> {
        self.palettes
            .iter()
            .find(|p| p.name.to_lowercase() == name.to_lowercase())
    }
}

// a Palette is a collection of flavours...
#[derive(Debug, Clone)]
pub struct Palette {
    pub name: String,
    pub flavours: Vec<Flavour>,
}

// and a Flavour is a collection of colours.
#[derive(Debug, Clone)]
pub struct Flavour {
    pub name: String,
    pub colours: Vec<FlavourColour>,
}

// ...each of wich can be toggled on and off.
#[derive(Debug, Clone)]
pub struct FlavourColour {
    pub colour: u32,
    pub enabled: bool,
}

// you can get a flavour from the palette
impl Palette {
    pub fn new(name: &str, flavours: Vec<Flavour>) -> Self {
        Self {
            name: name.to_string(),
            flavours,
        }
    }
    pub fn get_flavour(&self, flavour: &str) -> Option<&Flavour> {
        self.flavours
            .iter()
            .find(|f| f.name.to_lowercase() == flavour.to_lowercase())
    }
}

// and the Vec<hex codes> and Vec<lab values> of a flavour
impl Flavour {
    pub fn new(name: &str, colours: Vec<u32>) -> Self {
        Self {
            name: name.to_string(),
            colours: colours
                .into_iter()
                .map(|c| FlavourColour {
                    colour: c,
                    enabled: true,
                })
                .collect(),
        }
    }
    // get active colours in hex
    pub fn get_hex(&self) -> Vec<String> {
        self.colours
            .iter()
            .filter(|c| c.enabled)
            .map(|c| format!("#{:06x}", c.colour))
            .collect()
    }

    // get active colours as CIELAB
    pub fn get_labs(&self) -> Vec<Lab> {
        self.colours
            .iter()
            .filter(|c| c.enabled)
            .map(|c| {
                Lab::from_rgb(&[
                    ((c.colour >> 16) & 0xFF) as u8,
                    ((c.colour >> 8) & 0xFF) as u8,
                    (c.colour & 0xFF) as u8,
                ])
            })
            .collect()
    }
}

// the factory presets
impl Default for Library {
    fn default() -> Self {
        Self {
            palettes: vec![
                Palette::new(
                    "Catppuccin",
                    vec![
                        Flavour::new(
                            "Latte",
                            vec![
                                0xF5E0DC, 0xF2CDCD, 0xF5C2E7, 0xCBA6F7, 0xF38BA8, 0xEBA0AC,
                                0xFAB387, 0xF9E2AF, 0xA6E3A1, 0x94E2D5, 0x89DCEB, 0x74C7EC,
                                0x89B4FA, 0xB4BEFE, 0xCDD6F4, 0xBAC2DE, 0xA6ADC8, 0x9399B2,
                                0x7F849C, 0x6C7086, 0x585B70, 0x45475A, 0x313244, 0x1E1E2E,
                                0x181825, 0x11111B,
                            ],
                        ),
                        Flavour::new(
                            "Frappe",
                            vec![
                                0xF5E0DC, 0xF2CDCD, 0xF5C2E7, 0xCBA6F7, 0xF38BA8, 0xEBA0AC,
                                0xFAB387, 0xF9E2AF, 0xA6E3A1, 0x94E2D5, 0x89DCEB, 0x74C7EC,
                                0x89B4FA, 0xB4BEFE, 0xCDD6F4, 0xBAC2DE, 0xA6ADC8, 0x9399B2,
                                0x7F849C, 0x6C7086, 0x585B70, 0x45475A, 0x313244, 0x1E1E2E,
                                0x181825, 0x11111B,
                            ],
                        ),
                        Flavour::new(
                            "Macchiato",
                            vec![
                                0xF5E0DC, 0xF2CDCD, 0xF5C2E7, 0xCBA6F7, 0xF38BA8, 0xEBA0AC,
                                0xFAB387, 0xF9E2AF, 0xA6E3A1, 0x94E2D5, 0x89DCEB, 0x74C7EC,
                                0x89B4FA, 0xB4BEFE, 0xCDD6F4, 0xBAC2DE, 0xA6ADC8, 0x9399B2,
                                0x7F849C, 0x6C7086, 0x585B70, 0x45475A, 0x313244, 0x1E1E2E,
                                0x181825, 0x11111B,
                            ],
                        ),
                        Flavour::new(
                            "Mocha",
                            vec![
                                0xF5E0DC, 0xF2CDCD, 0xF5C2E7, 0xCBA6F7, 0xF38BA8, 0xEBA0AC,
                                0xFAB387, 0xF9E2AF, 0xA6E3A1, 0x94E2D5, 0x89DCEB, 0x74C7EC,
                                0x89B4FA, 0xB4BEFE, 0xCDD6F4, 0xBAC2DE, 0xA6ADC8, 0x9399B2,
                                0x7F849C, 0x6C7086, 0x585B70, 0x45475A, 0x313244, 0x1E1E2E,
                                0x181825, 0x11111B,
                            ],
                        ),
                    ],
                ),
                Palette::new(
                    "Dracula",
                    vec![Flavour::new(
                        "Classic",
                        vec![
                            0x282a36, 0x44475a, 0xf8f8f2, 0x6272a4, 0x8be9fd, 0x50fa7b, 0xffb86c,
                            0xff79c6, 0xbd93f9, 0xff5555, 0xf1fa8c,
                        ],
                    )],
                ),
                Palette::new(
                    "Dracula PRO",
                    vec![
                        Flavour::new(
                            "Default",
                            vec![
                                0xffff80, 0xff9580, 0x80ffea, 0xff80bf, 0xffca80, 0x8aff80,
                                0x9580ff, 0xf8f8f2, 0x454158, 0x7970a9, 0x22212c,
                            ],
                        ),
                        Flavour::new(
                            "Blade",
                            vec![
                                0xffff80, 0xff9580, 0x80ffea, 0xff80bf, 0xffca80, 0x8aff80,
                                0x9580ff, 0xf8f8f2, 0x415854, 0x70a99f, 0x212c2a,
                            ],
                        ),
                        Flavour::new(
                            "Buffy",
                            vec![
                                0xffff80, 0xff9580, 0x80ffea, 0xff80bf, 0xffca80, 0x8aff80,
                                0x9580ff, 0xf8f8f2, 0x544158, 0x9f70a9, 0x2a212c,
                            ],
                        ),
                        Flavour::new(
                            "Lincoln",
                            vec![
                                0xffff80, 0xff9580, 0x80ffea, 0xff80bf, 0xffca80, 0x8aff80,
                                0x9580ff, 0xf8f8f2, 0x585441, 0xa99f70, 0x2c2a21,
                            ],
                        ),
                        Flavour::new(
                            "Morbius",
                            vec![
                                0xffff80, 0xff9580, 0x80ffea, 0xff80bf, 0xffca80, 0x8aff80,
                                0x9580ff, 0xf8f8f2, 0x584145, 0xa97079, 0x2c2122,
                            ],
                        ),
                        Flavour::new(
                            "Van Helsing",
                            vec![
                                0xffff80, 0xff9580, 0x80ffea, 0xff80bf, 0xffca80, 0x8aff80,
                                0x9580ff, 0xf8f8f2, 0x414d58, 0x708ca9, 0x0b0d0f,
                            ],
                        ),
                    ],
                ),
                Palette::new(
                    "Nord",
                    vec![
                        Flavour::new("Polar Night", vec![0x2e3440, 0x3b4252, 0x434c5e, 0x4c566a]),
                        Flavour::new("Snow Storm", vec![0xd8dee9, 0xe5e9f0, 0xeceff4]),
                        Flavour::new("Frost", vec![0x8fbcbb, 0x88c0d0, 0x81a1c1, 0x5e81ac]),
                        Flavour::new(
                            "Aurora",
                            vec![0xbf616a, 0xd08770, 0xebcb8b, 0xa3be8c, 0xb48ead],
                        ),
                    ],
                ),
            ],
        }
    }
}
