use crate::utils::chat::log_msg_err;
use serenity::framework::standard::{
    macros::{command, group},
    Args, CommandResult,
};
use serenity::model::prelude::*;
use serenity::prelude::*;
use serenity::utils::MessageBuilder;
use tokio_stream::StreamExt;

#[command]
#[bucket = "spam"]
pub async fn respect(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    if args.is_empty() {
        log_msg_err(
            msg.channel_id
                .say(
                    &ctx.http,
                    String::from("Use me with @user (number of times to spam)"),
                )
                .await,
        );
        return Ok(());
    }
    let provided_user = args.single::<String>()?;
    let times = args.single::<u32>()?;

    // Validate based on inferred mentions (also cap number of times to spam)
    // (i.e: 1 user mention specfically, and can't be a bot/broadcast)
    // TODO: read up docs and find out how to do this correctly
    if msg.mention_everyone || times > 5 {
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
    if msg.mentions[0].bot {
        log_msg_err(
            msg.channel_id
                .say(
                    &ctx.http,
                    format!("Disrespect bots {times} times", times = times.to_string()),
                )
                .await,
        );
        return Ok(());
    }

    // Once everything's been "validated", create a stream (async iterator)
    // and spam the message
    let response = MessageBuilder::new()
        .push("!respect ")
        .mention(&msg.mentions[0])
        .push(" for ...something, IDK :shrug:")
        .build();

    let iterator_vec: Vec<u32> = (0..times).collect();
    let mut message_stream = tokio_stream::iter(iterator_vec);
    while let Some(_) = message_stream.next().await {
        log_msg_err(msg.channel_id.say(&ctx.http, &response).await)
    }

    Ok(())
}

#[group]
#[prefix = "spam"]
#[description = "Commands to spam other users/bots"]
#[commands(respect)]
struct Spam;
