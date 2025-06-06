use rustatlas::prelude::*;

pub trait RandomNumberGenerator {
    type Rng;
    fn set_rng(&self, rng: Self::Rng);
    fn set_seed(&self, seed: u64);
    fn gen_rand(&self) -> f64;
    fn time_step(&self, date: Date) -> NumericType;
}
