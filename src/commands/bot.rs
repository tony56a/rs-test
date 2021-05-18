use crate::commands::bot_cmds::rename::rename_user;
use crate::commands::bot_cmds::text::echo_text;
use crate::models::mpc::{MemeApiResponse, MemeMessage};
use crate::utils::chat::log_msg_err;
use crate::utils::image::{decode_message_from_image, embed_message_into_image};
use image::codecs::png::PngEncoder;
use image::imageops::FilterType;
use image::ColorType;
use serenity::framework::standard::{
    macros::{command, group},
    Args, CommandResult,
};
use serenity::model::prelude::*;
use serenity::prelude::*;

#[command]
pub async fn send(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    if args.is_empty() {
        log_msg_err(
            msg.channel_id
                .say(&ctx.http, String::from("Use me with @user Message"))
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

        let image_res = client
            .get(api_res.url)
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
        arguments: vec![("payload".to_string(), message_str)]
            .iter()
            .cloned()
            .collect(),
    };

    let payload_image = match embed_message_into_image(&input_image, &message) {
        None => return Ok(()),
        Some(image) => image,
    };
    let mut buf = Vec::new();
    let (width, height) = payload_image.dimensions();
    {
        PngEncoder::new(&mut buf)
            .encode(payload_image.as_raw(), width, height, ColorType::Rgb8)
            .expect("Image was not encoded correctly")
    }

    log_msg_err(
        msg.channel_id
            .send_files(&ctx.http, vec![(buf.as_slice(), "message.png")], |m| {
                m.content("!bot receive")
            })
            .await,
    );

    Ok(())
}

#[command]
pub async fn receive(ctx: &Context, msg: &Message) -> CommandResult {
    if msg.attachments.is_empty() {
        return Ok(());
    }

    let image_bytes = match msg.attachments[0].download().await {
        Ok(bytes) => bytes,
        Err(_) => {
            println!("Error downloading image");
            return Ok(());
        }
    };

    let image = image::load_from_memory(&image_bytes).expect("image loading failed");
    let message = match decode_message_from_image(&image) {
        None => return Ok(()),
        Some(message) => message,
    };

    let command_response = match message.command.to_lowercase().as_str() {
        "text" => echo_text(&message, ctx).await,
        "rename" => rename_user(&message, ctx).await,
        _ => {
            log_msg_err(
                msg.reply(&ctx.http, "dunno what to do with the message")
                    .await,
            );
            None
        }
    };

    if !command_response.is_none() {
        log_msg_err(
            msg.channel_id
                .say(&ctx.http, command_response.unwrap())
                .await,
        );
    }

    Ok(())
}

#[group]
#[prefix = "bot"]
#[description = "Commands for bot usage only"]
#[commands(send, receive)]
struct Bot;
