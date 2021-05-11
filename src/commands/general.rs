use crate::utils::chat::log_msg_err;
use serenity::framework::standard::{
    macros::{command, group},
    CommandResult,
};
use serenity::model::prelude::*;
use serenity::prelude::*;
use crate::models::holders::UserQuoteRepoHolder;
use crate::repos::quotes::UserQuoteRepository;
use serenity::utils::MessageBuilder;
use crate::models::user_quote::UserQuote;

#[command]
async fn ping(ctx: &Context, msg: &Message) -> CommandResult {
    log_msg_err(msg.channel_id.say(&ctx.http, "Pong!").await);
    Ok(())
}

#[command]
async fn quote(ctx: &Context, msg: &Message) -> CommandResult {

    let server_name = msg
        .guild_id
        .expect("Guild ID should be present")
        .0
        .to_string();

    {
        let data_read = ctx.data.read().await;
        let repository = data_read
            .get::<UserQuoteRepoHolder>()
            .expect("Expected repository in TypeMap.")
            .clone();

        let existing_quote_fetched = repository
            .get_random_quote(&server_name)
            .await;

        let existing_quote = match existing_quote_fetched {
            None => {
                println!("No quotes found");
                return Ok(())
            }
            Some(quote) => {quote}
        };

        let response = MessageBuilder::new()
            .push_quote_line(existing_quote.message_content)
            .push("- ")
            .mention(&existing_quote.author_id.parse::<UserId>().unwrap())
            .build();

        log_msg_err(
            msg.channel_id
                .say(&ctx.http, response)
                .await,
        );

    }

    Ok(())
}

#[group]
#[description = "General group of commands"]
#[commands(ping, quote)]
struct General;
