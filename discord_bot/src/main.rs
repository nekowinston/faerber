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

use std::{
    io::{Cursor, Write},
    path::PathBuf,
    time::Duration,
};

use colorschemes::LibraryManager;
use faerber::get_labs;
use faerber_lib::convert_naive;
use phf::phf_map;
use poise::serenity_prelude::{self as serenity, Mentionable, ReactionType};
use sha2::{Digest, Sha256};

extern crate oxipng;

struct Data {}
type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

#[derive(Clone, Copy)]
struct FlavorInfo {
    name: &'static str,
    emoji: &'static str,
    id: &'static str,
}
impl From<FlavorInfo> for serenity::CreateButton {
    fn from(info: FlavorInfo) -> Self {
        create_button(
            info.name,
            info.emoji.parse().expect("Failed to parse emoji"),
            info.id,
        )
    }
}
impl From<FlavorInfo> for String {
    fn from(info: FlavorInfo) -> Self {
        info.emoji.to_string() + " " + info.name
    }
}

const FLAVORS: phf::Map<&'static str, FlavorInfo> = phf_map! {
    "mocha" => FlavorInfo {
        name: "Mocha",
        emoji: "🌿",
        id: "mocha",
    },
    "macchiato" => FlavorInfo {
        name: "Macchiato",
        emoji: "🌺",
        id: "macchiato",
    },
    "frappe" => FlavorInfo {
        name: "Frappé",
        emoji: "🪴",
        id: "frappe",
    },
    "latte" =>FlavorInfo {
        name: "Latte",
        emoji: "🌻",
        id: "latte",
    },
};

const MAX_WIDTH: u32 = 1920 * 2;
const MAX_HEIGHT: u32 = 1080 * 2;

struct ConversionResult {
    path: PathBuf,
    downsized: bool,
}

async fn download_and_convert_image(url: &str, flavor: &str) -> Result<ConversionResult, Error> {
    let response = reqwest::get(url).await?;
    let bytes = response.bytes().await?;

    let mut hasher = Sha256::new();
    hasher.update(&bytes);
    let hashed_file_name = format!("{:x}_{flavor}.png", hasher.finalize());
    if PathBuf::from(&hashed_file_name).exists() {
        return Ok(ConversionResult {
            path: hashed_file_name.into(),
            // this doesn't really matter here
            downsized: false,
        });
    }

    let mut image = image::load_from_memory(&bytes)?;
    let mut imgsize = (image.width(), image.height());

    let downsized = if imgsize.0 > MAX_WIDTH || imgsize.1 > MAX_HEIGHT {
        image = image.resize_to_fill(MAX_WIDTH, MAX_HEIGHT, image::imageops::FilterType::Lanczos3);
        imgsize = (image.width(), image.height());
        true
    } else {
        false
    };

    let library = Library::new();
    let flavor = library
        .get("catppuccin")
        .expect("Could not find catppuccin in library")
        .get(flavor)
        .expect("Could not find flavor in catppuccin");

    let labs = get_labs(flavor.clone());

    let result = convert_naive(&image.to_rgba8(), faerber_lib::DEMethod::DE2000, &labs);
    let mut c = Cursor::new(Vec::new());
    image::write_buffer_with_format(
        &mut c,
        &result,
        imgsize.0,
        imgsize.1,
        image::ColorType::Rgba8,
        image::ImageFormat::Png,
    )?;
    let compressed = oxipng::optimize_from_memory(&c.into_inner(), &oxipng::Options::default())?;
    let mut file = std::fs::File::create(&hashed_file_name)?;
    file.write_all(&compressed)?;

    Ok(ConversionResult {
        path: hashed_file_name.into(),
        downsized,
    })
}

fn create_button(label: &str, emoji: ReactionType, id: &str) -> serenity::CreateButton {
    let mut b = serenity::CreateButton::default();
    b.custom_id(id);
    b.label(label);
    b.emoji(emoji);
    b.style(serenity::ButtonStyle::Primary);
    b
}

async fn ask_for_flavor(ctx: Context<'_>) -> Result<serenity::Message, serenity::Error> {
    ctx.send(|m| {
        m.ephemeral(true)
            .content("What flavor do you want?")
            .components(|c| {
                c.create_action_row(|row| {
                    // using a map here messed up the order of the buttons ¯\_(ツ)_/¯
                    row.add_button(FLAVORS["mocha"].into());
                    row.add_button(FLAVORS["macchiato"].into());
                    row.add_button(FLAVORS["frappe"].into());
                    row.add_button(FLAVORS["latte"].into());
                    row
                })
            })
    })
    .await?
    .into_message()
    .await
}

#[poise::command(context_menu_command = "Faerber")]
async fn faerber(
    ctx: Context<'_>,
    #[description = "Faerber"] message: serenity::Message,
) -> Result<(), Error> {
    if message.attachments.is_empty() {
        ctx.send(|m| m.ephemeral(true).content("No attachments found"))
            .await?;
        return Ok(());
    }
    // all further messages are ephemeral
    ctx.defer_ephemeral().await?;

    let mut non_image_attachments = false;
    for attachment in &message.attachments {
        if let Some(content_type) = &attachment.content_type {
            if !content_type.starts_with("image/") {
                non_image_attachments = true;
            }
        } else {
            non_image_attachments = true;
        }
    }
    if non_image_attachments {
        ctx.send(|m| {
            m.ephemeral(true)
                .content("Unsupported attachments on that message")
        })
        .await?;
        return Ok(());
    }

    let url = &message.attachments[0].url;

    let msg = ask_for_flavor(ctx).await?;
    let interaction = msg
        .await_component_interaction(ctx)
        .timeout(Duration::from_secs(30));

    if let Some(item) = interaction.await {
        let flavor = &item.data.custom_id;
        let flavorname: String = FLAVORS[flavor].into();

        item.create_interaction_response(&ctx, |r| {
            r.kind(serenity::InteractionResponseType::DeferredUpdateMessage)
        })
        .await?;

        let converted = download_and_convert_image(url, flavor).await?;
        let user = ctx.author().mention();
        msg.channel_id
            .send_message(&ctx, |m| {
                let mut text: String = format!("Here's your image in {flavorname} - requested by {user}");

                // add note if the image was downsized
                if converted.downsized {
                    text.push_str(&format!("\nImage sizes are limited to {MAX_WIDTH}x{MAX_HEIGHT}. Please use the CLI or web app for the full resolution."));
                }

                m.content(text).add_file(&converted.path);
                m
            })
            .await?;

        return Ok(());
    }

    Ok(())
}

#[tokio::main]
async fn main() {
    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![faerber()],
            ..Default::default()
        })
        .token(std::env::var("DISCORD_TOKEN").expect("missing DISCORD_TOKEN"))
        .intents(serenity::GatewayIntents::non_privileged())
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;

                Ok(Data {})
            })
        });

    framework.run().await.unwrap();
}
