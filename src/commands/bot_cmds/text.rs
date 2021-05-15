use crate::models::mpc::MemeMessage;
use serenity::client::Context;

pub async fn echo_text(msg: &MemeMessage, _ctx: &Context) -> Option<String> {
    if msg.arguments.contains_key("payload") {
        return Some(format!("Got a message: {}", msg.arguments["payload"]));
    }
    None
}
