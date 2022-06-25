use std::path::Path;

#[allow(unused_imports)]
use iced::{
    alignment, button, scrollable, slider, text_input, Alignment, Button, Checkbox, Color, Column,
    Command, Container, ContentFit, Element, Image, Length, Radio, Row, Sandbox, Scrollable,
    Settings, Slider, Space, Text, TextInput, Toggler,
};

use faerber::palettize;
use native_dialog::FileDialog;

pub fn main() -> iced::Result {
    Faerber::run(Settings::default())
}

#[derive(Debug)]
enum Faerber {
    Fresh { upload: button::State },
    Finished { upload: button::State },
}

#[derive(Debug, Clone)]
enum Message {
    Completed(Result<(), Error>),
    ButtonPressed,
}

impl Sandbox for Faerber {
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
                        let newpath = Path::new(&path).to_owned();
                        let npat = newpath.to_str().unwrap();
                        println!("{}", npat);
                        Command::perform(magic(npat.to_owned()), Message::Completed);
                        //palettize(path.to_str(), "latte", "result.png");
                    }
                    None => return,
                };
            }
            Message::Completed(Ok(())) => {
                *self = Self::Finished {
                    upload: button::State::new(),
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
            Self::Finished { upload } => Column::new()
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

async fn magic(path: String) -> Result<(), Error> {
    println!("running :D");
    palettize(path.as_str(), "latte", "result.png");
    Ok(())
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
enum Error {
    APIError,
    LanguageError,
}