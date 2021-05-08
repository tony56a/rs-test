use crate::utils::chat::log_msg_err;
use serenity::framework::standard::{
    macros::{command, group},
    Args, CommandResult,
};
use serenity::model::prelude::*;
use serenity::prelude::*;
use serenity::utils::MessageBuilder;
use crate::models::mpc::{MemeMessage, MemeApiResponse};
use image::ColorType;
use image::imageops::FilterType;
use crate::utils::image::embed_message_into_image;
use image::codecs::png::PngEncoder;

#[command]
pub async fn send(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    if args.is_empty() {
        log_msg_err(
            msg.channel_id
                .say(
                    &ctx.http,
                    String::from("Use me with @user Message"),
                )
                .await,
        );
        return Ok(());
    }
    let provided_user = args.single::<String>()?;
    let message_str = args.single_quoted::<String>()?;

    // Validate based on inferred mentions (also cap number of times to spam)
    // (i.e: 1 user mention specfically, and can't be a bot/broadcast)
    // TODO: read up docs and find out how to do this correctly
    if msg.mention_everyone {
        return Ok(());
    }
    if msg.mentions.is_empty() {
        log_msg_err(
            msg.channel_id
                .say(
                    &ctx.http,
                    format!("{user} doesn't exist!", user = provided_user),
                )
                .await,
        );
        return Ok(());
    }
    if !msg.mentions[0].bot {
        log_msg_err(
            msg.channel_id
                .say(
                    &ctx.http,
                    format!("Humans cannot understand the ancient language of the memes!"),
                )
                .await,
        );
        return Ok(());
    }

    let input_image = {
        let client = reqwest::Client::new();
        let api_res = client
            .get("https://meme-api.herokuapp.com/gimme")
            .send()
            .await
            .unwrap()
            .json::<MemeApiResponse>()
            .await
            .unwrap();

        let image_res = client.get(api_res.url)
            .send()
            .await
            .unwrap()
            .bytes()
            .await
            .unwrap();

        let image = image::load_from_memory(&image_res).expect("image loading failed");

        let resized_image = image.resize_exact(640, 480, FilterType::Triangle);
        resized_image
    };

    let message = MemeMessage {
        command: "text".to_string(),
        arguments: vec![
            ("payload".to_string(), message_str),
        ].iter().cloned().collect(),
    };

    let payload_image = match embed_message_into_image(&input_image, &message) {
        None => { return Ok(())}
        Some(image) => { image }
    };
    let mut buf = Vec::new();
    let (width, height) = payload_image.dimensions();
    {
        PngEncoder::new(&mut buf)
            .encode(
                payload_image.as_raw(),
                width,
                height,
                ColorType::Rgb8
            ).expect("Image was not encoded correctly")
    }

    log_msg_err(msg.channel_id.send_files(&ctx.http, vec![(buf.as_slice(), "message.png")],  |m| {
        m.content("!bot receive")
    }).await);

    Ok(())
}

#[group]
#[prefix = "bot"]
#[description = "Commands for bot usage only"]
#[commands(send)]
struct Bot;
