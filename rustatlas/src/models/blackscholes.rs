use rand::{rngs::StdRng, Rng, SeedableRng};
use rand_distr::StandardNormal;
use rayon::iter::{IntoParallelIterator, ParallelIterator};

use crate::core::meta::{MarketData, MarketRequest};
use crate::prelude::{
    Actual360, DayCountProvider, DiscountFactorRequest, ExchangeRateRequest, ForwardRateRequest,
    HasReferenceDate, SimpleModel,
};
use crate::time::date::Date;
use crate::utils::{errors::Result, num::Real};

use super::deterministicmodel::DeterministicModel;
use super::montecarlomodel::{MonteCarloModel, Simulations};

/// Simple Black-Scholes Monte Carlo generator
#[derive(Clone)]
pub struct BlackScholesModel<'a, T: Real> {
    pub simple: SimpleModel<'a, T>,
}

impl<'a, T: Real> BlackScholesModel<'a, T> {
    pub fn new(simple: SimpleModel<'a, T>) -> Self {
        Self { simple }
    }
}

impl<T: Real> DeterministicModel<T> for BlackScholesModel<'_, T> {
    fn reference_date(&self) -> Date {
        self.simple.reference_date()
    }

    fn gen_df_data(&self, df: DiscountFactorRequest) -> Result<T> {
        self.simple.gen_df_data(df)
    }

    fn gen_fx_data(&self, fx: ExchangeRateRequest) -> Result<T> {
        self.simple.gen_fx_data(fx)
    }

    fn gen_fwd_data(&self, fwd: ForwardRateRequest) -> Result<T> {
        self.simple.gen_fwd_data(fwd)
    }

    fn gen_numerarie(&self, market_request: &MarketRequest) -> Result<T> {
        self.simple.gen_numerarie(market_request)
    }
}

impl<'a, T: Real> MonteCarloModel<T> for BlackScholesModel<'a, T> {
    fn gen_scenarios(
        &self,
        market_requests: &[MarketRequest],
        n_sims: usize,
    ) -> Result<Simulations<T>> {
        let store = self.simple.market_store();
        let ref_date = store.reference_date();
        let local_ccy = store.local_currency();
        let idx = store.index_store();

        /* --- parallel over all paths ------------------------------------ */
        let scenarios: Vec<Vec<MarketData<T>>> = (0..n_sims)
            .into_par_iter()
            .map(|path_id| {
                /* each path gets its own reproducible RNG */
                let mut rng = StdRng::seed_from_u64(0xA55AA55Au64 + path_id as u64);

                /* collect the nodes of this scenario */
                let mut nodes = Vec::with_capacity(market_requests.len());

                for req in market_requests {
                    /* ======================================================
                     *  FX NODE  (Monte-Carlo path)
                     * ====================================================*/
                    if let Some(fx_req) = req.fx() {
                        /* maturity ....................................... */
                        let mat = fx_req.reference_date().unwrap_or(ref_date);
                        let t = Actual360::year_fraction::<T>(ref_date, mat);

                        /* spot ........................................... */
                        let spot_req = ExchangeRateRequest::new(
                            fx_req.first_currency(),  // base  (a)
                            fx_req.second_currency(), // quote (b)
                            Some(ref_date),
                        );
                        let s0 = self.simple.gen_fx_data(spot_req).unwrap();

                        /* discount factors at maturity .................. */
                        let second_ccy = match fx_req.second_currency() {
                            Some(ccy) => ccy,
                            None => local_ccy, // if no second currency is given, use local currency
                        };
                        let base_curve = idx.get_currency_curve(fx_req.first_currency()).unwrap();
                        let quote_curve = idx.get_currency_curve(second_ccy).unwrap();
                        let local_curve = idx.get_currency_curve(local_ccy).unwrap();

                        let p_base = self
                            .simple
                            .gen_df_data(DiscountFactorRequest::new(base_curve, mat))
                            .unwrap();
                        let p_quote = self
                            .simple
                            .gen_df_data(DiscountFactorRequest::new(quote_curve, mat))
                            .unwrap();
                        let p_local = self
                            .simple
                            .gen_df_data(DiscountFactorRequest::new(local_curve, mat))
                            .unwrap();

                        /* continuous short-rates ........................ */
                        let r_base = -p_base.ln() / t;
                        let r_quote = -p_quote.ln() / t;
                        let r_local = -p_local.ln() / t;

                        /* one-step GBM .................................. */
                        let sigma = store
                            .get_volatility(fx_req.first_currency(), second_ccy)
                            .unwrap();
                        let z = rng.sample::<f64, _>(StandardNormal);

                        let drift = (r_quote - r_base) - sigma * sigma * 0.5;
                        let s_t = s0 * (drift * t + sigma * t.sqrt() * z).exp();

                        /* ---------------- numerarie (local-currency) -----------
                         *
                         *  For a payoff settled in the **quote** currency *b* :
                         *      N_T =  FX_{b→L}(T) / P_L(0,T)
                         *
                         *  FX_{b→L}(T) is handled case-by-case:
                         *    1. L == b  → FX = 1
                         *    2. L == a  → FX = 1 / S_{a,b}(T)
                         *    3. else     → use interest-parity forward
                         * ----------------------------------------------------*/

                        let fx_b_to_l: T = if local_ccy == second_ccy {
                            T::from(1.0) // case (1)
                        } else if local_ccy == fx_req.first_currency() {
                            T::from(1.0) / s_t // case (2)
                        } else {
                            /* case (3) – build forward B/L using interest parity */
                            let spot_b_l = self
                                .simple
                                .gen_fx_data(ExchangeRateRequest::new(
                                    second_ccy,
                                    Some(local_ccy),
                                    Some(ref_date),
                                ))
                                .unwrap();

                            let fwd = spot_b_l * ((r_quote - r_local) * t).exp();
                            fwd
                        };
                        let numerarie = fx_b_to_l / p_local;

                        // other values
                        let fwd = match req.fwd() {
                            Some(fwd_req) => Some(self.simple.gen_fwd_data(fwd_req).unwrap()),
                            None => None,
                        };
                        let df = match req.df() {
                            Some(df_req) => Some(self.simple.gen_df_data(df_req).unwrap()),
                            None => None,
                        };

                        nodes.push(MarketData::new(
                            req.id(),
                            mat,
                            /* df  */ df,
                            /* fwd */ fwd,
                            /* fx  */ Some(s_t),
                            /* num */ numerarie,
                        ));
                    }
                    /* ======================================================
                     *  ALL OTHER NODES – deterministic
                     * ====================================================*/
                    else {
                        nodes.push(self.simple.gen_node(req).unwrap());
                    }
                } // loop over requests
                nodes
            })
            .collect();

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
