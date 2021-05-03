use rand::distributions::{Distribution, Standard};
use rand::Rng;

pub(crate) enum AttackEffectiveness {
    NotEffective,
    Effective,
    ExtraEffective,
}

impl AttackEffectiveness {
    pub fn msg_value(&self) -> &str {
        match *self {
            AttackEffectiveness::NotEffective => "not very effective...",
            AttackEffectiveness::Effective => "sort of effective?",
            AttackEffectiveness::ExtraEffective => "super effective!",
        }
    }

    pub fn value_multiplier(&self) -> f64 {
        match *self {
            AttackEffectiveness::NotEffective => 0.5,
            AttackEffectiveness::Effective => 1.0,
            AttackEffectiveness::ExtraEffective => 2.0,
        }
    }
}

impl Distribution<AttackEffectiveness> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> AttackEffectiveness {
        // match rng.gen_range(0, 3) { // rand 0.5, 0.6, 0.7
        match rng.gen_range(0..=2) {
            // rand 0.8
            0 => AttackEffectiveness::NotEffective,
            1 => AttackEffectiveness::Effective,
            _ => AttackEffectiveness::ExtraEffective,
        }
    }
}
