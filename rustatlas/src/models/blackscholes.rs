//! Black-Scholes Monte-Carlo model that **always prices in the store’s
//! `local_currency()`**.  
//!
//! * Every foreign currency *f* is simulated directly against the local
//!   currency *L* with drift **r<sub>f</sub> - r<sub>L</sub>**.  
//! * Any cross–pair *a/b* is obtained on the fly by triangulation  
//!   **S<sub>a,b</sub>(T) = FX<sub>a→L</sub>(T) / FX<sub>b→L</sub>(T)**.  
//! * The numeraire is the deterministic money-market account  
//!   **N<sub>T</sub> = 1 / P<sub>L</sub>(0,T)** for every node.

use rand::{rngs::StdRng, Rng, SeedableRng};
use rand_distr::StandardNormal;

use crate::prelude::*;

/// Simple Black-Scholes Monte-Carlo generator
#[derive(Clone)]
pub struct BlackScholesModel<'a> {
    pub simple: SimpleModel<'a>,
    pub seed: Option<u64>,
    pub time_handle: NumericType,
}

impl<'a> BlackScholesModel<'a> {
    pub fn new(simple: SimpleModel<'a>) -> Self {
        Self {
            simple,
            seed: None,
            time_handle: NumericType::zero(),
        }
    }

    pub fn with_seed(mut self, seed: u64) -> Self {
        self.seed = Some(seed);
        self
    }

    pub fn get_time_handle(&self) -> NumericType {
        self.time_handle
    }

    /* ------------------------------------------------------------------ */
    /* helper: simulate FX_{foreign→local}(T) and store in a cache         */
    /* ------------------------------------------------------------------ */
    fn simulate_fx_to_local(
        &self,
        foreign: Currency,
        mat: Date,
        t: NumericType,
        store: &MarketStore,
        rng: &mut StdRng,
    ) -> Result<NumericType> {
        /* spot FX_{f→L}(0) via triangulation supplied by the store */
        let spot = store
            .exchange_rate_store()
            .get_exchange_rate(foreign, store.local_currency())?;
        if mat == store.reference_date() {
            return Ok(spot.into());
        }
        /* discount factors */
        let idx = store.index_store();
        let f_curve = idx.get_currency_curve(foreign)?;
        let l_curve = idx.get_currency_curve(store.local_currency())?;
        let r_f = idx
            .get_index(f_curve)?
            .try_read()
            .unwrap()
            .term_structure()
            .unwrap()
            .forward_rate(
                store.reference_date(),
                mat,
                Compounding::Continuous,
                Frequency::Annual,
            )?;

        let r_l = idx
            .get_index(l_curve)?
            .try_read()
            .unwrap()
            .term_structure()
            .unwrap()
            .forward_rate(
                store.reference_date(),
                mat,
                Compounding::Continuous,
                Frequency::Annual,
            )?;

        /* volatility for pair f/L */
        let sigma = store.get_exchange_rate_volatility(foreign, store.local_currency())?;
        let z = rng.sample::<f64, _>(StandardNormal);

        let drift = (r_f - r_l) - sigma * sigma * 0.5;
        let s_t: NumericType = (spot * (drift * t + sigma * t.sqrt() * z).exp()).into();

        Ok(s_t)
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
    fn gen_numerarie(&self, m: NumerarieRequest) -> Result<NumericType> {
        let store = self.simple.market_store();
        let local_ccy = store.local_currency();
        let idx = store.index_store();
        let mat = m.reference_date();
        let p_local = self.simple.gen_df_data(DiscountFactorRequest::new(
            idx.get_currency_curve(local_ccy)?,
            mat,
        ))?;
        let result = (NumericType::one() / p_local).into();
        Ok(result)
    }
}

impl<'a> StochasticModel for BlackScholesModel<'a> {
    fn gen_scenario(&self, market_requests: &[MarketRequest]) -> Result<Scenario> {
        let store = self.simple.market_store();
        let ref_date = store.reference_date();
        let local_ccy = store.local_currency();
        let idx = store.index_store();

        /* RNG for this path */
        let mut rng = match self.seed {
            Some(seed) => StdRng::seed_from_u64(seed),
            None => StdRng::from_entropy(),
        };

        /* collect nodes */
        let mut nodes = Vec::with_capacity(market_requests.len());

        for req in market_requests {
            /* ============================================================
             *  FX node (stochastic)
             * ========================================================== */
            if let Some(fx_req) = req.fx() {
                let mat = fx_req.reference_date().unwrap_or(ref_date);
                let t = (Actual360::year_fraction(ref_date, mat) - self.time_handle).into();

                /* leg currencies */
                let ccy_a = fx_req.first_currency(); // base
                let ccy_b = fx_req.second_currency().unwrap_or(local_ccy); // quote (fallback L)

                /* simulate FX_{a→L}(T) and FX_{b→L}(T) once per currency */
                let fx_a_l = self.simulate_fx_to_local(ccy_a, mat, t, store, &mut rng)?;
                let fx_b_l = self.simulate_fx_to_local(ccy_b, mat, t, store, &mut rng)?;

                /* cross-pair value at T */
                let s_t = fx_a_l / fx_b_l;

                /* deterministic money-market numeraire */
                let p_local = self.simple.gen_df_data(DiscountFactorRequest::new(
                    idx.get_currency_curve(local_ccy)?,
                    mat,
                ))?;
                let numerarie: NumericType = (NumericType::one() / p_local).into();

                /* optional deterministic data */
                let fwd = req.fwd().map(|f| self.simple.gen_fwd_data(f).unwrap());
                let df = req.df().map(|d| self.simple.gen_df_data(d).unwrap());

                nodes.push(MarketData::new(
                    req.id(),
                    mat,
                    df,  // discount factor (optional)
                    fwd, // forward rate    (optional)
                    Some(s_t.into()),
                    numerarie,
                ));
            }
            /* ============================================================
             *  all other requests – deterministic
             * ========================================================== */
            else {
                nodes.push(self.gen_node(req)?);
            }
        }

        Ok(nodes)
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, RwLock};

    use super::*;
    fn create_market_store(
        local_ccy: Currency,
        s0_clpusd: NumericType,
        s0_useeur: NumericType,
        r_usd: NumericType,
        r_clp: NumericType,
        r_eur: NumericType,
    ) -> MarketStore {
        let ref_date = Date::new(2024, 1, 1);
        let mut store = MarketStore::new(ref_date, local_ccy);
        store
            .mut_exchange_rate_store()
            .add_exchange_rate(Currency::CLP, Currency::USD, s0_clpusd);

        store
            .mut_exchange_rate_store()
            .add_exchange_rate(Currency::USD, Currency::EUR, s0_useeur);
        let usd_curve = Arc::new(FlatForwardTermStructure::new(
            ref_date,
            r_usd,
            RateDefinition::default(),
        ));
        let index = Arc::new(RwLock::new(
            OvernightIndex::new(ref_date).with_term_structure(usd_curve),
        ));
        let _ = store.mut_index_store().add_index(0, index);
        store.mut_index_store().add_currency_curve(Currency::USD, 0);

        let clp_curve = Arc::new(FlatForwardTermStructure::new(
            ref_date,
            r_clp,
            RateDefinition::default(),
        ));
        let index_clp = Arc::new(RwLock::new(
            OvernightIndex::new(ref_date).with_term_structure(clp_curve),
        ));
        let _ = store.mut_index_store().add_index(1, index_clp);
        store.mut_index_store().add_currency_curve(Currency::CLP, 1);

        let eur_curve = Arc::new(FlatForwardTermStructure::new(
            ref_date,
            r_eur,
            RateDefinition::default(),
        ));
        let index_eur = Arc::new(RwLock::new(
            OvernightIndex::new(ref_date).with_term_structure(eur_curve),
        ));
        let _ = store.mut_index_store().add_index(2, index_eur);
        store.mut_index_store().add_currency_curve(Currency::EUR, 2);

        // add volatility
        store.mut_exchange_rate_store().add_volatility(
            Currency::CLP,
            Currency::USD,
            NumericType::new(0.2),
        );
        // add volatility
        store.mut_exchange_rate_store().add_volatility(
            Currency::EUR,
            Currency::USD,
            NumericType::new(0.2),
        );

        store.mut_exchange_rate_store().add_volatility(
            Currency::EUR,
            Currency::CLP,
            NumericType::new(0.2),
        );
        store
    }

    #[test]
    fn test_black_scholes_model() -> Result<()> {
        let store = create_market_store(
            Currency::USD,
            NumericType::new(1.0),
            NumericType::new(1.0),
            NumericType::new(0.05),
            NumericType::new(0.03),
            NumericType::new(0.02),
        );
        let model = BlackScholesModel::new(SimpleModel::new(&store));
        let date = Date::new(2024, 6, 1);
        let market_requests = vec![MarketRequest::new(
            0,
            Some(DiscountFactorRequest::new(0, date)),
            None,
            Some(ExchangeRateRequest::new(
                Currency::CLP,
                Some(Currency::USD),
                Some(date),
            )),
            None,
        )];
        let scenario = model.gen_scenario(&market_requests)?;
        assert!(!scenario.is_empty());
        Ok(())
    }

    #[test]
    fn test_parallel_model() -> Result<()> {
        let store = create_market_store(
            Currency::USD,
            NumericType::new(1.0),
            NumericType::new(1.0),
            NumericType::new(0.05),
            NumericType::new(0.03),
            NumericType::new(0.02),
        );
        let model = BlackScholesModel::new(SimpleModel::new(&store));
        let date = Date::new(2024, 6, 1);
        let market_requests = vec![MarketRequest::new(
            0,
            Some(DiscountFactorRequest::new(0, date)),
            None,
            Some(ExchangeRateRequest::new(
                Currency::CLP,
                Some(Currency::USD),
                Some(date),
            )),
            None,
        )];
        let scenario = model.gen_scenario(&market_requests)?;
        assert!(!scenario.is_empty());
        Ok(())
    }
}
