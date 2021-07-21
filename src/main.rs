mod commands;
mod constants;
mod endpoints;
mod models;
mod repos;
mod utils;

use std::env;

use serenity::async_trait;
use serenity::client::{Client, EventHandler, Context};
use serenity::framework::standard::StandardFramework;

use commands::{
    admin::ADMIN_GROUP, audio::AUDIO_GROUP, bot::BOT_GROUP, fight::FIGHT_GROUP,
    general::GENERAL_GROUP, imagine::IMAGINE_GROUP, spam::SPAM_GROUP
};
use endpoints::filters;
use songbird::SerenityInit;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use crate::constants::AWS_RESOURCE_REGION;
use crate::models::holders::{
    BotConfig, FightUserRepoHolder, FightWeaponRepoHolder, Owners, SoundboardMap,
    UserQuoteRepoHolder,
};
use crate::repos::fight_user::FightUserDDBRepository;
use crate::repos::fight_weapon::FightWeaponDDBRepository;
use crate::repos::quotes::UserQuoteDDBRepository;
use dynomite::retry::Policy;
use dynomite::{dynamodb::DynamoDbClient, Retries};
use serenity::http::Http;
use tokio::sync::RwLock;
use serenity::model::channel::Message;
use crate::commands::general;
use serenity::utils::MessageBuilder;
use crate::utils::chat::log_msg_err;

struct Handler;

#[async_trait]
impl EventHandler for Handler {

    async fn message(&self, ctx: Context, msg: Message) {
        println!("{}", msg.content.to_lowercase().starts_with(("i'm")));
        if msg.content.to_lowercase().starts_with("i'm") {
            let current_user = ctx.http.get_current_application_info().await.unwrap();

            let response = MessageBuilder::new()
                .push("Hi, ")
                .push(msg.content
                    .replace("i'm", "")
                    .replace("I'M", "")
                    .replace("i'M", "")
                    .replace("I'm", "")
                    .trim())
                .push(", I'm")
                .mention(&current_user.id)
                .build();

            log_msg_err(msg.channel_id.say(&ctx.http, response).await);
        }
    }
}

#[tokio::main]
async fn main() {
    // setup the discord bot
    // Login with a bot token from the environment
    let token = env::var("DISCORD_TOKEN").expect("discord token");

    let http = Http::new_with_token(&token);

    let owners = match http.get_current_application_info().await {
        Ok(info) => {
            let mut owners = HashSet::new();
            if let Some(team) = info.team {
                owners.insert(team.owner_user_id);
            } else {
                owners.insert(info.owner.id);
            }
            match http.get_current_user().await {
                Ok(_) => owners,
                Err(why) => panic!("Could not access the bot id: {:?}", why),
            }
        }
        Err(why) => panic!("Could not access application info: {:?}", why),
    };

    let framework = StandardFramework::new()
        .configure(|c| {
            c.prefix("?")
                .delimiters(vec![", ", ",", " "])
                .with_whitespace(true)
                .case_insensitivity(true)
                .owners(owners.clone())
        }) // set the bot's prefix to "?"
        .bucket("spam", |b| b.limit(1).time_span(2))
        .await
        .bucket("audio", |b| b.limit(1).time_span(2))
        .await
        .group(&GENERAL_GROUP)
        .group(&SPAM_GROUP)
        .group(&FIGHT_GROUP)
        .group(&AUDIO_GROUP)
        .group(&ADMIN_GROUP)
        .group(&IMAGINE_GROUP)
        .group(&BOT_GROUP);

    let mut client = Client::builder(token)
        .event_handler(Handler)
        .framework(framework)
        .register_songbird()
        .await
        .expect("Error creating client");

    // TODO: replace with real config library?
    let mut bot_config: HashMap<String, String> = HashMap::default();
    let deepai_token = env::var("DEEPAI_TOKEN").expect("deepai token");
    let current_revision = option_env!("CIRCLE_SHA1")
        .unwrap_or(constants::DEVELOPMENT_BUILD)
        .to_string();

    bot_config.insert(constants::DEEPAI_TOKEN_KEY.to_string(), deepai_token);
    bot_config.insert(constants::BUILD_HASH_KEY.to_string(), current_revision);

    let mapping = utils::audio::load_files();

    let fight_user_table_name = env::var("FIGHT_USERS_TABLE_NAME").expect("fight user table name");
    let fight_weapons_table_name =
        env::var("FIGHT_WEAPONS_TABLE_NAME").expect("fight weapon table name");
    let user_quote_table_mame = env::var("USER_QUOTE_TABLE_NAME").expect("user quotetable name");

    let ddb_client = DynamoDbClient::new(AWS_RESOURCE_REGION).with_retries(Policy::default());
    let fight_user_repo = Arc::new(FightUserDDBRepository::new_with_client(
        &ddb_client,
        fight_user_table_name.as_str(),
    ));
    let fight_weapon_repo = Arc::new(FightWeaponDDBRepository::new_with_client(
        &ddb_client,
        fight_weapons_table_name.as_str(),
    ));
    let user_quote_repo = Arc::new(UserQuoteDDBRepository::new_with_client(
        &ddb_client,
        user_quote_table_mame.as_str(),
    ));

    {
        // Open the data lock in write mode, so keys can be inserted to it.
        let mut data = client.data.write().await;

        data.insert::<BotConfig>(Arc::new(RwLock::new(bot_config)));
        data.insert::<SoundboardMap>(Arc::new(RwLock::new(mapping)));
        data.insert::<FightUserRepoHolder>(fight_user_repo);
        data.insert::<FightWeaponRepoHolder>(fight_weapon_repo);
        data.insert::<UserQuoteRepoHolder>(user_quote_repo);
        data.insert::<Owners>(Arc::new(RwLock::new(owners)));
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
