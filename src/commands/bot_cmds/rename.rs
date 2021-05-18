use crate::models::mpc::MemeMessage;
use serenity::client::Context;
use serenity::model::id::UserId;

pub async fn rename_user(msg: &MemeMessage, ctx: &Context) -> Option<String> {
    if msg.arguments.contains_key("user_id")
        && msg.arguments.contains_key("new_name")
        && msg.arguments.contains_key("guild_id")
    {
        let new_name = msg.arguments["new_name"].clone();
        let user_id = msg.arguments["user_id"].parse::<UserId>().unwrap();
        let guild_id = msg.arguments["guild_id"].parse::<u64>().unwrap();

        let guild = ctx.http.get_guild(guild_id).await.unwrap();
        let result = guild
            .edit_member(&ctx.http, user_id, |m| m.nickname(&new_name[..32]))
            .await;

        return match result {
            Ok(_) => Some(format!("Updated {}'s name", msg.arguments["user_id"])),
            Err(e) => {
                println!("{:?}", e);
                None
            }
        };
    }
    None
}
