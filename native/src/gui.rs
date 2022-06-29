use druid::widget::{Align, Button, Flex, Label, Padding, TextBox};
use druid::{
    AppLauncher, FontDescriptor, FontFamily, FontWeight, PlatformError, Widget, WindowDesc,
};

fn build_ui() -> impl Widget<()> {
    Padding::new(
        10.0,
        Flex::row().with_flex_child(
            Flex::column()
                .with_flex_child(
                    Align::centered(
                        Label::new("faerber!").with_font(
                            FontDescriptor::new(FontFamily::SYSTEM_UI)
                                .with_weight(FontWeight::BOLD)
                                .with_size(48.0),
                        ),
                    ),
                    1.0,
                )
                .with_flex_child(Align::centered(Button::new("Convert!")), 1.0),
            1.0,
        ),
    )
}

fn main() -> Result<(), PlatformError> {
    AppLauncher::with_window(WindowDesc::new(build_ui)).launch(())?;
    Ok(())
}
