use crate::prelude::*;

pub type Scenario = Vec<MarketData>;

/// Trait for models capable of generating Monte Carlo scenarios.
pub trait StochasticModel {
    /// Generate stochastic scenarios for the given market requests.
    fn gen_scenario(&self, market_request: &[MarketRequest]) -> Result<Scenario>;
}

pub trait ParallelSimulation {
    /// Generate stochastic scenarios in parallel for the given market requests.
    fn gen_parallel_scenario(
        &self,
        market_request: &[MarketRequest],
        num_threads: usize,
    ) -> Result<Vec<Scenario>>;
}
