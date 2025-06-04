use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

use crate::{
    data::termstructure::{DiscountFactorProvider, ForwardRateProvider, IndexesForDate},
    prelude::*,
};
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

pub trait StochasticModel {
    type Rng;
    fn set_rng(&self, rng: Self::Rng);
    fn set_seed(&self, seed: u64);
    fn gen_rand(&self) -> f64;
    fn time_step(&self, date: Date) -> NumericType;
}

pub trait MonteCarloEngine {
    fn generate_scenario(
        &self,
        event_dates: Vec<Date>,
        request: &Vec<SimulationDataRequest>,
    ) -> Result<Scenario>;
    // fn generate_scenarios(
    //     &self,
    //     request: &Vec<SimulationDataRequest>,
    //     num_scenarios: usize,
    // ) -> Result<Vec<Scenario>> {
    //     let scenarios = (0..num_scenarios)
    //         .into_iter()
    //         .map(|_| self.generate_scenario_for_date(request))
    //         .collect::<Result<Vec<SimulationData>>>()?;
    //     Ok(scenarios)
    // }

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
        // let normal = Normal::new(0.0, 1.0).unwrap();
        let mut rng = rand::thread_rng();
        // Generate a random number from the standard normal distribution
        // This is a simple way to generate a random number, but you can use any RNG you prefer
        rng.sample::<f64, _>(StandardNormal)
    }

    fn time_step(&self, date: Date) -> NumericType {
        self.day_counter.year_fraction(self.reference_date, date)
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
        // Check if the exchange rate is already cached
        // If not, retrieve it from historical data and cache it

        let s0_1 = match self
            .fx
            .get(&(request.first_currency(), self.local_currency))
        {
            Some(fx_rate) => fx_rate.read().unwrap().clone(),
            None => {
                triangulate_currencies(&self.fx, self.local_currency, request.first_currency())?
            }
        };

        let s0_2 = match self
            .fx
            .get(&(request.second_currency(), self.local_currency))
        {
            Some(fx_rate) => fx_rate.read().unwrap().clone(),
            None => {
                triangulate_currencies(&self.fx, self.local_currency, request.second_currency())?
            }
        };

        // time step (dt)
        let t: ADNumber = (self.time_step(request.date()) - self.time_handle).into();

        // volatility
        let vol1 = match self
            .fx_vols
            .get(&(request.first_currency(), self.local_currency))
        {
            Some(vol) => vol.read().unwrap().clone(),
            None => {
                if request.first_currency() == self.local_currency {
                    NumericType::zero()
                } else {
                    return Err(ScriptingError::NotFoundError(format!(
                        "Volatility not found for {} and {}",
                        request.first_currency(),
                        self.local_currency
                    )));
                }
            }
        };
        let vol2 = match self
            .fx_vols
            .get(&(request.second_currency(), self.local_currency))
        {
            Some(vol) => vol.read().unwrap().clone(),
            None => {
                if request.second_currency() == self.local_currency {
                    NumericType::zero()
                } else {
                    return Err(ScriptingError::NotFoundError(format!(
                        "Volatility not found for {} and {}",
                        request.second_currency(),
                        self.local_currency
                    )));
                }
            }
        };

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

        let z1 = self.gen_rand();
        let fx_1_l = s0_1
            * ((foreign_rate_1 - local_rate - vol1 * vol1 * 0.5) * t + vol1 * z1 * t.sqrt()).exp();

        let z2 = self.gen_rand();
        let fx_2_l = s0_2
            * ((foreign_rate_2 - local_rate - vol2 * vol2 * 0.5) * t + vol2 * z2 * t.sqrt()).exp();

        // we need to arrange so we effectively return ccy1 / ccy2
        if request.first_currency() == self.local_currency {
            // we have ccy1 as local currency, so we return ccy1 / ccy2
            let st = fx_2_l / fx_1_l;
            return Ok(st.into());
        }
        if request.second_currency() == self.local_currency {
            // we have ccy2 as local currency, so we return ccy1 / ccy2
            let st = fx_1_l / fx_2_l;
            return Ok(st.into());
        } else {
            // we have both currencies as foreign, so we return ccy1 / ccy2
            // this is the same as fx_1_l / fx_2_l
            let st = fx_1_l / fx_2_l;
            return Ok(st.into());
        }
        // let st = fx_2_l / fx_1_l;
    }
}

impl<'a> InterestRateModel for BlackScholesModel<'a> {
    fn simulate_df(&self, request: &DiscountFactorRequest) -> Result<NumericType> {
        if request.date() <= self.reference_date {
            return Ok(NumericType::new(1.0));
        }

        let df = self
            .rates
            .get_by_currency(self.local_currency)?
            .discount_factor(self.reference_date, request.date())?;
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

impl<'a> ParallelMonteCarloEngine for BlackScholesModel<'a> {
    fn put_on_tape(&self) {
        self.fx.iter().for_each(|((_, _), f)| {
            f.write().unwrap().put_on_tape();
        });

        // self.equities.iter().for_each(|(_, e)| {
        //     e.write().unwrap().put_on_tape();
        // });

        // self.equity_vols.iter().for_each(|(_, v)| {
        //     v.write().unwrap().put_on_tape();
        // });

        self.fx_vols.iter().for_each(|((_, _), v)| {
            v.write().unwrap().put_on_tape();
        });
    }

    fn is_initialized(&self) -> bool {
        self.is_initialized.read().unwrap().clone()
    }
}

pub enum Model {}
