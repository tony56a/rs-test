use crate::constants;
use crate::models::bot_config::BotConfig;
use reqwest;
use serde::Deserialize;
use serenity::framework::standard::{
    macros::{command, group},
    Args, CommandResult,
};
use serenity::model::prelude::*;
use serenity::prelude::*;

#[derive(Deserialize)]
struct ApiResponse {
    id: String,
    output_url: String,
}

#[command]
pub async fn picture(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    if args.is_empty() {
        msg.channel_id
            .say(
                &ctx.http,
                String::from("Use me with \"a sentence, any sentence\""),
            )
            .await?;
        return Ok(());
    }

    let sentence = args.single_quoted::<String>()?;

    let res = {
        let data_read = ctx.data.read().await;
        let bot_config_lock = data_read
            .get::<BotConfig>()
            .expect("Expected BotConfig in TypeMap.")
            .clone();
        let bot_config = bot_config_lock.read().await;
        let client = reqwest::Client::new();
        client
            .post("https://api.deepai.org/api/text2img")
            .header(
                "api-key",
                bot_config.get::<str>(&constants::DEEPAI_TOKEN_KEY).unwrap(),
            )
            .body(sentence)
            .send()
            .await?
            .json::<ApiResponse>()
            .await?
    };

    msg.channel_id.say(&ctx.http, res.output_url).await?;

    Ok(())
}

#[group]
#[prefix = "imagine"]
#[description = "Commands to synthesize things via ML"]
#[commands(picture)]
struct Imagine;
