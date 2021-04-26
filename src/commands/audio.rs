use rand::seq::SliceRandom;
use serenity::framework::standard::{
    macros::{command, group},
    Args, CommandResult,
};
use serenity::model::prelude::*;
use serenity::prelude::*;
use serenity::utils::MessageBuilder;
use std::collections::HashMap;

#[command]
#[only_in(guilds)]
pub async fn join(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let guild = msg.guild(&ctx.cache).await.unwrap();
    let guild_id = guild.id;

    if args.is_empty() {
        msg.channel_id
            .say(
                &ctx.http,
                String::from("Use me with \"The channel you want me to join\""),
            )
            .await?;
        return Ok(());
    }

    let voice_channel = args.single_quoted::<String>()?;

    let voice_channels: HashMap<String, ChannelId> = guild
        .channels
        .values()
        .into_iter()
        .filter_map(|channel| match channel.kind {
            ChannelType::Voice => Some(((&channel).name.to_lowercase(), channel.id)),
            _ => None,
        })
        .collect();

    //let voice_channels: HashMap<String, ChannelId>  = voice_channels_vec.into_iter().collect();

    if !voice_channels.contains_key(&voice_channel.to_lowercase()) {
        println!("{:?}", voice_channels);
        msg.channel_id
            .say(
                &ctx.http,
                format!("{channel} doesn't exist!", channel = voice_channel),
            )
            .await?;
        return Ok(());
    }
    let manager = songbird::get(ctx).await.expect("Songbird client").clone();

    let _ = manager
        .join(guild_id, u64::from(voice_channels[&voice_channel]))
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
            msg.channel_id
                .say(&ctx.http, format!("Failed: {:?}", e))
                .await;
        }

        msg.channel_id.say(&ctx.http, "Left voice channel").await;
    } else {
        msg.reply(ctx, "Not in a voice channel").await;
    }

    Ok(())
}

#[command]
#[only_in(guilds)]
async fn play(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let mapping: HashMap<&str, &str> = [
        (
            "that was easy",
            "https://www.youtube.com/watch?v=OE_d8UCiXcI",
        ),
        ("yeet", "https://www.youtube.com/watch?v=EwlM3kpqEo0"),
        ("bruh", "https://www.youtube.com/watch?v=2ZIpFytCSVc"),
        ("fart", "https://www.youtube.com/watch?v=dEOjOkHSShM"),
    ]
    .iter()
    .cloned()
    .collect();

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
        msg.channel_id.say(&ctx.http, response).await?;
        return Ok(());
    }

    let clip_name = match args.single_quoted::<String>() {
        Ok(name) => name.to_lowercase(),
        Err(_) => {
            msg.channel_id
                .say(&ctx.http, "Clip name is not valid!")
                .await;
            return Ok(());
        }
    };

    if !mapping.contains_key(&*clip_name) {
        msg.channel_id
            .say(&ctx.http, String::from("Effects are not available"))
            .await?;
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
        let source = match songbird::ytdl(mapping[&*clip_name]).await {
            Ok(source) => source,
            Err(why) => {
                println!("Err starting source: {:?}", why);

                msg.channel_id.say(&ctx.http, "Error sourcing ffmpeg").await;

                return Ok(());
            }
        };

        handler.play_source(source);

        msg.channel_id.say(&ctx.http, "Playing song").await;
    } else {
        msg.channel_id
            .say(&ctx.http, "Not in a voice channel to play in")
            .await;
    }

    Ok(())
}

#[group]
#[prefix = "audio"]
#[description = "Commands to audio"]
#[commands(join, leave, play)]
struct Audio;
