use rand::prelude::*;

use crate::core::marketstore::MarketStore;
use crate::core::meta::{MarketData, MarketRequest};
use crate::math::ad::Var;
use crate::models::{simplemodel::SimpleModel, traits::{Model, MonteCarloModel}};
use crate::time::daycounter::DayCounter;
use crate::utils::errors::Result;

/// Simple Monte Carlo model under risk free measure with random rates and fx.
pub struct RiskFreeMonteCarloModel<'a> {
    simple: SimpleModel<'a>,
    rate_sigma: f64,
    fx_sigma: f64,
}

fn sample_normal(rng: &mut ThreadRng, sigma: f64) -> f64 {
    let u1: f64 = rng.gen();
    let u2: f64 = rng.gen();
    let z = (-2.0 * u1.ln()).sqrt() * (2.0 * std::f64::consts::PI * u2).cos();
    z * sigma
}

impl<'a> RiskFreeMonteCarloModel<'a> {
    pub fn new(market_store: &'a MarketStore) -> Self {
        Self { simple: SimpleModel::new(market_store), rate_sigma: 0.01, fx_sigma: 0.05 }
    }
}

impl<'a> Model for RiskFreeMonteCarloModel<'a> {
    type Num = Var;
    fn reference_date(&self) -> crate::time::date::Date { self.simple.reference_date() }
    fn gen_df_data(&self, df: crate::core::meta::DiscountFactorRequest) -> Result<Self::Num> { self.simple.gen_df_data(df).map(Var::from) }
    fn gen_fx_data(&self, fx: crate::core::meta::ExchangeRateRequest) -> Result<Self::Num> { self.simple.gen_fx_data(fx).map(Var::from) }
    fn gen_fwd_data(&self, fwd: crate::core::meta::ForwardRateRequest) -> Result<Self::Num> { self.simple.gen_fwd_data(fwd).map(Var::from) }
    fn gen_numerarie(&self, mr: &MarketRequest) -> Result<Self::Num> { self.simple.gen_numerarie(mr).map(Var::from) }
}

impl<'a> MonteCarloModel for RiskFreeMonteCarloModel<'a> {
    fn gen_scenarios(
        &self,
        market_request: &[MarketRequest],
        n: usize,
    ) -> Result<Vec<Vec<MarketData<Self::Num>>>> {
        let mut rng = thread_rng();
        let mut scenarios = Vec::new();
        for _ in 0..n {
            let mut scenario = Vec::new();
            for req in market_request {
                let mut data = self.simple.gen_node(req)?;
                if let (Ok(df), Some(df_req)) = (data.df(), req.df()) {
                    let dt = DayCounter::Actual365.year_fraction(self.reference_date(), df_req.date());
                    let shock = sample_normal(&mut rng, self.rate_sigma * dt.sqrt());
                    data = MarketData::new(
                        data.id(),
                        data.reference_date(),
                        Some(df * (1.0 + shock)),
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
                        Some(fx * (1.0 + shock)),
                        data.numerarie(),
                    );
                }
                let data = MarketData::new(
                    data.id(),
                    data.reference_date(),
                    data.df().ok().map(Var::from),
                    data.fwd().ok().map(Var::from),
                    data.fx().ok().map(Var::from),
                    Var::from(data.numerarie()),
                );
                scenario.push(data);
            }
            scenarios.push(scenario);
        }
        Ok(scenarios)
    }
}

impl<'a> RiskFreeMonteCarloModel<'a> {
    /// Helper to convert scenarios generated with automatic differentiation values into plain f64 scenarios.
    pub fn scenarios_to_f64(
        scenarios: Vec<Vec<MarketData<Var>>>,
    ) -> Vec<Vec<MarketData<f64>>> {
        scenarios
            .into_iter()
            .map(|sc| {
                sc.into_iter()
                    .map(|d| {
                        MarketData::new(
                            d.id(),
                            d.reference_date(),
                            d.df().ok().map(|v| v.value()),
                            d.fwd().ok().map(|v| v.value()),
                            d.fx().ok().map(|v| v.value()),
                            d.numerarie().value(),
                        )
                    })
                    .collect()
            })
            .collect()
    }
}
