#![warn(
    // clippy::cargo,
    // clippy::expect_used,
    // clippy::pedantic,
    // clippy::unwrap_used,
    clippy::complexity,
    clippy::nursery,
    clippy::perf,
    clippy::style,
    clippy::suspicious,
)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use catppuccin_egui::{set_theme, LATTE, MOCHA};
use colorschemes::LibraryManager;
use eframe::egui;
use egui::{ColorImage, Sense, Stroke, Vec2};
use egui_extras::RetainedImage;
use rfd::FileDialog;
use std::{fs, path::PathBuf};
use strum::IntoEnumIterator;
use sublime_fuzzy::best_match;

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        initial_window_size: Some(egui::vec2(600.0, 600.0)),
        ..Default::default()
    };

    eframe::run_native("faerber", options, Box::new(|_cc| Box::<MyApp>::default()))
}

struct MyApp {
    image: Option<RetainedImage>,
    converted_image: Option<RetainedImage>,
    converted_image_src: Option<image::RgbaImage>,
    opened_file: Option<PathBuf>,
    library: LibraryManager,
    color_scheme: String,
    flavor: String,
    flavor_filter: String,
    method: faerber_lib::ConversionMethod,
}

impl Default for MyApp {
    fn default() -> Self {
        Self {
            image: None,
            converted_image: None,
            converted_image_src: None,
            opened_file: None,
            library: LibraryManager::new(),
            color_scheme: "wezterm".to_string(),
            flavor: "catppuccin mocha".to_string(),
            flavor_filter: "".to_string(),
            method: faerber_lib::ConversionMethod::De1976,
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        match frame.info().system_theme {
            Some(eframe::Theme::Dark) => set_theme(ctx, MOCHA),
            _ => set_theme(ctx, LATTE),
        }

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.horizontal_top(|ui| {
                ui.menu_button("File", |ui| self.file_menu(ui));
                if ui.button("Convert").clicked() {
                    if let Some(file) = &self.opened_file {
                        let img = image::open(file).unwrap();
                        self.image = None;
                        let x = match self.method {
                            faerber_lib::ConversionMethod::De1976
                            | faerber_lib::ConversionMethod::De1994T
                            | faerber_lib::ConversionMethod::De1994G
                            | faerber_lib::ConversionMethod::De2000 => faerber_lib::convert_naive(
                                &img.to_rgba8(),
                                self.method,
                                self.library
                                    .get("wezterm")
                                    .unwrap()
                                    .get(&self.flavor)
                                    .unwrap()
                                    .get_labs()
                                    .as_slice(),
                            ),
                            faerber_lib::ConversionMethod::DitherFloydSteinberg
                            | faerber_lib::ConversionMethod::DitherAtkinson
                            | faerber_lib::ConversionMethod::DitherStucki
                            | faerber_lib::ConversionMethod::DitherBurkes
                            | faerber_lib::ConversionMethod::DitherJarvisJudiceNinke
                            | faerber_lib::ConversionMethod::DitherSierra3 => {
                                faerber_lib::convert_dither(
                                    &img.to_rgba8(),
                                    self.method,
                                    self.library
                                        .get("wezterm")
                                        .unwrap()
                                        .get(&self.flavor)
                                        .unwrap()
                                        .get_u32s()
                                        .as_slice(),
                                )
                            }
                        };
                        let y = x.expect("Conversion failed");
                        self.converted_image_src = Some(
                            image::RgbaImage::from_vec(img.width(), img.height(), y.clone())
                                .unwrap(),
                        );
                        let ret_img = RetainedImage::from_color_image(
                            "converted_image",
                            ColorImage::from_rgba_unmultiplied(
                                [
                                    usize::try_from(img.width()).unwrap(),
                                    usize::try_from(img.height()).unwrap(),
                                ],
                                y.as_slice(),
                            ),
                        );
                        self.converted_image = Some(ret_img)
                    }
                }
            })
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            if let Some(file) = &mut self.opened_file {
                let is_svg = file.extension().unwrap() == "svg";
                let data = fs::read(file).unwrap();
                self.image = if is_svg {
                    Some(RetainedImage::from_svg_bytes("image", &data).unwrap())
                } else {
                    Some(RetainedImage::from_image_bytes("image", &data).unwrap())
                }
            }

            ui.separator();
            ui.label("Color scheme");
            ui.horizontal_wrapped(|ui| {
                self.library.library.keys().for_each(|color_scheme| {
                    ui.selectable_value(
                        &mut self.color_scheme,
                        color_scheme.to_string(),
                        color_scheme,
                    );
                });
                if ui.button("+").clicked() {}
            });
            ui.separator();
            ui.label("Flavor");
            ui.horizontal(|ui| {
                ui.add(egui::TextEdit::singleline(&mut self.flavor_filter));
                if ui.button("X").clicked() {
                    self.flavor_filter = "".to_string();
                }
            });
            ui.horizontal(|ui| {
                egui::ComboBox::from_label("")
                    .selected_text(&self.flavor)
                    .show_ui(ui, |ui| {
                        self.library
                            .library
                            .get(&self.color_scheme)
                            .unwrap()
                            .keys()
                            .filter_map(|f| {
                                if self.flavor_filter.is_empty() {
                                    Some(f)
                                } else {
                                    best_match(&self.flavor_filter, f).map(|_| f)
                                }
                            })
                            .for_each(|flavor| {
                                ui.selectable_value(&mut self.flavor, flavor.to_string(), flavor);
                            });
                    });
                if ui.button("+").clicked() {}
            });
            ui.label("Colors");
            ui.horizontal_wrapped(|ui| {
                let library = &self.library;
                library
                    .get(&self.color_scheme)
                    .unwrap()
                    .get(&self.flavor)
                    .unwrap()
                    .get_colors()
                    .iter()
                    .for_each(|color| {
                        let r = 10.0;
                        let size = Vec2::splat(2.0f32.mul_add(r, 5.0));
                        let (rect, sense) = ui
                            .allocate_at_least(size, Sense::union(Sense::hover(), Sense::click()));

                        let hint_text = format!("{}: {}", color.name, color.enabled);
                        sense
                            .clone()
                            .on_hover_cursor(egui::CursorIcon::PointingHand)
                            .on_hover_text(hint_text);

                        let cv = color.value;
                        let c_view = egui::Color32::from_rgba_premultiplied(
                            (cv >> 16 & 0xFF) as u8,
                            (cv >> 8 & 0xFF) as u8,
                            (cv & 0xFF) as u8,
                            255,
                        );

                        let c = match color.enabled {
                            true => c_view,
                            false => c_view.gamma_multiply(0.3),
                        };

                        if sense.clicked() {
                            self.library
                                .set_color(
                                    &self.color_scheme,
                                    &self.flavor,
                                    &color.name,
                                    !color.enabled,
                                )
                                .unwrap();
                        }

                        match color.enabled {
                            true => ui.painter().circle_filled(rect.center(), r, c),
                            false => ui.painter().circle(
                                rect.center(),
                                r,
                                egui::Color32::TRANSPARENT,
                                Stroke::new(1.0, c),
                            ),
                        };
                    });
            });
            ui.label("Conversion Method");
            ui.horizontal_wrapped(|ui| {
                // for each key of ConversionMethod, create a button changing the conversion method
                faerber_lib::ConversionMethod::iter().for_each(|method| {
                    ui.selectable_value(&mut self.method, method, method.to_string());
                });
            });
            ui.separator();
            if let Some(img) = &self.image {
                img.show_max_size(ui, egui::vec2(ui.available_width(), 300.0));
            }
            if let Some(img) = &self.converted_image {
                img.show_max_size(ui, egui::vec2(ui.available_width(), 300.0));
            }
        });
    }
}

impl MyApp {
    fn file_menu(&mut self, ui: &mut egui::Ui) {
        if ui.button("Open").clicked() {
            ui.close_menu();
            self.opened_file = FileDialog::new()
                .add_filter("Images", &["png", "jpg", "jpeg"])
                .add_filter("Vector files", &["svg"])
                .pick_file();
        }
        ui.add_enabled_ui(self.converted_image_src.is_some(), |ui| {
            if ui.button("Save").clicked() {
                let path = FileDialog::new()
                    .add_filter("Images", &["png", "jpg", "jpeg"])
                    .add_filter("Vector files", &["svg"])
                    .save_file();
                if let Some(path) = path {
                    _ = self
                        .converted_image_src
                        .as_ref()
                        .unwrap()
                        .save_with_format(&path, image::ImageFormat::from_path(&path).unwrap());
                }
            }
        });
    }
}
