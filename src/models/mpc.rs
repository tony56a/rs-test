use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Deserialize, Serialize)]
pub struct MemeMessage {
    pub command: String,
    pub arguments: HashMap<String, String>
}

#[derive(Deserialize)]
pub struct MemeApiResponse {
    #[serde(alias = "url")]
    pub url: String,
    #[serde(alias = "postLink")]
    pub post_link: String,
}