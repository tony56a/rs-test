use serenity::model::channel::Message;
use serenity::Error;

pub fn log_msg_err(msg: Result<Message, Error>) {
    match msg {
        Ok(_) => {}
        Err(e) => {
            println!("Error when publishing message: {:?}", e)
        }
    };
}
