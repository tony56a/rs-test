use crate::models::db::FightWeaponRepoHolder;
use crate::models::fight_weapon::FightWeapon;
use crate::repos::fight_weapon::FightWeaponRepository;
use crate::utils::chat::{log_msg_err, ADMIN_CHECK};
use serenity::client::Context;
use serenity::framework::standard::{
    macros::{command, group},
    Args, CommandResult,
};
use serenity::model::channel::Message;

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
            .expect("Expected Fight weapon repository in TypeMap.")
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
