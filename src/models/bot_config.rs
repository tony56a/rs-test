use serenity::prelude::TypeMapKey;
use std::collections::HashMap;
use std::sync::Arc;

use tokio::sync::RwLock;

// Global state config for bots, holds a kv mapping of things like:
// API tokens
// file mappings
// feature flags
pub struct BotConfig;

impl TypeMapKey for BotConfig {
    type Value = Arc<RwLock<HashMap<String, String>>>;
}
