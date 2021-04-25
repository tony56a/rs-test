use serenity::prelude::*;
use serenity::model::prelude::*;
use serenity::framework::standard::{
    Args,
    CommandResult,
    macros::{command, group},
};
use serenity::utils::MessageBuilder;
use tokio_stream::{StreamExt, Iter};

#[command]
#[bucket = "spam"]
pub async fn respect(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    if args.is_empty() {
        msg.channel_id.say(&ctx.http, String::from("Use me with @user (number of times to spam)")).await?;
        return Ok(());
    }
    let provided_user = args.single::<String>()?;
    let times = args.single::<u32>()?;

    // Infer the
    if msg.mention_everyone || times > 5  {
        return Ok(());
    }
    if msg.mentions.is_empty() {
        msg.channel_id.say(&ctx.http, format!("{user} doesn't exist!", user=provided_user)).await?;
        return Ok(());
    }
    if msg.mentions[0].bot {
        msg.channel_id.say(&ctx.http, format!("Disrespect bots {times} times", times=times.to_string())).await?;
        return Ok(());
    }

    let response = MessageBuilder::new()
        .push("!respect ")
        .mention(&msg.mentions[0])
        .push(" for ...something, IDK :shrug:")
        .build();

    let mut iterator_vec: Vec<u32> = (0..times).collect();
    let mut message_stream = tokio_stream::iter(iterator_vec);
    while let Some(v) = message_stream.next().await {
        let result = msg.channel_id.say(&ctx.http, &response).await;

    }

    Ok(())
}

#[group]
#[prefix = "spam"]
#[commands(respect)]
struct Spam;
