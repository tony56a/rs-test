use dynomite::Attributes;

#[derive(Attributes, Debug, Clone)]
pub struct FightWeapon {
    pub name: String,
    pub attack_val: f64,
}
