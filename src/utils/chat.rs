use crate::models::bot_config::Owners;
use serenity::client::Context;
use serenity::framework::standard::{
    macros::{check, command, group},
    Args, CommandOptions, Reason,
};
use serenity::model::channel::Message;
use serenity::model::Permissions;
use serenity::Error;

pub fn log_msg_err(msg: Result<Message, Error>) {
    match msg {
        Ok(_) => {}
        Err(e) => {
            println!("Error when publishing message: {:?}", e)
        }
    };
}

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
                return Ok(());
            }
        }
    }

    if owners.contains(&msg.author.id) {
        return Ok(());
    }

    Err(Reason::Log("Lacked owner permission".to_string()))
}
