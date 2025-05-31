use rand::prelude::*;

use crate::core::marketstore::MarketStore;
use crate::core::meta::{MarketData, MarketRequest};
use crate::prelude::{DiscountFactorRequest, ExchangeRateRequest};
use crate::time::daycounter::DayCounter;
use crate::utils::{errors::Result, num::Real};

/// Monte-Carlo model combining
/// • Hull–White one-factor short-rate dynamics  
/// • Stochastic-vol FX with CIR/Heston variance process.
pub struct StochasticVolAndRatesModel<'a, T: Real> {
    /* Hull–White parameters */
    market_store: &'a MarketStore<T>,
    mean_rev_a: T,     // a
    rate_vol_sigma: T, // σ_r

    /* Stochastic-vol parameters for FX */
    kappa: T,  // mean reversion speed of variance
    theta: T,  // long-run variance
    volvol: T, // σ_v
    rho: T,    // Corr(dW^S, dW^v)
    v0: T,     // initial variance

    /* Misc. */
    rng: ThreadRng,
}

impl<'a, T: Real> StochasticVolAndRatesModel<'a, T> {
    pub fn new(market_store: &'a MarketStore<T>) -> Self {
        Self {
            market_store,
            /* hand-picked, obviously you will later calibrate or make configurable */
            mean_rev_a: T::from(0.03),     // 3 % mean-reversion
            rate_vol_sigma: T::from(0.01), // 1 % vol of r

            kappa: T::from(1.50),
            theta: T::from(0.04),
            volvol: T::from(0.30),
            rho: T::from(-0.40),
            v0: T::from(0.04),

            rng: thread_rng(),
        }
    }

    /* ========== small helpers ================================================= */

    /// Draw (Z₁,Z₂) with Corr(Z₁,Z₂)=ρ
    fn correlated_normals(&mut self) -> (T, T) {
        // independent N(0,1)
        let z1: f64 = self.rng.sample(StandardNormal);
        let z2: f64 = self.rng.sample(StandardNormal);
        // correlate
        let rho = self.rho.into();
        let z2_corr = rho * z1 + (1.0_f64 - rho * rho).sqrt() * z2;
        (T::from(z1), T::from(z2_corr))
    }

    /// deterministic θ(t) term in HW.  Here we approximate with flat forward = r(0)
    fn theta_hw(&self, _t: T, r0: T) -> T {
        r0
    }
}

/* ========== Trait – Monte-Carlo ============================================ */

impl<'a, T: Real> MonteCarloModel<T> for StochasticVolAndRatesModel<'a, T> {
    fn gen_scenarios(
        &self,
        market_requests: &[MarketRequest],
        n_simulations: usize,
    ) -> Result<Vec<Vec<MarketData<T>>>> {
        /* --------- sort requests chronologically once ------------------------ */
        let mut idx_and_t = Vec::with_capacity(market_requests.len());
        for (idx, req) in market_requests.iter().enumerate() {
            // earliest relevant date for the node
            let d = req
                .df()
                .map(|dfr| dfr.date())
                .or_else(|| req.fx().and_then(|fx| fx.reference_date()))
                .unwrap_or_else(|| self.reference_date());
            let t: T = DayCounter::Actual365.year_fraction::<T>(self.reference_date(), d);
            idx_and_t.push((idx, t));
        }
        idx_and_t.sort_by(|(_, t1), (_, t2)| t1.partial_cmp(t2).unwrap());

        /* --------- pre-get today’s deterministic levels ---------------------- */
        // risk-free short rate r(0) -> take instant fwd from the DF curve
        // simplest: minus ln[ P(t=1d) ] / 1d
        let p_1d: T = self.simple.gen_df_data(DiscountFactorRequest::new(
            self.reference_date().add_days(1),
        ))?;
        let r0 = -p_1d.ln(); // 1-day approximation – good enough for demo

        let s0 = self
            .simple
            .gen_fx_data(ExchangeRateRequest::new(None) /* spot */)?;
        /* -------------------------------------------------------------------- */

        let a = self.mean_rev_a;
        let sig_r = self.rate_vol_sigma;
        let kappa = self.kappa;
        let theta = self.theta;
        let sig_v = self.volvol;

        let mut out = Vec::with_capacity(n_paths);

        /* ================= simulate each scenario =========================== */
        for _ in 0..n_paths {
            /* state variables at t = 0 */
            let mut r = r0;
            let mut v = self.v0;
            let mut s = s0;
            let mut df = T::one(); // P(0,0) = 1
            let mut t_prev = T::zero();

            // buffer for this path – positions preserved
            let mut path_data = vec![MarketData::dummy(); market_requests.len()];

            for &(original_idx, t) in &idx_and_t {
                let dt = t - t_prev;
                let dt_f64 = dt.to_f64();

                /* ===== draw correlated normals ===== */
                let (z_r, z_v) = self.correlated_normals();
                // dW^S needs to be correlated with v, so reuse z_v for spot
                let z_s = z_v; // ρ_SV = +1 (easily generalised)

                /* ===== Hull–White exact step ===== */
                // mean & variance of r over (t_prev,t]
                let m = r * (-a * dt).exp() + self.theta_hw(t, r0) * (T::one() - (-a * dt).exp());
                let var = (sig_r * sig_r) * (T::one() - (-T::from(2.0) * a * dt).exp())
                    / (T::from(2.0) * a);
                r = m + var.sqrt() * z_r;

                /* discount factor increment */
                df *= (-r * dt).exp();

                /* ===== variance (CIR) step – Euler-Milstein for positivity ===== */
                let v_sqrt = v.sqrt().max(T::zero());
                let v_new = (v + kappa * (theta - v) * dt + sig_v * v_sqrt * dt.sqrt() * z_v)
                    .max(T::zero());
                v = v_new;

                /* ===== FX spot step ===== */
                // under domestic risk-neutral measure drift ≈ 0 for demo
                s *= (-(T::from(0.5)) * v * dt + v.sqrt() * dt.sqrt() * z_s).exp();

                t_prev = t;

                /* ===== generate MarketData for this node =================== */
                let req = &market_requests[original_idx];
                let md = if let Some(df_req) = req.df() {
                    // node asked for discount factor
                    MarketData::new(req.id(), df_req.date(), Some(df), None, None, None)
                } else if req.fx().is_some() {
                    MarketData::new(req.id(), self.reference_date(), None, None, Some(s), None)
                } else {
                    // fall back to deterministic
                    self.simple.gen_node(req)?
                };
                path_data[original_idx] = md;
            } /* nodes loop */

            out.push(path_data);
        } /* scenarios loop */

        Ok(out)
    }
}
