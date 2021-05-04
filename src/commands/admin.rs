use crate::models::bot_config::Owners;
use crate::models::db::FightWeaponRepoHolder;
use crate::models::fight_weapon::FightWeapon;
use crate::repos::fight_weapon::FightWeaponRepository;
use crate::utils::chat::log_msg_err;
use serenity::client::Context;
use serenity::framework::standard::{
    macros::{check, command, group},
    Args, CommandOptions, CommandResult, Reason,
};
use serenity::model::channel::Message;
use serenity::model::Permissions;

#[check]
#[name = "Admin"]
async fn admin_check(
    ctx: &Context,
    msg: &Message,
    _: &mut Args,
    _: &CommandOptions,
) -> Result<(), Reason> {
    let owners = {
        let data_read = ctx.data.read().await;
        let owners_lock = data_read
            .get::<Owners>()
            .expect("Expected owners in TypeMap.")
            .clone();
        let owners = owners_lock.read().await;
        owners.clone()
    };

    if let Some(member) = &msg.member {
        for role in &member.roles {
            if role
                .to_role_cached(&ctx.cache)
                .await
                .map_or(false, |r| r.has_permission(Permissions::ADMINISTRATOR))
            {
                return Ok(())
            }
        }
    }

    if owners.contains(&msg.author.id) {
        return Ok(());
    }

    Err(Reason::Log("Lacked owner permission".to_string()))
}

// A command can have sub-commands, just like in command lines tools.
// Imagine `cargo help` and `cargo help run`.
#[command]
#[sub_commands(add_weapon)]
async fn weapons(_: &Context, _: &Message, _args: Args) -> CommandResult {
    Ok(())
}

// This will only be called if preceded by the `upper`-command.
#[command("add")]
async fn add_weapon(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    if args.is_empty() {
        log_msg_err(
            msg.channel_id
                .say(
                    &ctx.http,
                    String::from("Use me with weapon_name attack_value"),
                )
                .await,
        );
        return Ok(());
    }

    let weapon_name = args.single_quoted::<String>()?;
    let attack_val = args.single::<f64>()?;
    let server_name = msg
        .guild_id
        .expect("Guild ID should be present")
        .0
        .to_string();

    let new_fight_weapon = FightWeapon {
        name: weapon_name.clone(),
        attack_val: attack_val,
        server_name: server_name.clone(),
    };

    {
        let data_read = ctx.data.read().await;
        let repository = data_read
            .get::<FightWeaponRepoHolder>()
            .expect("Expected Fight user repository in TypeMap.")
            .clone();

        let existing_weapon = repository
            .get_fight_weapon(&weapon_name, &server_name)
            .await;
        if existing_weapon.is_some() {
            log_msg_err(
                msg.channel_id
                    .say(&ctx.http, "weapon already exists!")
                    .await,
            );
            return Ok(());
        }

        repository.create_fight_weapon(&new_fight_weapon).await;
        log_msg_err(msg.channel_id.say(&ctx.http, "weapon created!").await);
    }
    Ok(())
}

#[group]
#[prefix = "admin"]
#[checks(Admin)]
#[commands(weapons)]
struct Admin;
