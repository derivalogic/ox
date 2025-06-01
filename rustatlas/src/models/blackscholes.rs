use rand::{rngs::StdRng, Rng, SeedableRng};
use rand_distr::StandardNormal;

use crate::prelude::*;

/// Simple Black-Scholes Monte Carlo generator
#[derive(Clone)]
pub struct BlackScholesModel<'a> {
    pub simple: SimpleModel<'a>,
}

impl<'a> BlackScholesModel<'a> {
    pub fn new(simple: SimpleModel<'a>) -> Self {
        Self { simple }
    }
}

impl DeterministicModel for BlackScholesModel<'_> {
    fn reference_date(&self) -> Date {
        self.simple.reference_date()
    }

    fn gen_df_data(&self, df: DiscountFactorRequest) -> Result<NumericType> {
        self.simple.gen_df_data(df)
    }

    fn gen_fx_data(&self, fx: ExchangeRateRequest) -> Result<NumericType> {
        self.simple.gen_fx_data(fx)
    }

    fn gen_fwd_data(&self, fwd: ForwardRateRequest) -> Result<NumericType> {
        self.simple.gen_fwd_data(fwd)
    }

    fn gen_numerarie(&self, market_request: &MarketRequest) -> Result<NumericType> {
        self.simple.gen_numerarie(market_request)
    }
}

impl<'a> StochasticModel for BlackScholesModel<'a> {
    fn gen_scenario(&self, market_requests: &[MarketRequest]) -> Result<Scenario> {
        let store = self.simple.market_store();
        let ref_date = store.reference_date();
        let local_ccy = store.local_currency();
        let idx = store.index_store();

        /* --- parallel over all paths ------------------------------------ */
        let scenario = {
            /* each path gets its own reproducible RNG */
            let mut rng = StdRng::seed_from_u64(0xA55AA55Au64 as u64);

            /* collect the nodes of this scenario */
            let mut nodes = Vec::with_capacity(market_requests.len());

            for req in market_requests {
                /* ======================================================
                 *  FX NODE  (Monte-Carlo path)
                 * ====================================================*/
                if let Some(fx_req) = req.fx() {
                    /* maturity ....................................... */
                    let mat = fx_req.reference_date().unwrap_or(ref_date);
                    let t = Actual360::year_fraction(ref_date, mat);

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
                    let sigma =
                        store.get_exchange_rate_volatility(fx_req.first_currency(), second_ccy)?;
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

                    let fx_b_to_l = if local_ccy == second_ccy {
                        1.0 // case (1)
                    } else if local_ccy == fx_req.first_currency() {
                        1.0 / s_t // case (2)
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
        };

        Ok(scenario)
    }
}
