use crate::repos::fight_user::FightUserDDBRepository;
use crate::repos::fight_weapon::FightWeaponDDBRepository;
use serenity::prelude::TypeMapKey;
use std::sync::Arc;

// Holds the repositories
pub struct FightWeaponRepoHolder;
pub struct FightUserRepoHolder;

impl TypeMapKey for FightUserRepoHolder {
    type Value = Arc<FightUserDDBRepository>;
}

impl TypeMapKey for FightWeaponRepoHolder {
    type Value = Arc<FightWeaponDDBRepository>;
}
