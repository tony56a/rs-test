use crate::models::db::FightUserRepoHolder;
use crate::models::fight_effectiveness::AttackEffectiveness;
use crate::models::fight_user::FightUser;
use crate::models::fight_weapon::FightWeapon;
use crate::repos::fight_user::FightUserRepository;
use crate::utils::chat::log_msg_err;
use serenity::framework::standard::{
    macros::{command, group},
    Args, CommandResult,
};
use serenity::model::prelude::*;
use serenity::prelude::*;
use serenity::utils::MessageBuilder;

const DEFAULT_HITPOINTS: f64 = 100.0;

fn get_user_data<'a>(
    msg: &'a Message,
    args: &'a mut Args,
    allow_mention: bool,
) -> (Option<&'a User>, String) {
    let user_to_query = if args.is_empty() || !allow_mention {
        &msg.author
    } else {
        let _ = args.single::<String>();
        if msg.mention_everyone || msg.mentions.is_empty() {
            return (None, String::default());
        }
        &msg.mentions[0]
    };

    let server_name = msg
        .guild_id
        .expect("Guild ID should be present")
        .0
        .to_string();

    (Some(user_to_query), server_name)
}

#[command]
pub async fn spawn(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let (user_to_create_result, server_name) = get_user_data(msg, &mut args, true);

    let user_to_create = match user_to_create_result {
        None => return Ok(()),
        Some(result) => result,
    };

    let user_to_create_id = user_to_create.id.to_string();

    let fight_user = FightUser::new(user_to_create_id.as_str(), server_name.as_str(), DEFAULT_HITPOINTS);

    {
        let data_read = ctx.data.read().await;
        let repository = data_read
            .get::<FightUserRepoHolder>()
            .expect("Expected Fight user repository in TypeMap.")
            .clone();

        let existing_user = repository
            .get_fight_user(&user_to_create_id, &server_name)
            .await;
        if existing_user.is_some() {
            log_msg_err(msg.channel_id.say(&ctx.http, "User already exists!").await);
            return Ok(());
        }

        repository.create_fight_user(&fight_user).await;
        log_msg_err(msg.channel_id.say(&ctx.http, "User created!").await);
    }

    Ok(())
}

#[command]
pub async fn status(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let (user_to_query_result, server_name) = get_user_data(msg, &mut args, true);

    let user_to_query = match user_to_query_result {
        None => return Ok(()),
        Some(result) => result,
    };

    let user_to_query_id = user_to_query.id.to_string();

    let user = {
        let data_read = ctx.data.read().await;
        let repository = data_read
            .get::<FightUserRepoHolder>()
            .expect("Expected Fight user repository in TypeMap.")
            .clone();

        let existing_user: Option<FightUser> = repository
            .get_fight_user(&user_to_query_id, &server_name)
            .await;
        if existing_user.is_none() {
            let response = MessageBuilder::new()
                .mention(user_to_query)
                .push(" isn't in the game!")
                .build();
            log_msg_err(msg.channel_id.say(&ctx.http, response).await);
            return Ok(());
        }
        existing_user.unwrap()
    };

    let mut mb = MessageBuilder::new();
    let mut response = mb
        .push("User status for: ")
        .mention(user_to_query)
        .push("\n")
        .push_line(format!("Hitpoints: {:.2}", user.hitpoints));

    match user.weapon {
        None => {}
        Some(weapon) => {
            response = response.push_line(format!("Weapon: {}", weapon.name));
        }
    }

    if user.knocked_out.len() > 0 {
        response = response.push_line("Knocked out: ");
        response = user
            .knocked_out
            .into_iter()
            .fold(response, |cb, user_id| {
                cb.push("\t*")
                    .mention(&user_id.parse::<UserId>().unwrap())
                    .push_line("")
            });
    }

    if user.knocked_out_by.len() > 0 {
        response = response.push_line("Knocked out by: ");
        response = user
            .knocked_out_by
            .into_iter()
            .fold(response, |cb, user_id| {
                cb.push("\t*")
                    .mention(&user_id.parse::<UserId>().unwrap())
                    .push_line("")
            });
    }

    log_msg_err(msg.channel_id.say(&ctx.http, &mb.build()).await);

    Ok(())
}

#[command]
pub async fn attack(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    if args.is_empty() {
        log_msg_err(msg.channel_id.say(&ctx.http, "Use me with @user".to_string()).await);
        return Ok(());
    }

    let (user_to_query_result, server_name) = get_user_data(msg, &mut args, true);

    let user_to_query = match user_to_query_result {
        None => return Ok(()),
        Some(result) => result,
    };

    let attacking_user_id = &msg.author.id;
    let attacked_user_id = &user_to_query.id;

    if attacking_user_id == attacked_user_id {
        log_msg_err(msg.channel_id.say(&ctx.http, "Stop hitting yourself!").await);
        return Ok(());
    }

    let (attacking_user, attacked_user): (FightUser, FightUser) = {
        let data_read = ctx.data.read().await;
        let repository = data_read
            .get::<FightUserRepoHolder>()
            .expect("Expected Fight user repository in TypeMap.")
            .clone();

        let attacking_user = repository
            .get_fight_user(&attacking_user_id.to_string(), &server_name)
            .await;
        if attacking_user.is_none() {
            let response = MessageBuilder::new()
                .mention(attacking_user_id)
                .push(" isn't in the game!")
                .build();
            log_msg_err(msg.channel_id.say(&ctx.http, response).await);
            return Ok(());
        }

        let attacked_user = repository
            .get_fight_user(&attacked_user_id.to_string(), &server_name)
            .await;
        if attacked_user.is_none() {
            let response = MessageBuilder::new()
                .mention(attacked_user_id)
                .push(" isn't in the game!")
                .build();
            log_msg_err(msg.channel_id.say(&ctx.http, response).await);
            return Ok(());
        }
        (attacking_user.unwrap(), attacked_user.unwrap())
    };

    if (&attacked_user).hitpoints <= 0.0 {
        let response = MessageBuilder::new()
            .mention(user_to_query)
            .push(" has already been knocked out!")
            .build();

        log_msg_err(msg.channel_id.say(&ctx.http, response).await);
        return Ok(())
    }

    let attack_effectiveness = rand::random::<AttackEffectiveness>();
    let attacking_weapon = &(attacking_user).weapon.clone().unwrap_or(FightWeapon {
        name: "fists".to_string(),
        attack_val: 5.0,
    });
    let attack_value: f64 = attacking_weapon.attack_val * attack_effectiveness.value_multiplier();

    let attacked_new_hitpoints = attacked_user.hitpoints - attack_value;
    let (new_attacking_user, new_attacked_user) = {
        if attacked_new_hitpoints <= 0.0 {
            let mut attacked_knocked_out_by_vec = attacked_user.knocked_out_by.clone();
            attacked_knocked_out_by_vec.push(attacking_user_id.to_string());
            if attacked_knocked_out_by_vec.len() > 10 {
                attacked_knocked_out_by_vec = attacked_knocked_out_by_vec[1..].to_owned();
            }

            let mut attacking_knocked_out_vec = attacking_user.knocked_out.clone();
            attacking_knocked_out_vec.push(attacked_user_id.to_string());
            if attacking_knocked_out_vec.len() > 10 {
                attacking_knocked_out_vec = attacking_knocked_out_vec[1..].to_owned();
            }

            let knocked_out_msg = MessageBuilder::new()
                .mention(attacking_user_id)
                .push(" knocked out ")
                .mention(attacked_user_id)
                .push_line(format!("!"))
                .build();
            log_msg_err(msg.channel_id.say(&ctx.http, knocked_out_msg).await);

            (
                FightUser {
                    knocked_out: attacking_knocked_out_vec,
                    ..attacking_user.clone()
                },
                FightUser {
                    knocked_out_by: attacked_knocked_out_by_vec,
                    hitpoints: 0.0,
                    ..attacked_user.clone()
                },
            )
        } else {
            (
                attacking_user.clone(),
                FightUser {
                    hitpoints: attacked_new_hitpoints,
                    ..attacked_user
                },
            )
        }
    };

    {
        let data_read = ctx.data.read().await;
        let repository = data_read
            .get::<FightUserRepoHolder>()
            .expect("Expected Fight user repository in TypeMap.")
            .clone();

        repository
            .update_fight_user(
                &new_attacked_user.user_id,
                server_name.as_str(),
                &new_attacked_user,
            )
            .await?;
        repository
            .update_fight_user(
                &new_attacking_user.user_id,
                server_name.as_str(),
                &new_attacking_user,
            )
            .await?;
    }

    let attack_summary = MessageBuilder::new()
        .mention(attacking_user_id)
        .push(" attacked ")
        .mention(attacked_user_id)
        .push(format!(" with {}, ", attacking_weapon.name))
        .push_line(format!(
            "it was {} Did {:.2} points of damage",
            attack_effectiveness.msg_value(),
            attack_value
        ))
        .build();
    log_msg_err(msg.channel_id.say(&ctx.http, attack_summary).await);

    Ok(())
}

#[command]
pub async fn revive(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {

    let (user_to_query_result, server_name) = get_user_data(msg, &mut args, true);
    let user_to_query = match user_to_query_result {
        None => return Ok(()),
        Some(result) => result,
    };

    let queried_user: FightUser = {
        let data_read = ctx.data.read().await;
        let repository = data_read
            .get::<FightUserRepoHolder>()
            .expect("Expected Fight user repository in TypeMap.")
            .clone();

        let user = repository
            .get_fight_user(&user_to_query.id.to_string(), &server_name)
            .await;
        if user.is_none() {
            let response = MessageBuilder::new()
                .mention(user_to_query)
                .push(" isn't in the game!")
                .build();
            log_msg_err(msg.channel_id.say(&ctx.http, response).await);
            return Ok(());
        }
        user
    }.unwrap();

    if (&queried_user).hitpoints > 0.0 {
        let response = MessageBuilder::new()
            .mention(user_to_query)
            .push(" hasn't been knocked out yet!")
            .build();

        log_msg_err(msg.channel_id.say(&ctx.http, response).await);
        return Ok(())
    }

    let revived_user = FightUser {
        hitpoints: DEFAULT_HITPOINTS,
        ..queried_user.clone()
    };

    {
        let data_read = ctx.data.read().await;
        let repository = data_read
            .get::<FightUserRepoHolder>()
            .expect("Expected Fight user repository in TypeMap.")
            .clone();

        repository
            .update_fight_user(
                &revived_user.user_id,
                server_name.as_str(),
                &revived_user,
            )
            .await?;
    }

    let revive_summary = MessageBuilder::new()
        .push("Revivied ")
        .mention(user_to_query)
        .push_line("!")
        .build();
    log_msg_err(msg.channel_id.say(&ctx.http, revive_summary).await);

    Ok(())
}

#[group]
#[prefix = "fight"]
#[description = "Commands to attack other users/bots"]
#[commands(spawn, status, attack, revive)]
#[only_in(guilds)]
struct Fight;
