use dynomite::Item;
use serenity::model::channel::Message;
use std::cmp::Ordering;
use uuid::Uuid;

#[derive(Item, Debug, Clone, Default, Eq)]
pub struct UserQuote {
    #[dynomite(partition_key)]
    server_name: String,
    #[dynomite(sort_key)]
    sort_id: Uuid,

    pub author_id: String,
    pub channel_id: String,
    pub message_id: String,
    pub message_content: String,
}

impl UserQuote {
    pub fn new(msg: &Message, server_name: &str) -> UserQuote {
        UserQuote {
            server_name: server_name.to_string(),
            sort_id: Uuid::new_v4(),
            author_id: msg.author.id.to_string(),
            message_id: msg.id.to_string(),
            channel_id: msg.channel_id.to_string(),
            message_content: msg.content.clone(),
        }
    }

    pub fn key(&self) -> String {
        let mut key = String::from(&self.channel_id);
        key.push_str("+");
        key.push_str(&self.message_id);
        key
    }

    pub fn sort_id_key(&self) -> String {
        self.sort_id.to_hyphenated().to_string()
    }
}

impl Ord for UserQuote {
    fn cmp(&self, other: &Self) -> Ordering {
        self.key().cmp(&other.key())
    }
}

impl PartialOrd for UserQuote {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for UserQuote {
    fn eq(&self, other: &Self) -> bool {
        self.key() == other.key()
    }
}
