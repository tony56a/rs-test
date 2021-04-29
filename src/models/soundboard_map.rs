use serenity::prelude::TypeMapKey;
use std::collections::HashMap;
use std::sync::Arc;

use std::path::PathBuf;
use tokio::sync::RwLock;

// mapping of file names to the File path
pub struct SoundboardMap;

impl TypeMapKey for SoundboardMap {
    type Value = Arc<RwLock<HashMap<String, PathBuf>>>;
}
