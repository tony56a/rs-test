use crate::models::fight_weapon::FightWeapon;
use dynomite::Item;

#[derive(Item, Debug, Clone)]
pub struct FightUser {
    #[dynomite(partition_key)]
    pub user_id: String,
    #[dynomite(sort_key)]
    pub server_name: String,
    pub hitpoints: f64,
    pub weapon: Option<FightWeapon>,
    pub knocked_out: Vec<String>,
    pub knocked_out_by: Vec<String>,
}

impl FightUser {
    pub fn new(user_id: &str, server_name: &str, hitpoints: f64) -> FightUser {
        FightUser {
            user_id: user_id.to_string(),
            server_name: server_name.to_string(),
            hitpoints: hitpoints,
            weapon: None,
            knocked_out: Vec::default(),
            knocked_out_by: Vec::default(),
        }
    }
}
