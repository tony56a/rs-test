use serenity::framework::standard::{
    macros::{command, group},
    CommandResult,
};
use serenity::model::prelude::*;
use serenity::prelude::*;

#[command]
pub async fn ping(ctx: &Context, msg: &Message) -> CommandResult {
    msg.channel_id.say(&ctx.http, "Pong!").await?;
    Ok(())
}

#[group]
#[description = "General group of commands"]
#[commands(ping)]
struct General;
