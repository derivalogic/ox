use rand::{rngs::StdRng, Rng, SeedableRng};

use crate::core::meta::{MarketData, MarketRequest};
use crate::time::date::Date;
use crate::utils::{errors::Result, num::Real};

fn sample_normal<T: Real>(rng: &mut StdRng) -> T {
    let u1: f64 = rng.gen();
    let u2: f64 = rng.gen();
    let z = (-2.0 * u1.ln()).sqrt() * (2.0 * std::f64::consts::PI * u2).cos();
    T::from(z)
}

/// Simple Black-Scholes Monte Carlo generator
#[derive(Clone, Copy)]
pub struct BlackScholesModel<T: Real> {
    pub s0: T,
    pub r: T,
    pub vol: T,
    pub maturity: T,
    pub reference: Date,
}

impl<T: Real> BlackScholesModel<T> {
    pub fn new(s0: T, r: T, vol: T, maturity: T, reference: Date) -> Self {
        Self { s0, r, vol, maturity, reference }
    }

    pub fn gen_scenarios(
        &self,
        reqs: &[MarketRequest],
        n: usize,
        seed: u64,
    ) -> Result<Vec<Vec<MarketData<T>>>> {
        let mut rng = StdRng::seed_from_u64(seed);
        let dt = self.maturity;
        let discount = (self.r * dt).exp();
        let mut scenarios = Vec::with_capacity(n);
        for _ in 0..n {
            let z: T = sample_normal(&mut rng);
            let drift = (self.r - self.vol * self.vol * T::from(0.5)) * dt;
            let diffusion = self.vol * dt.sqrt() * z;
            let x: T = drift + diffusion;
            let st = self.s0 * x.exp();
            let scenario: Vec<MarketData<T>> = reqs
                .iter()
                .map(|req| {
                    let fx = req.fx().map(|_| st);
                    MarketData::new(req.id(), self.reference, None, None, fx, discount)
                })
                .collect();
            scenarios.push(scenario);
        }
        Ok(scenarios)
    }
}

fn norm_pdf<T: Real>(x: T) -> T {
    let inv_sqrt_2pi = T::from(1.0 / (2.0_f64 * std::f64::consts::PI).sqrt());
    inv_sqrt_2pi * (-(x * x) * T::from(0.5)).exp()
}

fn norm_cdf<T: Real>(x: T) -> T {
    let one = T::from(1.0);
    let k = one / (one + T::from(0.2316419) * x.abs());
    let k_sum = k
        * (T::from(0.31938153)
            + k * (-T::from(0.356563782)
            + k * (T::from(1.781477937)
            + k * (-T::from(1.821255978) + k * T::from(1.330274429)))));
    let approx = one - norm_pdf(x) * k_sum;
    if x >= T::from(0.0) {
        approx
    } else {
        one - approx
    }
}

pub fn bs_price<T: Real>(s: T, k: T, r: T, vol: T, t: T) -> T {
    let sqt = t.sqrt();
    let d1 = ((s / k).ln() + (r + vol * vol * T::from(0.5)) * t) / (vol * sqt);
    let d2 = d1 - vol * sqt;
    s * norm_cdf(d1) - k * (-r * t).exp() * norm_cdf(d2)
}

pub fn bs_delta<T: Real>(s: T, k: T, r: T, vol: T, t: T) -> T {
    let sqt = t.sqrt();
    let d1 = ((s / k).ln() + (r + vol * vol * T::from(0.5)) * t) / (vol * sqt);
    norm_cdf(d1)
}

pub fn bs_gamma<T: Real>(s: T, k: T, r: T, vol: T, t: T) -> T {
    let sqt = t.sqrt();
    let d1 = ((s / k).ln() + (r + vol * vol * T::from(0.5)) * t) / (vol * sqt);
    norm_pdf(d1) / (s * vol * sqt)
}

pub fn bs_theta<T: Real>(s: T, k: T, r: T, vol: T, t: T) -> T {
    let sqt = t.sqrt();
    let d1 = ((s / k).ln() + (r + vol * vol * T::from(0.5)) * t) / (vol * sqt);
    let d2 = d1 - vol * sqt;
    -(s * norm_pdf(d1) * vol) / (T::from(2.0) * sqt) - r * k * (-r * t).exp() * norm_cdf(d2)
}

/// Return price and Greeks for convenience
pub fn bs_price_delta_gamma_theta<T: Real>(s: T, k: T, r: T, vol: T, t: T) -> (T, T, T, T) {
    (
        bs_price(s, k, r, vol, t),
        bs_delta(s, k, r, vol, t),
        bs_gamma(s, k, r, vol, t),
        bs_theta(s, k, r, vol, t),
    )
}
