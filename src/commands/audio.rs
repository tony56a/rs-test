use crate::models::soundboard_map::SoundboardMap;
use crate::utils::chat::log_msg_err;
use serenity::framework::standard::{
    macros::{command, group},
    Args, CommandResult,
};
use serenity::model::prelude::*;
use serenity::prelude::*;
use serenity::utils::MessageBuilder;
use songbird::tracks::PlayMode;
use std::collections::HashMap;
use std::{thread, time};

#[command]
#[only_in(guilds)]
pub async fn join(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let guild = msg.guild(&ctx.cache).await.unwrap();
    let guild_id = guild.id;

    if args.is_empty() {
        log_msg_err(
            msg.channel_id
                .say(
                    &ctx.http,
                    String::from("Use me with \"The channel you want me to join\""),
                )
                .await,
        );
        return Ok(());
    }

    let voice_channel = String::from(args.single_quoted::<String>()?.to_lowercase().trim());

    let voice_channels: HashMap<String, ChannelId> = guild
        .channels
        .values()
        .into_iter()
        .filter_map(|channel| match channel.kind {
            ChannelType::Voice => Some(((&channel).name.to_lowercase(), channel.id)),
            _ => None,
        })
        .collect();

    if !voice_channels.contains_key(&voice_channel) {
        log_msg_err(
            msg.channel_id
                .say(
                    &ctx.http,
                    format!("{channel} doesn't exist!", channel = voice_channel),
                )
                .await,
        );
        return Ok(());
    }
    let manager = songbird::get(ctx).await.expect("Songbird client").clone();

    let _ = manager
        .join(
            guild_id,
            u64::from(voice_channels[&voice_channel.to_lowercase()]),
        )
        .await;

    Ok(())
}

#[command]
#[only_in(guilds)]
pub async fn leave(ctx: &Context, msg: &Message) -> CommandResult {
    let guild = msg.guild(&ctx.cache).await.unwrap();
    let guild_id = guild.id;

    let manager = songbird::get(ctx)
        .await
        .expect("Songbird Voice client placed in at initialisation.")
        .clone();
    let has_handler = manager.get(guild_id).is_some();

    if has_handler {
        if let Err(e) = manager.remove(guild_id).await {
            log_msg_err(
                msg.channel_id
                    .say(&ctx.http, format!("Failed: {:?}", e))
                    .await,
            );
        }
    } else {
        log_msg_err(msg.channel_id.say(ctx, "Not in a voice channel").await);
    }

    Ok(())
}

#[command]
#[only_in(guilds)]
async fn play(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let mapping = {
        let data_read = ctx.data.read().await;
        let mapping_lock = data_read
            .get::<SoundboardMap>()
            .expect("Expected Soundboard mapping in TypeMap.")
            .clone();
        let mapping = mapping_lock.read().await;
        mapping.clone()
    };
    if args.is_empty() {
        let response = mapping
            .keys()
            .cloned()
            .into_iter()
            .fold(
                MessageBuilder::new()
                    .push("Use me with the \"the clip name\"! Valid Clips are:\n "),
                |cb, clip_name| cb.push(format!("\t* {}\n", clip_name)),
            )
            .build();
        log_msg_err(msg.channel_id.say(&ctx.http, response).await);
        return Ok(());
    }

    let clip_name = match args.single_quoted::<String>() {
        Ok(name) => name.to_lowercase(),
        Err(_) => {
            log_msg_err(
                msg.channel_id
                    .say(&ctx.http, "Clip name is not valid!")
                    .await,
            );
            return Ok(());
        }
    };

    if !mapping.contains_key(&*clip_name) {
        log_msg_err(
            msg.channel_id
                .say(&ctx.http, String::from("Effects are not available"))
                .await,
        );
        return Ok(());
    }

    let guild = msg.guild(&ctx.cache).await.unwrap();
    let guild_id = guild.id;

    let manager = songbird::get(ctx)
        .await
        .expect("Songbird Voice client placed in at initialisation.")
        .clone();

    if let Some(handler_lock) = manager.get(guild_id) {
        let mut handler = handler_lock.lock().await;
        let source = match songbird::ffmpeg(mapping[&*clip_name].to_str().unwrap()).await {
            Ok(source) => source,
            Err(why) => {
                println!("Err starting source: {:?}", why);

                log_msg_err(msg.channel_id.say(&ctx.http, "Error sourcing ffmpeg").await);

                return Ok(());
            }
        };

        handler.play_only_source(source);
    } else {
        log_msg_err(
            msg.channel_id
                .say(&ctx.http, "Not in a voice channel to play in")
                .await,
        );
    }

    Ok(())
}

#[command]
#[only_in(guilds)]
async fn clip(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let guild = msg.guild(&ctx.cache).await.unwrap();
    let guild_id = guild.id;

    let mapping = {
        let data_read = ctx.data.read().await;
        let mapping_lock = data_read
            .get::<SoundboardMap>()
            .expect("Expected Soundboard mapping in TypeMap.")
            .clone();
        let mapping = mapping_lock.read().await;
        mapping.clone()
    };

    if args.is_empty() {
        let response = mapping
            .keys()
            .cloned()
            .into_iter()
            .fold(
                MessageBuilder::new()
                    .push("Use me with \"the clip name\" \"the channel\"! Valid Clips are:\n "),
                |cb, clip_name| cb.push(format!("\t* {}\n", clip_name)),
            )
            .build();
        log_msg_err(msg.channel_id.say(&ctx.http, response).await);
        return Ok(());
    }

    let clip_name = match args.single_quoted::<String>() {
        Ok(name) => name.to_lowercase(),
        Err(_) => {
            log_msg_err(
                msg.channel_id
                    .say(&ctx.http, "Clip name is not valid!")
                    .await,
            );
            return Ok(());
        }
    };
    let voice_channel = String::from(args.single_quoted::<String>()?.to_lowercase().trim());

    if !mapping.contains_key(&*clip_name) {
        log_msg_err(
            msg.channel_id
                .say(&ctx.http, String::from("Clip name is not available"))
                .await,
        );
        return Ok(());
    }

    let voice_channels: HashMap<String, ChannelId> = guild
        .channels
        .values()
        .into_iter()
        .filter_map(|channel| match channel.kind {
            ChannelType::Voice => Some(((&channel).name.to_lowercase(), channel.id)),
            _ => None,
        })
        .collect();

    if !voice_channels.contains_key(&voice_channel) {
        log_msg_err(
            msg.channel_id
                .say(
                    &ctx.http,
                    format!("{channel} doesn't exist!", channel = voice_channel),
                )
                .await,
        );
        return Ok(());
    }
    let manager = songbird::get(ctx)
        .await
        .expect("Songbird Voice client placed in at initialisation.")
        .clone();

    let _ = manager
        .join(guild_id, u64::from(voice_channels[&voice_channel]))
        .await;

    if let Some(handler_lock) = manager.get(guild_id) {
        let mut handler = handler_lock.lock().await;
        let source = match songbird::ffmpeg(mapping[&*clip_name].to_str().unwrap()).await {
            Ok(source) => source,
            Err(why) => {
                println!("Err starting source: {:?}", why);
                log_msg_err(msg.channel_id.say(&ctx.http, "Error sourcing ffmpeg").await);

                return Ok(());
            }
        };

        let handle = handler.play_only_source(source);

        // Delay until end of the clip + 500ms (for remaining audio packets or something)
        while handle.get_info().await?.playing != PlayMode::End {}
        thread::sleep(time::Duration::from_millis(500));
    } else {
        log_msg_err(
            msg.channel_id
                .say(&ctx.http, "Not in a voice channel to play in")
                .await,
        );
    }

    let has_handler = manager.get(guild_id).is_some();
    if has_handler {
        if let Err(e) = manager.remove(guild_id).await {
            log_msg_err(
                msg.channel_id
                    .say(&ctx.http, format!("Failed: {:?}", e))
                    .await,
            );
        }
    } else {
        log_msg_err(msg.channel_id.say(ctx, "Not in a voice channel").await);
    }

    Ok(())
}

#[group]
#[prefix = "audio"]
#[description = "Commands to audio"]
#[commands(join, leave, play, clip)]
struct Audio;
