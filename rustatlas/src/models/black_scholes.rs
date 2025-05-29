use rand::prelude::*;

use crate::{
    core::meta::{MarketData, MarketRequest, DiscountFactorRequest, ExchangeRateRequest, ForwardRateRequest},
    models::traits::{Model, MonteCarloModel},
    time::{date::Date, daycounter::DayCounter},
    utils::{errors::Result, num::Real},
};

/// Simple Black-Scholes model generating Monte Carlo scenarios.
#[derive(Clone, Copy)]
pub struct BlackScholesModel<T: Real> {
    pub s0: T,
    pub rate: T,
    pub vol: T,
    pub maturity: T,
    pub reference: Date,
}

impl<T: Real> BlackScholesModel<T> {
    pub fn new(s0: T, rate: T, vol: T, maturity: T, reference: Date) -> Self {
        Self { s0, rate, vol, maturity, reference }
    }

    fn sample_normal(rng: &mut ThreadRng) -> f64 {
        let u1: f64 = rng.gen();
        let u2: f64 = rng.gen();
        (-2.0 * u1.ln()).sqrt() * (2.0 * std::f64::consts::PI * u2).cos()
    }
}

impl<T: Real> Model<T> for BlackScholesModel<T> {
    fn reference_date(&self) -> Date { self.reference }

    fn gen_df_data(&self, df: DiscountFactorRequest) -> Result<T> {
        let dt = DayCounter::Actual365.year_fraction::<T>(self.reference, df.date());
        Ok((-self.rate * dt).exp())
    }

    fn gen_fx_data(&self, _fx: ExchangeRateRequest) -> Result<T> { Ok(self.s0) }

    fn gen_fwd_data(&self, _fwd: ForwardRateRequest) -> Result<T> { Ok(self.rate) }

    fn gen_numerarie(&self, _mr: &MarketRequest) -> Result<T> { Ok(T::from(1.0)) }
}

impl<T: Real> MonteCarloModel<T> for BlackScholesModel<T> {
    fn gen_scenarios(
        &self,
        market_request: &[MarketRequest],
        n: usize,
    ) -> Result<Vec<Vec<MarketData<T>>>> {
        let mut rng = thread_rng();
        let dt = self.maturity;
        let discount = (self.rate * dt).exp();
        let sqrt_dt = dt.sqrt();
        let half: T = T::from(0.5);
        let mut scenarios = Vec::with_capacity(n);
        for _ in 0..n {
            let z = Self::sample_normal(&mut rng);
            let zt = T::from(z);
            let st = self.s0 * ((self.rate - half * self.vol * self.vol) * dt + self.vol * sqrt_dt * zt).exp();
            let sc = market_request
                .iter()
                .map(|req| {
                    let fx = req.fx().map(|_| st);
                    MarketData::new(req.id(), self.reference, None, None, fx, discount)
                })
                .collect();
            scenarios.push(sc);
        }
        Ok(scenarios)
    }
}
