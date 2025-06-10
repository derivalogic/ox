use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

use crate::prelude::*;
use rand::Rng;
use rand_distr::StandardNormal;
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use rustatlas::prelude::*;

pub trait FxModel {
    fn simulate_fx(&self, request: &ExchangeRateRequest) -> Result<NumericType>;
}

pub trait InterestRateModel {
    fn simulate_df(&self, request: &DiscountFactorRequest) -> Result<NumericType>;
    // fn simulate_fwd(&self, request: &ForwardRateRequest) -> Result<NumericType>;
}

pub trait EquityModel {
    fn simulate_equity(&self, request: &EquityRequest) -> Result<NumericType>;
}

pub trait NumerarieModel {
    fn simulate_numerarie(&self, date: Date) -> Result<NumericType>;
}

pub trait MarketModel: FxModel + InterestRateModel + EquityModel + NumerarieModel {}

pub trait MonteCarloEngine {
    fn generate_scenario(
        &self,
        event_dates: Vec<Date>,
        request: &Vec<SimulationDataRequest>,
    ) -> Result<Scenario>;

    fn generate_scenarios(
        &self,
        event_dates: Vec<Date>,
        request: &Vec<SimulationDataRequest>,
        num_scenarios: usize,
    ) -> Result<Vec<Scenario>> {
        let scenarios = (0..num_scenarios)
            .into_iter()
            .map(|_| self.generate_scenario(event_dates.clone(), request))
            .collect::<Result<Vec<Scenario>>>()?;
        Ok(scenarios)
    }
}

pub trait ParallelMonteCarloEngine: MonteCarloEngine + Sync + Send {
    fn initialize_for_parallelization(&self) {
        Tape::rewind_to_mark();
        self.put_on_tape();
        Tape::set_mark();
    }

    fn put_on_tape(&self);

    fn is_initialized(&self) -> bool;

    fn par_generate_scenarios(
        &self,
        event_dates: Vec<Date>,
        request: &Vec<SimulationDataRequest>,
        num_scenarios: usize,
    ) -> Result<Vec<Scenario>> {
        // one initialise per *thread* – Rayon guarantees that the closure is
        // called once per worker the first time it needs a job
        rayon::scope(|s| {
            s.spawn(|_| self.initialize_for_parallelization());
        });

        (0..num_scenarios)
            .into_par_iter()
            .map(|_| self.generate_scenario(event_dates.clone(), request))
            .collect()
    }
}

pub struct BlackScholesModel<'a> {
    reference_date: Date,
    local_currency: Currency,
    historical_data: &'a HistoricalData,
    fx: HashMap<(Currency, Currency), RwLock<NumericType>>,
    rates: IndexesForDate<Arc<RwLock<NumericType>>>,
    equities: HashMap<String, RwLock<NumericType>>,
    equity_vols: HashMap<String, RwLock<NumericType>>,
    fx_vols: HashMap<(Currency, Currency), RwLock<NumericType>>,
    is_initialized: RwLock<bool>,
    day_counter: DayCounter,
    time_handle: NumericType,
}

impl<'a> BlackScholesModel<'a> {
    pub fn new(
        reference_date: Date,
        local_currency: Currency,
        historical_data: &'a HistoricalData,
    ) -> Self {
        Self {
            reference_date,
            local_currency,
            historical_data,
            fx: HashMap::new(),
            rates: IndexesForDate::new(),
            equities: HashMap::new(),
            equity_vols: HashMap::new(),
            fx_vols: HashMap::new(),
            is_initialized: RwLock::new(false),
            day_counter: DayCounter::Actual360,
            time_handle: NumericType::zero(),
        }
    }

    pub fn clear(&mut self) {
        // clear all data
        self.fx.clear();

        self.equities.clear();
        self.equity_vols.clear();
        self.fx_vols.clear();
        *self.is_initialized.write().unwrap() = false;
    }

    pub fn initialize(&mut self) -> Result<()> {
        // fill spot data into hashmaps
        self.clear();
        self.historical_data
            .exchange_rates()
            .get_exchange_rates(self.reference_date)?
            .iter()
            .for_each(|(ccys, rate)| {
                self.fx
                    .entry(*ccys)
                    .or_insert_with(|| RwLock::new(NumericType::new(*rate)));
            });

        self.historical_data
            .volatilities()
            .get_fx_volatilities(self.reference_date)?
            .iter()
            .for_each(|(ccys, vol)| {
                self.fx_vols
                    .entry(*ccys)
                    .or_insert_with(|| RwLock::new(NumericType::new(*vol)));
            });

        self.rates = self
            .historical_data
            .term_structures()
            .get_term_structures(self.reference_date)?
            .into();

        Ok(())
    }

    pub fn fx(&self) -> &HashMap<(Currency, Currency), RwLock<NumericType>> {
        &self.fx
    }

    pub fn rates(&self) -> &IndexesForDate<Arc<RwLock<NumericType>>> {
        &self.rates
    }

    pub fn equities(&self) -> &HashMap<String, RwLock<NumericType>> {
        &self.equities
    }

    pub fn equity_vols(&self) -> &HashMap<String, RwLock<NumericType>> {
        &self.equity_vols
    }

    pub fn fx_vols(&self) -> &HashMap<(Currency, Currency), RwLock<NumericType>> {
        &self.fx_vols
    }

    pub fn reference_date(&self) -> Date {
        self.reference_date
    }

    pub fn local_currency(&self) -> Currency {
        self.local_currency
    }

    pub fn time_handle(&self) -> NumericType {
        self.time_handle
    }

    fn spot_in_local(&self, ccy: Currency) -> Result<NumericType> {
        if ccy == self.local_currency {
            return Ok(NumericType::one());
        }
        // try (ccy, local)  ─────────────────────────────────────────────
        if let Some(p) = self.fx.get(&(ccy, self.local_currency)) {
            return Ok(p.read().unwrap().clone());
        }
        // try (local, ccy)  ─────────────────────────────────────────────
        if let Some(p) = self.fx.get(&(self.local_currency, ccy)) {
            return Ok((NumericType::one() / p.read().unwrap().clone()).into());
        }
        // fall back to triangulation (may still need inversion)
        let l_over_ccy = triangulate_currencies(&self.fx, self.local_currency, ccy)?;
        Ok((NumericType::one() / l_over_ccy).into()) // l/ccy → ccy/l
    }

    fn fx_vol(&self, ccy: Currency) -> Result<NumericType> {
        // helper: 0 vol if the currency IS local, otherwise look both directions
        if ccy == self.local_currency {
            return Ok(NumericType::zero());
        }
        if let Some(v) = self.fx_vols.get(&(ccy, self.local_currency)) {
            return Ok(v.read().unwrap().clone());
        }
        if let Some(v) = self.fx_vols.get(&(self.local_currency, ccy)) {
            return Ok(v.read().unwrap().clone());
        }
        Err(ScriptingError::NotFoundError(format!(
            "Volatility not found for {} and {}",
            ccy, self.local_currency
        )))
    }

    fn time_step(&self, date: Date) -> NumericType {
        self.day_counter.year_fraction(self.reference_date, date)
    }
}

impl<'a> RandomNumberGenerator for BlackScholesModel<'a> {
    type Rng = rand::rngs::ThreadRng;

    fn set_rng(&self, _rng: Self::Rng) {
        // Placeholder for setting RNG
    }

    fn set_seed(&self, _seed: u64) {}

    fn gen_rand(&self) -> f64 {
        // let normal = Normal::new(0.0, 1.0).unwrap();
        let mut rng = rand::thread_rng();
        // Generate a random number from the standard normal distribution
        // This is a simple way to generate a random number, but you can use any RNG you prefer
        rng.sample::<f64, _>(StandardNormal)
    }
}

impl<'a> FxModel for BlackScholesModel<'a> {
    fn simulate_fx(&self, request: &ExchangeRateRequest) -> Result<NumericType> {
        if request.date() <= self.reference_date {
            // this already triangulates the currencies
            let s = self
                .historical_data
                .exchange_rates()
                .get_exchange_rate(
                    request.date(),
                    request.first_currency(),
                    request.second_currency(),
                )
                .map_err(|e| {
                    ScriptingError::NotFoundError(format!(
                        "Exchange rate not found for {} and {}: {}",
                        request.first_currency(),
                        request.second_currency(),
                        e
                    ))
                })?;
            return Ok(NumericType::new(s));
        }

        if request.first_currency() == request.second_currency() {
            return Ok(NumericType::new(1.0));
        }

        let s0_1 = self.spot_in_local(request.first_currency())?;
        let s0_2 = self.spot_in_local(request.second_currency())?;

        // time step (dt)
        let t: NumericType = (self.time_step(request.date()) - self.time_handle).into();

        let vol1 = self.fx_vol(request.first_currency())?;
        let vol2 = self.fx_vol(request.second_currency())?;

        // we need to get the risk free curves
        let local_rate = self
            .rates
            .get_by_currency(self.local_currency)?
            .fwd_rate_from_rate_definition(
                self.reference_date,
                request.date(),
                RateDefinition::new(
                    DayCounter::Actual360,
                    Compounding::Continuous,
                    Frequency::Annual,
                ),
            )?;

        let foreign_rate_1 = self
            .rates
            .get_by_currency(request.first_currency())?
            .fwd_rate_from_rate_definition(
                self.reference_date,
                request.date(),
                RateDefinition::new(
                    DayCounter::Actual360,
                    Compounding::Continuous,
                    Frequency::Annual,
                ),
            )?;

        let foreign_rate_2 = self
            .rates
            .get_by_currency(request.second_currency())?
            .fwd_rate_from_rate_definition(
                self.reference_date,
                request.date(),
                RateDefinition::new(
                    DayCounter::Actual360,
                    Compounding::Continuous,
                    Frequency::Annual,
                ),
            )?;

        let z = self.gen_rand();
        let rho = NumericType::zero(); // Assuming no correlation for simplicity, can be set to a value between -1 and 1
        let z_perp = self.gen_rand();
        let z1 = z;
        let z2 = rho * z + (-rho * rho + 1.0).sqrt() * z_perp;

        let fx_1_l = s0_1
            * ((foreign_rate_1 - local_rate - vol1 * vol1 * 0.5) * t + vol1 * z1 * t.sqrt()).exp();

        let fx_2_l = s0_2
            * ((foreign_rate_2 - local_rate - vol2 * vol2 * 0.5) * t + vol2 * z2 * t.sqrt()).exp();

        let st = fx_1_l / fx_2_l;
        Ok(st.into())
    }
}

impl<'a> InterestRateModel for BlackScholesModel<'a> {
    fn simulate_df(&self, request: &DiscountFactorRequest) -> Result<NumericType> {
        if request.to_date() <= request.from_date() {
            return Ok(NumericType::new(1.0));
        }

        let df = self
            .rates
            .get_by_currency(self.local_currency)?
            .discount_factor(request.from_date(), request.to_date())?;
        return Ok(df);
    }
}

impl<'a> NumerarieModel for BlackScholesModel<'a> {
    fn simulate_numerarie(&self, date: Date) -> Result<NumericType> {
        if date <= self.reference_date {
            return Ok(NumericType::new(1.0));
        }

        // Get the discount factor for the local currency
        let df = self
            .rates
            .get_by_currency(self.local_currency)?
            .discount_factor(self.reference_date, date)?;
        Ok((NumericType::one() / df).into())
    }
}

impl<'a> MonteCarloEngine for BlackScholesModel<'a> {
    fn generate_scenario(
        &self,
        event_dates: Vec<Date>,
        request: &Vec<SimulationDataRequest>,
    ) -> Result<Scenario> {
        event_dates
            .into_iter()
            .zip(request.iter())
            .map(|(date, req)| {
                let numerarie = self.simulate_numerarie(date)?;
                let dfs: Vec<NumericType> = req
                    .dfs()
                    .iter()
                    .map(|df| self.simulate_df(df))
                    .collect::<Result<Vec<_>>>()?;
                let fxs: Vec<NumericType> = req
                    .fxs()
                    .iter()
                    .map(|fx| self.simulate_fx(fx))
                    .collect::<Result<Vec<_>>>()?;

                Ok(SimulationData::new(
                    numerarie,
                    dfs,
                    Vec::new(), // fwds are not implemented yet
                    fxs,
                    Vec::new(), // equities are not implemented yet
                ))
            })
            .collect::<Result<Vec<_>>>()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::termstructure::{TermStructure, TermStructureKey, TermStructureType};

    fn market_data(reference_date: Date) -> HistoricalData {
        let mut store = HistoricalData::new();
        store.mut_exchange_rates().add_exchange_rate(
            reference_date,
            Currency::CLP,
            Currency::USD,
            800.0,
        );

        store.mut_exchange_rates().add_exchange_rate(
            reference_date,
            Currency::JPY,
            Currency::USD,
            142.0,
        );

        store.mut_volatilities().add_fx_volatility(
            reference_date,
            Currency::USD,
            Currency::CLP,
            0.0,
        );

        store.mut_volatilities().add_fx_volatility(
            reference_date,
            Currency::CLP,
            Currency::USD,
            0.0,
        );

        store.mut_volatilities().add_fx_volatility(
            reference_date,
            Currency::JPY,
            Currency::USD,
            0.0,
        );

        store.mut_volatilities().add_fx_volatility(
            reference_date,
            Currency::USD,
            Currency::JPY,
            0.0,
        );

        store.mut_volatilities().add_fx_volatility(
            reference_date,
            Currency::CLP,
            Currency::JPY,
            0.0,
        );

        // general
        let year_fractions = vec![1.0];
        let interpolator = Interpolator::Linear;
        let enable_extrapolation = true;
        let rate_definition = RateDefinition::new(
            DayCounter::Actual360,
            Compounding::Continuous,
            Frequency::Annual,
        );
        let term_structure_type = TermStructureType::FlatForward;

        // CLP term structure
        let clp_ts_key = TermStructureKey::new(Currency::CLP, true, Some("CLP".to_string()));
        let clp_rate = vec![0.03];

        let clp_ts = TermStructure::new(
            clp_ts_key,
            year_fractions.clone(),
            clp_rate,
            interpolator,
            enable_extrapolation,
            rate_definition,
            term_structure_type,
        );

        // USD term structure
        let usd_ts_key = TermStructureKey::new(Currency::USD, true, Some("USD".to_string()));
        let usd_rate = vec![0.02];

        let usd_ts = TermStructure::new(
            usd_ts_key,
            year_fractions.clone(),
            usd_rate,
            interpolator,
            enable_extrapolation,
            rate_definition,
            term_structure_type,
        );

        store
            .mut_term_structures()
            .add_term_structure(reference_date, clp_ts);
        store
            .mut_term_structures()
            .add_term_structure(reference_date, usd_ts);

        // JPY term structure
        let jpy_ts_key = TermStructureKey::new(Currency::JPY, true, Some("JPY".to_string()));
        let jpy_rate = vec![0.01];
        let jpy_ts = TermStructure::new(
            jpy_ts_key,
            year_fractions,
            jpy_rate,
            interpolator,
            enable_extrapolation,
            rate_definition,
            term_structure_type,
        );
        store
            .mut_term_structures()
            .add_term_structure(reference_date, jpy_ts);

        store
    }

    /// 1) Local currency **CLP**  – check USD/CLP × CLP/USD ≈ 1
    #[test]
    fn reciprocity_with_domestic_clp() {
        let today = Date::new(2025, 6, 5);
        let binding = market_data(today);
        let mut model = BlackScholesModel::new(
            today,
            Currency::CLP, // domestic
            &binding,
        );
        model.initialize().unwrap();

        // one year forward so we run through `simulate_fx`
        let t1y = Date::new(2026, 6, 5);

        let usd_clp = ExchangeRateRequest::new(Currency::USD, Currency::CLP, t1y);
        let clp_usd = ExchangeRateRequest::new(Currency::CLP, Currency::USD, t1y);

        let r1 = model.simulate_fx(&usd_clp).unwrap().value();
        let r2 = model.simulate_fx(&clp_usd).unwrap().value();

        // must be exact reciprocals
        assert!((r1 * r2 - 1.0).abs() < 1e-12);
    }

    /// 2) Local currency **USD** – this is the configuration that was broken
    #[test]
    fn reciprocity_with_domestic_usd() {
        let today = Date::new(2025, 6, 5);
        let binding = market_data(today);
        let mut model = BlackScholesModel::new(
            today,
            Currency::USD, // domestic
            &binding,
        );
        model.initialize().unwrap();

        let t1y = Date::new(2026, 6, 5);

        let usd_clp = ExchangeRateRequest::new(Currency::USD, Currency::CLP, t1y);
        let clp_usd = ExchangeRateRequest::new(Currency::CLP, Currency::USD, t1y);

        let r1 = model.simulate_fx(&usd_clp).unwrap().value();
        let r2 = model.simulate_fx(&clp_usd).unwrap().value();

        // before the patch r1·r2 ≫ 1 (≈6.4 × 10⁵); after the patch it is 1
        assert!((r1 * r2 - 1.0).abs() < 1e-12);
    }

    #[test]
    fn forward_price_matches_interest_parity_requests_clp_usd() {
        // domestic = CLP
        let today = Date::new(2025, 6, 5);
        let hd = market_data(today);

        let mut model = BlackScholesModel::new(today, Currency::CLP, &hd);
        model.initialize().unwrap();

        let fut = Date::new(2025, 12, 4); // ≈ 0.5y
        let t = model.day_counter.year_fraction(today, fut);

        // *** note the constructor signature: (first, second, date) ***
        let req = ExchangeRateRequest::new(Currency::CLP, Currency::USD, fut);

        let fwd = model.simulate_fx(&req).unwrap().value();

        let spot = 800.0; // CLP/USD stored in `market_data`
        let r_dom = 0.03; // CLP curve
        let r_for = 0.02; // USD curve
        let should_be = spot * f64::exp((r_dom - r_for) * t.value());

        assert!((fwd - should_be).abs() < 1e-4); // now passes
    }

    #[test]
    fn forward_price_matches_interest_parity_requests_usd_clp() {
        // domestic = USD
        let today = Date::new(2025, 6, 5);
        let hd = market_data(today);

        let mut model = BlackScholesModel::new(today, Currency::USD, &hd);
        model.initialize().unwrap();

        let fut = Date::new(2025, 12, 4); // ≈ 0.5y
        let t = model.day_counter.year_fraction(today, fut);

        // *** note the constructor signature: (first, second, date) ***
        let req = ExchangeRateRequest::new(Currency::USD, Currency::CLP, fut);

        let fwd = model.simulate_fx(&req).unwrap().value();

        let spot = 1.0 / 800.0; // USD/CLP stored in `market_data`
        let r_dom = 0.02; // USD curve
        let r_for = 0.03; // CLP curve
        let should_be = spot * f64::exp((r_dom - r_for) * t.value());

        assert!((fwd - should_be).abs() < 1e-4); // now passes
    }

    #[test]
    fn forward_price_matches_interest_parity_requests_jpy_usd_local_clp() {
        // domestic = CLP
        let today = Date::new(2025, 6, 5);
        let hd = market_data(today);

        let mut model = BlackScholesModel::new(today, Currency::CLP, &hd);
        model.initialize().unwrap();

        let fut = Date::new(2025, 12, 4); // ≈ 0.5y
        let t = model.day_counter.year_fraction(today, fut);

        // *** note the constructor signature: (first, second, date) ***
        let req = ExchangeRateRequest::new(Currency::JPY, Currency::USD, fut);

        let fwd = model.simulate_fx(&req).unwrap().value();

        let spot = 142.0; // JPY/USD stored in `market_data`
        let r_dom = 0.01; // JPY curve
        let r_for = 0.02; // USD curve
        let should_be = spot * f64::exp((r_dom - r_for) * t.value());

        assert!((fwd - should_be).abs() < 1e-4); // now passes
    }
}
