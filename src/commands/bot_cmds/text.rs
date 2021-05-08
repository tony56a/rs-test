use crate::models::mpc::MemeMessage;
use serenity::client::Context;
use serenity::utils::MessageBuilder;

pub async fn echo_text(msg: &MemeMessage, ctx: &Context) -> Option<String> {
    if msg.arguments.contains_key("payload") {
        return Some(format!("Got a message: {}", msg.arguments["payload"]));
    }
    None
}
