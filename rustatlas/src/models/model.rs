use crate::prelude::*;

pub enum MonteCarloModel<T: Real> {
    BlackScholes(BlackScholesModel<T>),
}

impl<T: Real> MonteCarloModel<T> {
    pub fn init_model();
}
