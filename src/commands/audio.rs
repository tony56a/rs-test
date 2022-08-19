use crate::models::holders::SoundboardMap;
use crate::utils::audio::{combine_files, generate_tts_file};
use crate::utils::chat::log_msg_err;
use image::EncodableLayout;
use reqwest;
use serde::{Deserialize, Serialize};
use serenity::framework::standard::{
    macros::{command, group},
    Args, CommandResult,
};
use serenity::model::prelude::*;
use serenity::prelude::*;
use serenity::utils::MessageBuilder;
use std::collections::HashMap;
use std::fs::File;
use std::io::copy;
use std::path::PathBuf;
use std::{fs, thread, time};
use tokio::time::{timeout, Duration};

#[derive(Serialize)]
struct SpeechGenerationRequest {
    text: String,
    character: String,
    emotion: String,
}

#[derive(Deserialize)]
struct SpeechGenerationResponse {
    wavNames: Vec<String>,
}

#[command]
#[bucket = "audio"]
#[only_in(guilds)]
async fn clip(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let guild = msg.guild(&ctx.cache).await.unwrap();
    let guild_id = guild.id;

    let mapping = get_soundboard_mapping(ctx).await;

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

    let channel_id = check_voice_channels(&guild, &voice_channel).await;
    if channel_id.is_none() {
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

    play_clip_in_guild(
        ctx,
        msg,
        guild_id,
        &mapping[&*clip_name],
        &channel_id.unwrap(),
    )
    .await;

    Ok(())
}

#[command("clip")]
#[bucket = "audio"]
#[only_in(guilds)]
async fn speak_clip(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let guild = msg.guild(&ctx.cache).await.unwrap();
    let guild_id = guild.id;

    let mapping = get_soundboard_mapping(ctx).await;

    if args.is_empty() {
        let response = mapping
            .keys()
            .cloned()
            .into_iter()
            .fold(
                MessageBuilder::new()
                    .push("Use me with \"some text\" \"the clip name\" \"the channel\"! Valid Clips are:\n "),
                |cb, clip_name| cb.push(format!("\t* {}\n", clip_name)),
            )
            .build();
        log_msg_err(msg.channel_id.say(&ctx.http, response).await);
        return Ok(());
    }

    let tts_text = args.single_quoted::<String>().unwrap();
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

    let tts_file = generate_tts_file(tts_text.as_str()).unwrap();
    let combined_file = combine_files(&mapping[&*clip_name], &tts_file).unwrap();

    let channel_id = check_voice_channels(&guild, &voice_channel).await;
    if channel_id.is_none() {
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

    play_clip_in_guild(ctx, msg, guild_id, &combined_file, &channel_id.unwrap()).await;

    fs::remove_file(tts_file).ok();
    fs::remove_file(combined_file).ok();
    Ok(())
}

#[command]
#[bucket = "audio"]
#[sub_commands(speak_clip)]
async fn speak(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let guild = msg.guild(&ctx.cache).await.unwrap();
    let guild_id = guild.id;

    if args.is_empty() {
        let response = MessageBuilder::new()
            .push_line("Use me with \"some text\" \"the channel\"!")
            .build();
        log_msg_err(msg.channel_id.say(&ctx.http, response).await);
        return Ok(());
    }

    let tts_text = args.single_quoted::<String>().unwrap();
    let voice_channel = String::from(args.single_quoted::<String>()?.to_lowercase().trim());

    let tts_file = generate_tts_file(tts_text.as_str()).unwrap();

    let channel_id = check_voice_channels(&guild, &voice_channel).await;
    if channel_id.is_none() {
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

    play_clip_in_guild(ctx, msg, guild_id, &tts_file, &channel_id.unwrap()).await;

    fs::remove_file(tts_file).ok();
    Ok(())
}

#[command]
#[bucket = "audio"]
#[sub_commands(speak_clip)]
async fn say(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let guild = msg.guild(&ctx.cache).await.unwrap();
    let guild_id = guild.id;

    let mapping = get_soundboard_mapping(ctx).await;

    if args.is_empty() {
        let response = MessageBuilder::new()
            .push_line("Use me with \"some text\" \"the channel\"!")
            .build();
        log_msg_err(msg.channel_id.say(&ctx.http, response).await);
        return Ok(());
    }

    let mut tts_text = args.single_quoted::<String>().unwrap();
    tts_text.truncate(200);

    let voice_channel = String::from(args.single_quoted::<String>()?.to_lowercase().trim());
    let channel_id = check_voice_channels(&guild, &voice_channel).await;
    if channel_id.is_none() {
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

    let retrievalRequest = SpeechGenerationRequest {
        text: tts_text,
        character: "GLaDOS".to_string(),
        emotion: "Contextual".to_string(),
    };

    let client = reqwest::Client::new();

    let api_res_payload = client
        .post("https://api.15.ai/app/getAudioFile5")
        .json::<SpeechGenerationRequest>(&retrievalRequest)
        .send()
        .await
        .unwrap();

    let text = api_res_payload.text().await.unwrap();

    println!("15.ai API response {}", text);
    let api_res: SpeechGenerationResponse = serde_json::from_str(&text)?;

    let response_file_name = api_res.wavNames[0].clone();
    let url = format!("https://cdn.15.ai/audio/{}", response_file_name);
    let response_file = reqwest::get(&url).await?;

    let fname = PathBuf::from(format!("/tmp/{}", response_file_name));

    let mut tts_file = File::create(&fname)?;
    let content = response_file.bytes().await?;
    copy(&mut content.as_bytes(), &mut tts_file)?;

    let combined_file = combine_files(&fname, &mapping["sponsor"]).unwrap();

    play_clip_in_guild(ctx, msg, guild_id, &combined_file, &channel_id.unwrap()).await;

    fs::remove_file(&fname).ok();
    fs::remove_file(combined_file).ok();
    Ok(())
}

#[group]
#[prefix = "audio"]
#[description = "Commands to audio"]
#[commands(clip, speak, say)]
struct Audio;

async fn play_clip_in_guild(
    ctx: &Context,
    msg: &Message,
    guild_id: GuildId,
    source_path: &PathBuf,
    channel_id: &ChannelId,
) {
    let source = match songbird::ffmpeg(source_path.to_str().unwrap()).await {
        Ok(source) => source,
        Err(why) => {
            println!("Err starting source: {:?}", why);
            log_msg_err(msg.channel_id.say(&ctx.http, "Error sourcing ffmpeg").await);
            return;
        }
    };

    let manager = songbird::get(ctx)
        .await
        .expect("Songbird Voice client placed in at initialisation.")
        .clone();

    let _ = manager.join(guild_id, channel_id.as_u64().clone()).await;

    if let Some(handler_lock) = manager.get(guild_id) {
        let mut handler = handler_lock.lock().await;
        let duration = (&source.metadata.duration).unwrap() + time::Duration::from_millis(500);
        let handle = handler.play_only_source(source);

        // Delay until end of the clip + 500ms (for remaining audio packets or something)
        let _ = handle.get_info().await;
        thread::sleep(duration);
    } else {
        log_msg_err(
            msg.channel_id
                .say(&ctx.http, "Not in a voice channel to play in")
                .await,
        );
        return;
    }

    if manager.get(guild_id).is_some() {
        loop {
            if let Err(e) = timeout(Duration::from_secs(1), manager.remove(guild_id)).await {
                log_msg_err(
                    msg.channel_id
                        .say(&ctx.http, format!("Failed: {:?}", e))
                        .await,
                );
            } else {
                break;
            }
        }
    } else {
        log_msg_err(msg.channel_id.say(ctx, "Not in a voice channel").await);
        return;
    }
}

async fn check_voice_channels(guild: &Guild, voice_channel: &String) -> Option<ChannelId> {
    let voice_channels: HashMap<String, ChannelId> = guild
        .channels
        .values()
        .into_iter()
        .filter_map(|channel| match channel.kind {
            ChannelType::Voice => Some(((&channel).name.to_lowercase(), channel.id)),
            _ => None,
        })
        .collect();

    voice_channels.get(voice_channel).map(|value| value.clone())
}

async fn get_soundboard_mapping(ctx: &Context) -> HashMap<String, PathBuf> {
    let data_read = ctx.data.read().await;
    let mapping_lock = data_read
        .get::<SoundboardMap>()
        .expect("Expected Soundboard mapping in TypeMap.")
        .clone();
    let mapping = mapping_lock.read().await;
    mapping.clone()
}
