use dynomite::Item;

#[derive(Item, Debug, Clone, Default)]
pub struct FightWeapon {
    #[dynomite(partition_key)]
    pub name: String,
    #[dynomite(sort_key)]
    pub server_name: String,
    pub attack_val: f64,
}
