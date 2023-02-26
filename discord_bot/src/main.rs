use std::time::Duration;

use faerber::{get_labs, LIBRARY};
use faerber_lib::convert;
use image::RgbaImage;
use poise::serenity_prelude::{self as serenity};

struct Data {}
type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

async fn download_and_convert_image(url: &str, flavor: &str) -> Result<String, Error> {
    let response = reqwest::get(url).await?;
    let bytes = response.bytes().await?;

    let image: RgbaImage = image::load_from_memory(&bytes)
        .expect("Unable to open image")
        .to_rgba8();
    let imgsize = (image.width(), image.height());

    let flavor = LIBRARY
        .get("catppuccin")
        .expect("Could not find catppuccin in library")
        .get(flavor)
        .expect("Could not find flavor in catppuccin");

    let labs = get_labs(flavor.clone());

    let result = convert(image, faerber_lib::DEMethod::DE2000, &labs);
    let _r = image::save_buffer(
        "cache.png",
        &result,
        imgsize.0,
        imgsize.1,
        image::ColorType::Rgba8,
    );

    Ok("cache.png".to_string())
}

async fn ask_for_flavor(ctx: Context<'_>) -> Result<serenity::Message, serenity::Error> {
    ctx.send(|m| {
        m.ephemeral(true)
            .content("What flavor do you want?")
            .components(|c| {
                c.create_action_row(|row| {
                    row.create_select_menu(|menu| {
                        menu.custom_id("flavor_select");
                        menu.placeholder("No flavor selected");
                        menu.options(|o| {
                            o.create_option(|opt| opt.label("🌿 Mocha").value("mocha"));
                            o.create_option(|opt| opt.label("🌺 Macchiato").value("macchiato"));
                            o.create_option(|opt| opt.label("🪴 Frappe").value("frappe"));
                            o.create_option(|opt| opt.label("🌻 Latte").value("latte"))
                        })
                    })
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
        ctx.defer_ephemeral().await?;
        ctx.say("No attachments found").await?;
        return Ok(());
    } else {
        ctx.defer().await?;
    }

    // check if the attachments start with "image/"
    let mut non_image_attachments = false;
    for attachment in message.attachments.iter() {
        if attachment.width.is_none() {
            non_image_attachments = true;
        }
    }
    if non_image_attachments {
        ctx.defer_ephemeral().await?;
        ctx.say("Unsupported attachments on that message").await?;
        return Ok(());
    }

    let url = &message.attachments[0].url;

    let msg = ask_for_flavor(ctx).await?;
    let interaction = msg
        .await_component_interaction(ctx)
        .author_id(ctx.author().id)
        .timeout(Duration::from_secs(30));

    if let Some(item) = interaction.await {
        let flavor = &item.data.values[0];
        download_and_convert_image(url, flavor).await?;
        item.create_interaction_response(&ctx, |r| {
            r.kind(serenity::InteractionResponseType::ChannelMessageWithSource)
                .interaction_response_data(|d| {
                    d.content(format!("Here's your picture in {}!", item.data.values[0]))
                        .add_file("./cache.png")
                })
        })
        .await?;
        msg.delete(&ctx).await?;

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
