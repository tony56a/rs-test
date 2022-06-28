use crate::constants;
use crate::models::holders::BotConfig;
use crate::utils::chat::log_msg_err;
use base64::decode;
use image::{load_from_memory, ImageOutputFormat};
use reqwest;
use reqwest::Response;
use reqwest_middleware::{ClientBuilder, ClientWithMiddleware};
use serde::Deserialize;
use serenity::framework::standard::{
    macros::{command, group},
    Args, CommandResult,
};
use serenity::model::prelude::*;
use serenity::prelude::*;
use std::collections::HashMap;
use reqwest_retry::policies::ExponentialBackoff;
use reqwest_retry::RetryTransientMiddleware;

#[derive(Deserialize)]
struct ImageApiResponse {
    output_url: String,
}

#[derive(Deserialize)]
struct TextApiResponse {
    output: String,
}

#[derive(Deserialize)]
struct Text2ImApiResponse {
    images: Vec<String>,
}

#[command]
pub async fn picture(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    if args.is_empty() {
        log_msg_err(
            msg.channel_id
                .say(
                    &ctx.http,
                    String::from("Use me with \"a sentence, any sentence\""),
                )
                .await,
        );
        return Ok(());
    }
    let response = invoke_deepai(ctx, msg, args, "https://api.deepai.org/api/text2img").await;
    let res = response.json::<ImageApiResponse>().await?;
    log_msg_err(msg.channel_id.say(&ctx.http, res.output_url).await);
    Ok(())
}

// This differs from picture, in that it's using a real model (https://hf.space/gradioiframe/valhalla/glide-text2im/api)
#[command]
pub async fn image(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    if args.is_empty() {
        log_msg_err(
            msg.channel_id
                .say(
                    &ctx.http,
                    String::from("Use me with \"a sentence, any sentence\""),
                )
                .await,
        );
        return Ok(());
    }

    let initial_message = match msg
        .channel_id
        .say(&ctx.http, "Beep boop, thinking...")
        .await
    {
        Ok(message) => message,
        Err(e) => {
            // Failure path, just return
            println!("Error when publishing message: {:?}", e);
            return Ok(());
        }
    };

    let retry_policy = ExponentialBackoff::builder().build_with_max_retries(3);
    let client = ClientBuilder::new(reqwest::Client::new())
        .with(RetryTransientMiddleware::new_with_policy(retry_policy))
        .build();

    let sentence = String::from(args.single_quoted::<String>().unwrap().trim());
    let mut map = HashMap::new();
    //map.insert("data", [sentence.clone()]);
    map.insert("prompt", sentence.clone());

    // let url = "https://hf.space/gradioiframe/valhalla/glide-text2im/+/api/predict/";
    let url = "https://backend.craiyon.com/generate";

    let response = client
        .post(url)
        .json(&map)
        .send()
        .await;

    match response {
        Ok(responsPayload) => {
            let images = responsPayload.json::<Text2ImApiResponse>().await?;
            let image_content_base64 = images.images[0].replace("\n", "");
            let image_buf = decode(image_content_base64).unwrap();
            let image = load_from_memory(image_buf.as_slice()).unwrap();
            let mut buf = Vec::new();
            image.write_to(&mut buf, ImageOutputFormat::Png);
            msg.channel_id
                .delete_message(&ctx.http, initial_message.id)
                .await?;
            log_msg_err(
                msg.channel_id
                    .send_files(&ctx.http, vec![(buf.as_slice(), "message.png")], |m| {
                        m.content(&sentence)
                    })
                    .await,
            );
        }
        Err(e) => {
            msg.channel_id
                .delete_message(&ctx.http, initial_message.id)
                .await?;
            println!("Error when Getting image: {:?}", e)
        }
    };

    Ok(())
}

#[command]
pub async fn story(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    if args.is_empty() {
        log_msg_err(
            msg.channel_id
                .say(
                    &ctx.http,
                    String::from("Use me with \"a sentence, any sentence\""),
                )
                .await,
        );
        return Ok(());
    }
    let response = invoke_deepai(ctx, msg, args, "https://api.deepai.org/api/text-generator").await;
    let res = response.json::<TextApiResponse>().await?;
    log_msg_err(msg.channel_id.say(&ctx.http, res.output).await);
    Ok(())
}

#[command]
pub async fn weird_shit(ctx: &Context, msg: &Message) -> CommandResult {
    let res = {
        let data_read = ctx.data.read().await;
        let bot_config_lock = data_read
            .get::<BotConfig>()
            .expect("Expected BotConfig in TypeMap.")
            .clone();
        let bot_config = bot_config_lock.read().await;
        let client = reqwest::Client::new();

        let params = [("image", "https://loremflickr.com/640/480/deepdream")];

        client
            .post("https://api.deepai.org/api/deepdream")
            .header(
                "api-key",
                bot_config.get::<str>(&constants::DEEPAI_TOKEN_KEY).unwrap(),
            )
            .form(&params)
            .send()
            .await?
            .json::<ImageApiResponse>()
            .await?
    };

    log_msg_err(msg.channel_id.say(&ctx.http, res.output_url).await);
    Ok(())
}

async fn invoke_deepai(ctx: &Context, msg: &Message, mut args: Args, url: &str) -> Response {
    let sentence = String::from(args.single_quoted::<String>().unwrap().trim());
    let params = [("text", sentence)];

    {
        let data_read = ctx.data.read().await;
        let bot_config_lock = data_read
            .get::<BotConfig>()
            .expect("Expected BotConfig in TypeMap.")
            .clone();
        let bot_config = bot_config_lock.read().await;
        let client = reqwest::Client::new();
        client
            .post(url)
            .header(
                "api-key",
                bot_config.get::<str>(&constants::DEEPAI_TOKEN_KEY).unwrap(),
            )
            .form(&params)
            .send()
            .await
            .unwrap()
    }
}

#[group]
#[prefix = "imagine"]
#[description = "Commands to synthesize things via ML"]
#[commands(picture, weird_shit, story, image)]
struct Imagine;
