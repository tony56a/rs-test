mod commands;
mod endpoints;

use std::env;

use serenity::async_trait;
use serenity::client::{Client, EventHandler};
use serenity::framework::standard::StandardFramework;

use commands::{
    attack::ATTACK_GROUP, audio::AUDIO_GROUP, general::GENERAL_GROUP, spam::SPAM_GROUP,
};
use endpoints::filters;
use songbird::SerenityInit;

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
        .group(&AUDIO_GROUP);

    // Login with a bot token from the environment
    let token = env::var("DISCORD_TOKEN").expect("token");
    let mut client = Client::builder(token)
        .event_handler(Handler)
        .framework(framework)
        .register_songbird()
        .await
        .expect("Error creating client");

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
