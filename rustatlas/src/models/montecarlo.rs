use rand::prelude::*;

use crate::core::marketstore::MarketStore;
use crate::core::meta::{MarketData, MarketRequest};
use crate::math::ad::Var;
use crate::models::{
    simplemodel::SimpleModel,
    traits::{Model, MonteCarloModel},
};
use crate::time::daycounter::DayCounter;
use crate::utils::errors::Result;
use crate::utils::num::Real;

/// Simple Monte Carlo model under risk free measure with random rates and fx.
pub struct RiskFreeMonteCarloModel<'a, T: Real> {
    simple: SimpleModel<'a, T>,
    rate_sigma: T,
    fx_sigma: T,
}

fn sample_normal<T: Real>(rng: &mut ThreadRng, sigma: T) -> T {
    let u1: f64 = rng.gen();
    let u2: f64 = rng.gen();
    let z = (-2.0 * u1.ln()).sqrt() * (2.0 * std::f64::consts::PI * u2).cos();
    sigma * z
}

impl<'a, T: Real> RiskFreeMonteCarloModel<'a, T> {
    pub fn new(market_store: &'a MarketStore<T>) -> Self {
        Self {
            simple: SimpleModel::new(market_store),
            rate_sigma: T::from(0.01),
            fx_sigma: T::from(0.05),
        }
    }
}

impl<'a, T: Real> Model<T> for RiskFreeMonteCarloModel<'a, T> {
    fn reference_date(&self) -> crate::time::date::Date {
        self.simple.reference_date()
    }
    fn gen_df_data(&self, df: crate::core::meta::DiscountFactorRequest) -> Result<T> {
        self.simple.gen_df_data(df)
    }
    fn gen_fx_data(&self, fx: crate::core::meta::ExchangeRateRequest) -> Result<T> {
        self.simple.gen_fx_data(fx)
    }
    fn gen_fwd_data(&self, fwd: crate::core::meta::ForwardRateRequest) -> Result<T> {
        self.simple.gen_fwd_data(fwd)
    }
    fn gen_numerarie(&self, mr: &MarketRequest) -> Result<T> {
        self.simple.gen_numerarie(mr)
    }
}

impl<'a, T: Real> MonteCarloModel<T> for RiskFreeMonteCarloModel<'a, T> {
    fn gen_scenarios(
        &self,
        market_request: &[MarketRequest],
        n: usize,
    ) -> Result<Vec<Vec<MarketData<T>>>> {
        let mut rng = thread_rng();
        let mut scenarios = Vec::new();
        for _ in 0..n {
            let mut scenario = Vec::new();
            for req in market_request {
                let mut data = self.simple.gen_node(req)?;
                if let (Ok(df), Some(df_req)) = (data.df(), req.df()) {
                    let dt = DayCounter::Actual365
                        .year_fraction::<T>(self.reference_date(), df_req.date());
                    let shock = sample_normal(&mut rng, self.rate_sigma * dt.sqrt());
                    data = MarketData::new(
                        data.id(),
                        data.reference_date(),
                        Some(df * (shock + 1.0)),
                        data.fwd().ok(),
                        data.fx().ok(),
                        data.numerarie(),
                    );
                }
                if let (Ok(fx), Some(fx_req)) = (data.fx(), req.fx()) {
                    let dt = fx_req
                        .reference_date()
                        .map(|d| DayCounter::Actual365.year_fraction(self.reference_date(), d))
                        .unwrap_or(0.0);
                    let shock = sample_normal(&mut rng, self.fx_sigma * dt.sqrt());
                    data = MarketData::new(
                        data.id(),
                        data.reference_date(),
                        data.df().ok(),
                        data.fwd().ok(),
                        Some(fx * (shock + 1.0)),
                        data.numerarie(),
                    );
                }
                let data = MarketData::new(
                    data.id(),
                    data.reference_date(),
                    data.df().ok(),
                    data.fwd().ok(),
                    data.fx().ok(),
                    data.numerarie(),
                );
                scenario.push(data);
            }
            scenarios.push(scenario);
        }
        Ok(scenarios)
    }
}
