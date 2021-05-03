use crate::repos::fight_user::FightUserDDBRepository;
use serenity::prelude::TypeMapKey;
use std::sync::Arc;

// Holds the repository
pub struct FightUserRepoHolder;

impl TypeMapKey for FightUserRepoHolder {
    type Value = Arc<FightUserDDBRepository>;
}
