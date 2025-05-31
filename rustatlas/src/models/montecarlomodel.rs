use crate::{
    prelude::{MarketData, MarketRequest},
    utils::{errors::Result, num::Real},
};

pub type Scenario<T> = Vec<MarketData<T>>;
pub type Simulations<T> = Vec<Scenario<T>>;

/// Trait for models capable of generating Monte Carlo scenarios.
pub trait MonteCarloModel<T: Real> {
    /// Generate stochastic scenarios for the given market requests.
    fn gen_scenarios(
        &self,
        market_request: &[MarketRequest],
        n_simulations: usize,
    ) -> Result<Simulations<T>>;
}
