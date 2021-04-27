mod commands;
mod constants;
mod endpoints;
mod models;

use std::env;

use serenity::async_trait;
use serenity::client::{Client, EventHandler};
use serenity::framework::standard::StandardFramework;

use crate::models::bot_config::BotConfig;
use commands::{
    attack::ATTACK_GROUP, audio::AUDIO_GROUP, general::GENERAL_GROUP, imagine::IMAGINE_GROUP,
    spam::SPAM_GROUP,
};
use endpoints::filters;
use songbird::SerenityInit;
use std::collections::HashMap;
use std::sync::Arc;

use tokio::sync::RwLock;

struct Handler;

#[async_trait]
impl EventHandler for Handler {}

#[tokio::main]
async fn main() {
    // setup the discord bot
    let framework = StandardFramework::new()
        .configure(|c| {
            c.prefix("?")
                .delimiters(vec![", ", ",", " "])
                .with_whitespace(true)
        }) // set the bot's prefix to "?"
        .bucket("spam", |b| b.limit(1).time_span(2))
        .await
        .group(&GENERAL_GROUP)
        .group(&SPAM_GROUP)
        .group(&ATTACK_GROUP)
        .group(&AUDIO_GROUP)
        .group(&IMAGINE_GROUP);

    // Login with a bot token from the environment
    let token = env::var("DISCORD_TOKEN").expect("discord token");
    let mut client = Client::builder(token)
        .event_handler(Handler)
        .framework(framework)
        .register_songbird()
        .await
        .expect("Error creating client");

    {
        // TODO: replace with real config library?
        let mut bot_config: HashMap<String, String> = HashMap::default();
        let deepai_token = env::var("DEEPAI_TOKEN").expect("deepai token");

        bot_config.insert(
            constants::DEEPAI_TOKEN_KEY.to_string(),
            deepai_token.to_string(),
        );

        // Open the data lock in write mode, so keys can be inserted to it.
        let mut data = client.data.write().await;

        data.insert::<BotConfig>(Arc::new(RwLock::new(bot_config)));
    }

    // define Server endpoints
    let http_port = env::var("PORT").expect("http_port").parse::<u16>().unwrap();
    let http_service = warp::serve(filters()).run(([0, 0, 0, 0], http_port));

    tokio::spawn(async move {
        http_service.await;
    });

    if let Err(why) = client.start().await {
        panic!("Client error: {:?}", why);
    }
}
