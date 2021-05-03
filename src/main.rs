mod commands;
mod constants;
mod endpoints;
mod models;
mod repos;
mod utils;

use std::env;

use serenity::async_trait;
use serenity::client::{Client, EventHandler};
use serenity::framework::standard::StandardFramework;

use crate::models::bot_config::BotConfig;
use commands::{
    audio::AUDIO_GROUP, fight::FIGHT_GROUP, general::GENERAL_GROUP, imagine::IMAGINE_GROUP,
    spam::SPAM_GROUP,
};
use endpoints::filters;
use songbird::SerenityInit;
use std::collections::HashMap;
use std::sync::Arc;

use crate::constants::AWS_RESOURCE_REGION;
use crate::models::db::FightUserRepoHolder;
use crate::models::soundboard_map::SoundboardMap;
use crate::repos::fight_user::FightUserDDBRepository;
use dynomite::retry::Policy;
use dynomite::{
    dynamodb::{DynamoDb, DynamoDbClient, ListTablesInput},
    Retries,
};
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
        .group(&FIGHT_GROUP)
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

    // TODO: replace with real config library?
    let mut bot_config: HashMap<String, String> = HashMap::default();
    let deepai_token = env::var("DEEPAI_TOKEN").expect("deepai token");

    bot_config.insert(
        constants::DEEPAI_TOKEN_KEY.to_string(),
        deepai_token.to_string(),
    );

    let mapping = utils::audio::load_files();

    let fight_user_table_name = env::var("FIGHT_USERS_TABLE_NAME").expect("fight user table name");
    let ddb_client = DynamoDbClient::new(AWS_RESOURCE_REGION).with_retries(Policy::default());
    let fight_user_repo = Arc::new(FightUserDDBRepository::new_with_client(
        &ddb_client,
        fight_user_table_name.as_str(),
    ));

    {
        // Open the data lock in write mode, so keys can be inserted to it.
        let mut data = client.data.write().await;

        data.insert::<BotConfig>(Arc::new(RwLock::new(bot_config)));
        data.insert::<SoundboardMap>(Arc::new(RwLock::new(mapping)));
        data.insert::<FightUserRepoHolder>(fight_user_repo);
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
