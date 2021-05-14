use crate::models::fight_weapon::FightWeapon;
use crate::models::holders::{FightWeaponRepoHolder, UserQuoteRepoHolder};
use crate::models::user_quote::UserQuote;
use crate::repos::fight_weapon::FightWeaponRepository;
use crate::repos::quotes::UserQuoteRepository;
use crate::utils::chat::{log_msg_err, ADMIN_CHECK};
use serenity::client::Context;
use serenity::framework::standard::{
    macros::{command, group},
    Args, CommandResult,
};
use serenity::model::channel::{Message, MessageReference};

#[command]
#[sub_commands(add_weapon)]
async fn weapons(_: &Context, _: &Message, _args: Args) -> CommandResult {
    Ok(())
}

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

#[command]
#[sub_commands(add_quote, delete_quote)]
async fn quote(_: &Context, _: &Message, _args: Args) -> CommandResult {
    Ok(())
}

#[command("add")]
async fn add_quote(ctx: &Context, msg: &Message) -> CommandResult {
    let quoted_messaage = match &msg.message_reference {
        None => return Ok(()),
        Some(message_ref) => {
            ctx.http
                .get_message(message_ref.channel_id.0, message_ref.message_id.unwrap().0)
                .await?
        }
    };
    let server_name = msg
        .guild_id
        .expect("Guild ID should be present")
        .0
        .to_string();

    {
        let data_read = ctx.data.read().await;
        let repository = data_read
            .get::<UserQuoteRepoHolder>()
            .expect("Expected Fight weapon repository in TypeMap.")
            .clone();
        let user_quote = UserQuote::new(&quoted_messaage, &server_name);

        let existing_quote = repository
            .get_quote_by_id(quoted_messaage.id, quoted_messaage.channel_id)
            .await;
        if existing_quote.is_some() {
            log_msg_err(msg.channel_id.say(&ctx.http, "quote already exists!").await);
            return Ok(());
        }

        repository.create_user_quote(&user_quote).await;
        log_msg_err(msg.reply(&ctx.http, "saved.").await);
    }
    Ok(())
}

#[command("delete")]
async fn delete_quote(ctx: &Context, msg: &Message) -> CommandResult {
    let quoted_messaage = match &msg.message_reference {
        None => return Ok(()),
        Some(message_ref) => {
            ctx.http
                .get_message(message_ref.channel_id.0, message_ref.message_id.unwrap().0)
                .await?
        }
    };
    let server_name = msg
        .guild_id
        .expect("Guild ID should be present")
        .0
        .to_string();

    {
        let data_read = ctx.data.read().await;
        let repository = data_read
            .get::<UserQuoteRepoHolder>()
            .expect("Expected Fight weapon repository in TypeMap.")
            .clone();

        repository.delete_quote(quoted_messaage.id, quoted_messaage.channel_id, &server_name)
            .await;

        log_msg_err(msg.reply(&ctx.http, "deleted.").await);
    }
    Ok(())
}

#[group]
#[prefix = "admin"]
#[checks(Admin)]
#[commands(weapons, quote)]
struct Admin;
