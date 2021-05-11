use crate::repos::fight_user::FightUserDDBRepository;
use crate::repos::fight_weapon::FightWeaponDDBRepository;
use serenity::prelude::TypeMapKey;
use std::sync::Arc;
use tokio::sync::RwLock;
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use serenity::model::id::UserId;
use crate::repos::quotes::UserQuoteDDBRepository;

// Holds the repositories
pub struct FightWeaponRepoHolder;
pub struct FightUserRepoHolder;
pub struct UserQuoteRepoHolder;

impl TypeMapKey for FightUserRepoHolder {
    type Value = Arc<FightUserDDBRepository>;
}

impl TypeMapKey for FightWeaponRepoHolder {
    type Value = Arc<FightWeaponDDBRepository>;
}

impl TypeMapKey for UserQuoteRepoHolder {
    type Value = Arc<UserQuoteDDBRepository>;
}

// mapping of file names to the File path
pub struct SoundboardMap;

impl TypeMapKey for SoundboardMap {
    type Value = Arc<RwLock<HashMap<String, PathBuf>>>;
}

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
