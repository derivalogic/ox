use crate::prelude::*;

pub type Scenario<T> = Vec<MarketData<T>>;

/// Trait for models capable of generating Monte Carlo scenarios.
pub trait StochasticModel<T: GenericNumber> {
    /// Generate stochastic scenarios for the given market requests.
    fn gen_scenario(&self, market_request: &[MarketRequest]) -> Result<Scenario<T>>;
}
