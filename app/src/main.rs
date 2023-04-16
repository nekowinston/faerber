#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use colorschemes::LibraryManager;
use eframe::egui;
use egui_extras::RetainedImage;
use rfd::FileDialog;
use std::{fs, path::PathBuf};

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        initial_window_size: Some(egui::vec2(600.0, 600.0)),
        ..Default::default()
    };

    eframe::run_native(
        "farbenfroh.io - faerber",
        options,
        Box::new(|_cc| Box::<MyApp>::default()),
    )
}

struct MyApp {
    image: Option<RetainedImage>,
    // original_image: Option<RetainedImage>,
    // converted_image: Option<RetainedImage>,
    opened_file: Option<PathBuf>,
    library: LibraryManager,
    color_scheme: String,
    flavor: String,
}

impl Default for MyApp {
    fn default() -> Self {
        Self {
            image: None,
            opened_file: None,
            library: LibraryManager::new(),
            color_scheme: "Atelier Estuary (base16)".to_string(),
            flavor: "Atelier Estuary (base16)".to_string(),
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            if ui.button("Open").clicked() {
                self.opened_file = FileDialog::new()
                    .add_filter("Images", &["png", "jpg", "jpeg"])
                    .add_filter("Vector files", &["svg"])
                    .pick_file();
            }

            if let Some(file) = &mut self.opened_file {
                let data = fs::read(file).unwrap();
                self.image = Some(RetainedImage::from_image_bytes("image", &data).unwrap());
            }

            if let Some(image) = &self.image {
                image.show_max_size(ui, egui::vec2(300.0, 300.0));
            }

            ui.separator();
            ui.label("Color scheme");
            ui.horizontal(|ui| {
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
                println!("{:?}", self.library.library);
                self.library.library[&self.color_scheme]
                    .keys()
                    .for_each(|flavor| {
                        ui.selectable_value(&mut self.flavor, flavor.to_string(), flavor);
                    });
                if ui.button("+").clicked() {}
            });
        });
    }
}
