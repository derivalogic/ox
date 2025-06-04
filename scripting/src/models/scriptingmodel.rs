use std::{collections::HashMap, sync::RwLock};

use crate::{
    data::termstructure::{TermStructure, TermStructureStore},
    prelude::*,
};
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use rustatlas::prelude::*;

pub trait FxModel {
    fn simulate_fx(&self, request: &ExchangeRateRequest) -> Result<NumericType>;
}

pub trait InterestRateModel {
    fn simulate_df(&self, request: &DiscountFactorRequest) -> Result<NumericType>;
    fn simulate_fwd(&self, request: &ForwardRateRequest) -> Result<NumericType>;
}

pub trait EquityModel {
    fn simulate_equity(&self, request: &EquityRequest) -> Result<NumericType>;
}

pub trait MarketModel: FxModel + InterestRateModel + EquityModel {}

pub trait StochasticModel {
    type Rng;
    fn set_rng(&self, rng: Self::Rng);
    fn set_seed(&self, seed: u64);
    fn gen_rand(&self) -> f64;
    fn time_step(&self, date: Date) -> NumericType;
}

pub trait MonteCarloEngine {
    fn generate_scenario(&self, request: &SimulationDataRequest) -> Result<SimulationData>;
    fn generate_scenarios(
        &self,
        request: &SimulationDataRequest,
        num_scenarios: usize,
    ) -> Result<Vec<SimulationData>> {
        let scenarios = (0..num_scenarios)
            .into_iter()
            .map(|_| self.generate_scenario(request))
            .collect::<Result<Vec<SimulationData>>>()?;
        Ok(scenarios)
    }
}

pub trait ParallelMonteCarloEngine: MonteCarloEngine + Sync + Send {
    fn initialize(&self) {
        Tape::rewind_to_mark();
        self.put_on_tape();
        Tape::set_mark();
    }

    fn put_on_tape(&self);

    fn is_initialized(&self) -> bool;

    fn generate_parallel_scenarios(
        &self,
        request: &SimulationDataRequest,
        num_scenarios: usize,
    ) -> Result<Vec<SimulationData>> {
        // one initialise per *thread* – Rayon guarantees that the closure is
        // called once per worker the first time it needs a job
        rayon::scope(|s| {
            s.spawn(|_| self.initialize());
        });

        (0..num_scenarios)
            .into_par_iter()
            .map(|_| self.generate_scenario(request))
            .collect()
    }
}

pub struct BlackScholesModel<'a> {
    reference_date: Date,
    local_currency: Currency,
    historical_data: &'a HistoricalData,
    fx: HashMap<(Currency, Currency), RwLock<NumericType>>,
    rates: TermStructureStore<RwLock<NumericType>>,
    equities: HashMap<String, RwLock<NumericType>>,
    equity_vols: HashMap<String, RwLock<NumericType>>,
    fx_vols: HashMap<(Currency, Currency), RwLock<NumericType>>,
    is_initialized: RwLock<bool>,
    day_counter: DayCounter,
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
            rates: TermStructures::new(),
            equities: HashMap::new(),
            equity_vols: HashMap::new(),
            fx_vols: HashMap::new(),
            is_initialized: RwLock::new(false),
            day_counter: DayCounter::Actual360,
        }
    }

    pub fn clear(&mut self) {
        // clear all data
        self.fx.clear();
        self.rates.clear();
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

        let tmp_rates = self
            .historical_data
            .term_structures()
            .get_term_structures(self.reference_date)?;

        
        Ok(())
    }

    pub fn fx(&self) -> &HashMap<(Currency, Currency), RwLock<NumericType>> {
        &self.fx
    }

    pub fn rates(&self) -> &HashMap<Currency, RwLock<NumericType>> {
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
}

impl<'a> StochasticModel for BlackScholesModel<'a> {
    type Rng = rand::rngs::ThreadRng;

    fn set_rng(&self, _rng: Self::Rng) {
        // Placeholder for setting RNG
    }

    fn set_seed(&self, _seed: u64) {
        // Placeholder for setting seed
    }

    fn gen_rand(&self) -> f64 {
        rand::random()
    }

    fn time_step(&self, date: Date) -> NumericType {
        self.day_counter.year_fraction(self.reference_date, date)
    }
}

impl<'a> FxModel for BlackScholesModel<'a> {
    fn simulate_fx(&self, request: &ExchangeRateRequest) -> Result<NumericType> {
        if request.date() <= self.reference_date {
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
        // Check if the exchange rate is already cached
        // If not, retrieve it from historical data and cache it
        let s0 = match self
            .fx
            .get(&(request.first_currency(), request.second_currency()))
        {
            Some(fx_rate) => fx_rate.read().unwrap().clone(),
            None => triangulate_currencies(
                &self.fx,
                request.first_currency(),
                request.second_currency(),
            )?,
        };

        // time step (dt)
        let t = self.time_step(request.date());

        // volatility
        let volatility = match self
            .fx_vols
            .get(&(request.first_currency(), request.second_currency()))
        {
            Some(vol) => vol.read().unwrap().clone(),
            None => {
                return Err(ScriptingError::NotFoundError(format!(
                    "Volatility not found for {} and {}",
                    request.first_currency(),
                    request.second_currency()
                )));
            }
        };

        // we need to get the risk free curves

        let drift = NumericType::zero(); // Placeholder for drift, can be set based on risk-free rates

        // Black-Scholes simulation
        let z = self.gen_rand();
        let st = s0 * ((drift - volatility * volatility * 0.5) * t + volatility * z * t.sqrt());
        Ok(st.into())
    }
}

impl<'a> MonteCarloEngine for BlackScholesModel<'a> {
    fn generate_scenario(&self, request: &SimulationDataRequest) -> Result<SimulationData> {
        // Implement the logic to generate a single scenario based on the request
        // This is a placeholder implementation
        Ok(SimulationData::default())
    }
}

impl<'a> ParallelMonteCarloEngine for BlackScholesModel<'a> {
    fn put_on_tape(&self) {
        self.fx.iter().for_each(|((_, _), f)| {
            f.write().unwrap().put_on_tape();
        });

        self.rates.iter().for_each(|(_, r)| {
            r.write().unwrap().put_on_tape();
        });

        self.equities.iter().for_each(|(_, e)| {
            e.write().unwrap().put_on_tape();
        });

        self.equity_vols.iter().for_each(|(_, v)| {
            v.write().unwrap().put_on_tape();
        });

        self.fx_vols.iter().for_each(|((_, _), v)| {
            v.write().unwrap().put_on_tape();
        });
    }

    fn is_initialized(&self) -> bool {
        self.is_initialized.read().unwrap().clone()
    }
}

pub enum Model {}
