use crate::constants;
use crate::models::holders::BotConfig;
use crate::utils::chat::log_msg_err;
use reqwest;
use serde::Deserialize;
use serenity::framework::standard::{
    macros::{command, group},
    Args, CommandResult,
};
use serenity::model::prelude::*;
use serenity::prelude::*;
use reqwest::Response;

#[derive(Deserialize)]
struct ImageApiResponse {
    output_url: String,
}

#[derive(Deserialize)]
struct TextApiResponse {
    output: String,
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

    let res = {
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
    };
    res
}

#[group]
#[prefix = "imagine"]
#[description = "Commands to synthesize things via ML"]
#[commands(picture, weird_shit, story)]
struct Imagine;
