use crate::{
    data::termstructure::TermStructureStore,
    utils::errors::{Result, ScriptingError},
};
use rustatlas::prelude::*;
use serde::{Deserialize, Serialize};
use std::{
    collections::{HashMap, HashSet, VecDeque},
    sync::{Arc, RwLock},
};

type Name = String;

#[derive(Default, Serialize, Deserialize)]
pub struct ExchangeRates {
    exchange_rates: HashMap<Date, HashMap<(Currency, Currency), f64>>,
    exchange_rate_cache: RwLock<HashMap<Date, Arc<RwLock<HashMap<(Currency, Currency), f64>>>>>,
}

impl ExchangeRates {
    pub fn new() -> Self {
        ExchangeRates {
            exchange_rates: HashMap::new(),
            exchange_rate_cache: RwLock::new(HashMap::new()),
        }
    }

    pub fn add_exchange_rate(
        &mut self,
        reference_date: Date,
        from_currency: Currency,
        to_currency: Currency,
        rate: f64,
    ) {
        self.exchange_rates
            .entry(reference_date)
            .or_default()
            .insert((from_currency, to_currency), rate);
    }

    pub fn get_exchange_rates(
        &self,
        reference_date: Date,
    ) -> Result<&HashMap<(Currency, Currency), f64>> {
        self.exchange_rates
            .get(&reference_date)
            .ok_or(ScriptingError::NotFoundError(format!(
                "No exchange rates found for reference date: {}",
                reference_date
            )))
    }

    pub fn get_exchange_rate(
        &self,
        reference_date: Date,
        first_ccy: Currency,
        second_ccy: Currency,
    ) -> Result<f64> {
        let first_ccy = first_ccy;
        let second_ccy = second_ccy;

        if first_ccy == second_ccy {
            return Ok(1.0);
        }

        let cache_key = (first_ccy, second_ccy);

        let storage =
            self.exchange_rates
                .get(&reference_date)
                .ok_or(ScriptingError::NotFoundError(format!(
                    "No exchange rates found for reference date: {}",
                    reference_date
                )))?;

        let mut cache_guard = self.exchange_rate_cache.write().unwrap();
        let cache_entry = cache_guard
            .entry(reference_date)
            .or_insert_with(|| Arc::new(RwLock::new(HashMap::new())));
        let mut mutable_cache = cache_entry.write().unwrap();

        if let Some(cached_rate) = mutable_cache.get(&cache_key) {
            return Ok(*cached_rate);
        }

        let mut q: VecDeque<(Currency, f64)> = VecDeque::new();
        let mut visited: HashSet<Currency> = HashSet::new();
        q.push_back((first_ccy, 1.0));
        visited.insert(first_ccy);

        while let Some((current_ccy, rate)) = q.pop_front() {
            for (&(source, dest), &map_rate) in storage {
                if source == current_ccy && !visited.contains(&dest) {
                    let new_rate = rate * map_rate;
                    if dest == second_ccy {
                        let new_rate_value = new_rate.into();
                        mutable_cache.insert((first_ccy, second_ccy), new_rate_value);
                        mutable_cache.insert((second_ccy, first_ccy), 1.0 / new_rate_value);
                        return Ok(new_rate_value);
                    }
                    visited.insert(dest);
                    q.push_back((dest, new_rate.into()));
                } else if dest == current_ccy && !visited.contains(&source) {
                    let new_rate = rate / map_rate;
                    if source == second_ccy {
                        let new_rate_value = new_rate.into();
                        mutable_cache.insert((first_ccy, second_ccy), new_rate_value);
                        mutable_cache.insert((second_ccy, first_ccy), 1.0 / new_rate_value);
                        return Ok(new_rate_value);
                    }
                    visited.insert(source);
                    q.push_back((source, new_rate.into()));
                }
            }
        }
        Err(ScriptingError::NotFoundError(format!(
            "No exchange rate found between {:?} and {:?}",
            first_ccy, second_ccy
        )))
    }
}

pub fn triangulate_currencies(
    exchange_rates: &HashMap<(Currency, Currency), RwLock<NumericType>>,
    ccy1: Currency,
    ccy2: Currency,
) -> Result<NumericType> {
    // 1. trivial case
    if ccy1 == ccy2 {
        return Ok(NumericType::one());
    }

    // 2. try direct quote
    if let Some(rate) = exchange_rates.get(&(ccy1, ccy2)) {
        return Ok(rate.read().unwrap().clone());
    }
    // 3. try inverse quote
    if let Some(rate) = exchange_rates.get(&(ccy2, ccy1)) {
        let val = NumericType::one() / rate.read().unwrap().clone();
        return Ok(val.into());
    }

    // 4. breadth-first search for any path
    let mut visited: HashSet<Currency> = HashSet::new();
    let mut q: VecDeque<(Currency, NumericType)> = VecDeque::new();
    q.push_back((ccy1, NumericType::one()));

    while let Some((cur_ccy, acc_rate)) = q.pop_front() {
        if cur_ccy == ccy2 {
            return Ok(acc_rate);
        }
        if !visited.insert(cur_ccy) {
            continue; // already expanded
        }

        // explore neighbours
        for ((base, terms), quote) in exchange_rates.iter() {
            let val = quote.read().unwrap().clone();
            if *base == cur_ccy && !visited.contains(terms) {
                // 1 `base` = quote `terms`
                q.push_back((*terms, (acc_rate * val).into()));
            } else if *terms == cur_ccy && !visited.contains(base) {
                // 1 `terms` = quote `base`  ⇒  1 `base` = 1/quote `terms`
                q.push_back((*base, (acc_rate / val).into()));
            }
        }
    }

    Err(ScriptingError::NotFoundError(format!(
        "Unable to triangulate {} → {}",
        ccy1, ccy2
    )))
}

#[derive(Default, Clone, Serialize, Deserialize)]
pub struct Fixings {
    fixings: HashMap<Date, HashMap<Name, f64>>,
}

impl Fixings {
    pub fn new() -> Self {
        Fixings {
            fixings: HashMap::new(),
        }
    }

    pub fn add_fixing(&mut self, reference_date: Date, name: Name, value: f64) {
        self.fixings
            .entry(reference_date)
            .or_default()
            .insert(name, value);
    }

    pub fn get_fixing(&self, reference_date: Date, name: &Name) -> Option<f64> {
        self.fixings
            .get(&reference_date)
            .and_then(|map| map.get(name).cloned())
    }
}
#[derive(Default, Clone, Serialize, Deserialize)]
pub struct Volatilities {
    equity_vol: HashMap<Date, HashMap<String, f64>>,
    fx_vol: HashMap<Date, HashMap<(Currency, Currency), f64>>,
}

impl Volatilities {
    pub fn new() -> Self {
        Volatilities {
            equity_vol: HashMap::new(),
            fx_vol: HashMap::new(),
        }
    }

    pub fn add_equity_volatility(
        &mut self,
        reference_date: Date,
        equity_id: String,
        volatility: f64,
    ) {
        self.equity_vol
            .entry(reference_date)
            .or_default()
            .insert(equity_id, volatility);
    }

    pub fn get_equity_volatility(&self, reference_date: Date, equity_id: &str) -> Option<f64> {
        self.equity_vol
            .get(&reference_date)
            .and_then(|map| map.get(equity_id).cloned())
    }

    pub fn add_fx_volatility(
        &mut self,
        reference_date: Date,
        from_currency: Currency,
        to_currency: Currency,
        volatility: f64,
    ) {
        self.fx_vol
            .entry(reference_date)
            .or_default()
            .insert((from_currency, to_currency), volatility);
    }

    pub fn get_fx_volatility(
        &self,
        reference_date: Date,
        from_currency: Currency,
        to_currency: Currency,
    ) -> Result<f64> {
        self.fx_vol
            .get(&reference_date)
            .and_then(|map| map.get(&(from_currency, to_currency)).cloned())
            .ok_or(ScriptingError::NotFoundError(format!(
                "No FX volatility found for {} to {} on {}",
                from_currency, to_currency, reference_date
            )))
    }

    pub fn get_fx_volatilities(
        &self,
        reference_date: Date,
    ) -> Result<&HashMap<(Currency, Currency), f64>> {
        self.fx_vol
            .get(&reference_date)
            .ok_or(ScriptingError::NotFoundError(format!(
                "No FX volatilities found for reference date: {}",
                reference_date
            )))
    }
}

pub struct HistoricalData {
    exchange_rates: ExchangeRates,
    fixings: Fixings,
    volatilities: Volatilities,
    term_structures: TermStructureStore<f64>,
}

impl HistoricalData {
    pub fn new() -> Self {
        HistoricalData {
            exchange_rates: ExchangeRates::new(),
            fixings: Fixings::new(),
            volatilities: Volatilities::new(),
            term_structures: TermStructureStore::new(),
        }
    }

    pub fn exchange_rates(&self) -> &ExchangeRates {
        &self.exchange_rates
    }

    pub fn fixings(&self) -> &Fixings {
        &self.fixings
    }

    pub fn volatilities(&self) -> &Volatilities {
        &self.volatilities
    }

    pub fn term_structures(&self) -> &TermStructureStore<f64> {
        &self.term_structures
    }

    pub fn mut_exchange_rates(&mut self) -> &mut ExchangeRates {
        &mut self.exchange_rates
    }
    pub fn mut_fixings(&mut self) -> &mut Fixings {
        &mut self.fixings
    }
    pub fn mut_volatilities(&mut self) -> &mut Volatilities {
        &mut self.volatilities
    }
    pub fn mut_term_structures(&mut self) -> &mut TermStructureStore<f64> {
        &mut self.term_structures
    }
}
