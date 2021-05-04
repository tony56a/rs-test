use serenity::prelude::TypeMapKey;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use serenity::model::id::UserId;
use tokio::sync::RwLock;

// Global state config for bots, holds a kv mapping of things like:
// API tokens
// file mappings
// feature flags
pub struct BotConfig;

impl TypeMapKey for BotConfig {
    type Value = Arc<RwLock<HashMap<String, String>>>;
}

pub struct Owners;

impl TypeMapKey for Owners {
    type Value = Arc<RwLock<HashSet<UserId>>>;
}
