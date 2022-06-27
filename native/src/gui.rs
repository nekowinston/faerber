mod colour_library;

use iced::image as iced_image;
#[allow(unused_imports)]
use iced::{
    alignment, button, scrollable, slider, text_input, Alignment, Button, Checkbox, Color, Column,
    Command, Container, ContentFit, Element, Image, Length, Radio, Row, Sandbox, Scrollable,
    Settings, Slider, Space, Text, TextInput, Toggler,
};

use crate::colour_library::Library;
use faerber::convert;
use native_dialog::FileDialog;

pub fn main() -> iced::Result {
    FaerberApp::run(Settings::default())
}

#[derive(Debug)]
enum FaerberApp {
    Fresh {
        upload: button::State,
    },
    Finished {
        upload: button::State,
        result: FaerberImage,
    },
}

#[derive(Debug, Clone)]
enum Message {
    Completed(Result<Vec<u8>, Error>),
    ButtonPressed,
}

#[derive(Debug, Clone)]
struct FaerberImage {
    data: Vec<u8>,
    width: u32,
    height: u32,
}

impl FaerberImage {
    fn new(path: String) -> FaerberImage {
        let image = image::open(path).unwrap();

        FaerberImage {
            data: image.to_rgba8().to_vec(),
            width: image.width(),
            height: image.height(),
        }
    }
    fn convert(&self) -> Result<Vec<u8>, Error> {
        println!("Converting image");
        let image = image::load_from_memory(&self.data).unwrap().to_rgba8();
        let library: Library = Library::default();
        let labs = library
            .get_palette("Catppuccin")
            .unwrap()
            .get_flavour("")
            .unwrap()
            .get_labs();
        let data = convert(image, faerber::DEMethod::DE2000, &labs);
        Ok(data)
    }
}

impl Sandbox for FaerberApp {
    type Message = Message;

    fn new() -> Self {
        Self::Fresh {
            upload: button::State::new(),
        }
    }

    fn title(&self) -> String {
        String::from("Farbenfroh")
    }

    fn update(&mut self, message: Self::Message) {
        match message {
            Message::ButtonPressed => {
                println!("Button pressed");
                let path = FileDialog::new()
                    .set_location("~")
                    .add_filter("PNG Image", &["png"])
                    .add_filter("JPEG Image", &["jpg", "jpeg"])
                    .show_open_single_file()
                    .unwrap();
                match path {
                    Some(ref path) => {
                        println!("File selected: {:?}", path);
                        let path_str = path.to_str().unwrap().to_string();
                        let img = FaerberImage::new(path_str);
                        //Command::perform(async move {image.convert().await}, Message::Completed);
                        let res = img.convert();
                        match res {
                            Ok(data) => {
                                println!("Conversion successful");
                                println!("Image loaded, {}", img.data.len());
                                *self = Self::Finished {
                                    upload: button::State::new(),
                                    result: FaerberImage::new(String::from("")),
                                };
                            }
                            Err(e) => {
                                println!("Conversion failed: {:?}", e);
                            }
                        };
                    }
                    None => (),
                }
                },
            Message::Completed(Ok(img)) => {
                println!("Image loaded, {}", img.len());
                *self = Self::Finished {
                    upload: button::State::new(),
                    result: FaerberImage::new(String::from("")),
                }
            }
            Message::Completed(Err(_error)) => {
                *self = Self::Fresh {
                    upload: button::State::new(),
                };
                println!("An error occured.");
            }
        }
    }

    fn view(&mut self) -> Element<Self::Message> {
        let content = match self {
            Self::Fresh { upload } => Column::new()
                .padding(20)
                .align_items(Alignment::Center)
                .push(Text::new("faerber!").size(100))
                .push(Button::new(upload, Text::new("Upload")).on_press(Message::ButtonPressed)),
            Self::Finished { upload, result: _ } => Column::new()
                .padding(20)
                .align_items(Alignment::Center)
                .push(Text::new("faerber!").size(100))
                .push(Button::new(upload, Text::new("Upload")).on_press(Message::ButtonPressed))
                .push(Image::new("result.png")),
        };
        Container::new(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .center_y()
            .into()
    }
}

#[derive(Debug, Clone)]
enum Error {}