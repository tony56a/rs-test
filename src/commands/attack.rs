use rand::seq::SliceRandom;
use serenity::framework::standard::{
    macros::{command, group},
    Args, CommandResult,
};
use serenity::model::prelude::*;
use serenity::prelude::*;
use serenity::utils::MessageBuilder;

#[command]
pub async fn punch(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    if args.is_empty() {
        msg.channel_id
            .say(&ctx.http, String::from("Use me with @user"))
            .await?;
        return Ok(());
    }

    let _ = args.single::<String>()?;
    if msg.mention_everyone || msg.mentions.is_empty() {
        return Ok(());
    }

    let effectiveness: Vec<String> = vec![
        String::from("super effective!"),
        String::from("not very effective..."),
        String::from("sort of effective?"),
    ];
    let msg_effectiveness = effectiveness.choose(&mut rand::thread_rng()).unwrap();
    let response = MessageBuilder::new()
        .mention(&msg.author)
        .push(" used punch on ")
        .mention(&msg.mentions[0])
        .push(format!("! It's {}", msg_effectiveness))
        .build();

    match msg.channel_id.say(&ctx.http, &response).await {
        Err(e) => println!("Some error during message sending: {:?}", e),
        Ok(_) => {} //Don't care, just do whatever
    }
    Ok(())
}

#[group]
#[prefix = "attack"]
#[description = "Commands to attack other users/bots"]
#[commands(punch)]
struct Attack;
