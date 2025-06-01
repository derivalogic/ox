use std::{
    collections::{HashMap, HashSet, VecDeque},
    sync::{Arc, Mutex},
};

use crate::prelude::*;
/// # ExchangeRateStore
/// A store for exchange rates.
/// Exchange rates are stored as a map of pairs of currencies to rates.
///
/// ## Details
/// - Exchange rates are stored as a map of pairs of currencies to rates.
/// - NumericTypehe exchange rate between two currencies is calculated by traversing the graph of exchange rates.
#[derive(Clone)]
pub struct ExchangeRateStore {
    reference_date: Date,
    exchange_rate_map: HashMap<(Currency, Currency), NumericType>,
    volatility_map: HashMap<(Currency, Currency), NumericType>,
    exchange_rate_cache: Arc<Mutex<HashMap<(Currency, Currency), NumericType>>>,
}

impl ExchangeRateStore {
    pub fn new(date: Date) -> ExchangeRateStore {
        ExchangeRateStore {
            reference_date: date,
            volatility_map: HashMap::new(),
            exchange_rate_map: HashMap::new(),
            exchange_rate_cache: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn with_exchange_rates(
        &mut self,
        exchange_rate_map: HashMap<(Currency, Currency), NumericType>,
    ) -> &mut Self {
        self.exchange_rate_map = exchange_rate_map;
        self
    }

    pub fn add_exchange_rate(
        &mut self,
        currency1: Currency,
        currency2: Currency,
        rate: NumericType,
    ) {
        self.exchange_rate_map.insert((currency1, currency2), rate);
    }

    pub fn reference_date(&self) -> Date {
        self.reference_date
    }

    pub fn add_volatility(
        &mut self,
        currency1: Currency,
        currency2: Currency,
        volatility: NumericType,
    ) {
        self.volatility_map
            .insert((currency1, currency2), volatility);
    }

    pub fn get_volatility(&self, currency1: Currency, currency2: Currency) -> Result<NumericType> {
        if let Some(vol) = self.volatility_map.get(&(currency1, currency2)) {
            Ok(*vol)
        } else if let Some(vol) = self.volatility_map.get(&(currency2, currency1)) {
            Ok(*vol)
        } else {
            Err(AtlasError::NotFoundErr(format!(
                "No volatility for pair {:?}/{:?}",
                currency1, currency2
            )))
        }
    }

    pub fn get_volatility_map(&self) -> HashMap<(Currency, Currency), NumericType> {
        self.volatility_map.clone()
    }

    pub fn get_exchange_rate_map(&self) -> HashMap<(Currency, Currency), NumericType> {
        self.exchange_rate_map.clone()
    }

    pub fn get_exchange_rate(
        &self,
        first_ccy: Currency,
        second_ccy: Currency,
    ) -> Result<NumericType> {
        let first_ccy = first_ccy;
        let second_ccy = second_ccy;

        if first_ccy == second_ccy {
            return Ok(NumericType::from(1.0));
        }

        let cache_key = (first_ccy, second_ccy);
        if let Some(cached_rate) = self.exchange_rate_cache.lock().unwrap().get(&cache_key) {
            return Ok(*cached_rate);
        }

        let mut q: VecDeque<(Currency, NumericType)> = VecDeque::new();
        let mut visited: HashSet<Currency> = HashSet::new();
        q.push_back((first_ccy, NumericType::from(1.0)));
        visited.insert(first_ccy);

        let mut mutable_cache = self.exchange_rate_cache.lock().unwrap();
        while let Some((current_ccy, rate)) = q.pop_front() {
            for (&(source, dest), &map_rate) in &self.exchange_rate_map {
                if source == current_ccy && !visited.contains(&dest) {
                    let new_rate = rate * map_rate;
                    if dest == second_ccy {
                        mutable_cache.insert((first_ccy, second_ccy), new_rate);
                        mutable_cache.insert((second_ccy, first_ccy), 1.0 / new_rate);
                        return Ok(new_rate);
                    }
                    visited.insert(dest);
                    q.push_back((dest, new_rate));
                } else if dest == current_ccy && !visited.contains(&source) {
                    let new_rate = rate / map_rate;
                    if source == second_ccy {
                        mutable_cache.insert((first_ccy, second_ccy), new_rate);
                        mutable_cache.insert((second_ccy, first_ccy), 1.0 / new_rate);
                        return Ok(new_rate);
                    }
                    visited.insert(source);
                    q.push_back((source, new_rate));
                }
            }
        }
        Err(AtlasError::NotFoundErr(format!(
            "No exchange rate found between {:?} and {:?}",
            first_ccy, second_ccy
        )))
    }
}

impl AdvanceExchangeRateStoreInTime for ExchangeRateStore {
    fn advance_to_period(
        &self,
        period: Period,
        index_store: &IndexStore,
    ) -> Result<ExchangeRateStore> {
        let new_date = self.reference_date + period;
        self.advance_to_date(new_date, index_store)
    }

    fn advance_to_date(&self, date: Date, index_store: &IndexStore) -> Result<ExchangeRateStore> {
        if self.reference_date() != index_store.reference_date() {
            return Err(AtlasError::InvalidValueErr(format!(
                "Reference date of exchange rate store and index store do not match"
            )));
        }

        let mut new_store = ExchangeRateStore::new(date);
        for ((ccy1, ccy2), fx) in self.exchange_rate_map.iter() {
            let compound_factor = index_store.currency_forescast_factor(*ccy1, *ccy2, date);
            match compound_factor {
                Ok(cf) => new_store.add_exchange_rate(*ccy1, *ccy2, *fx * cf),
                Err(_) => {
                    // If the compound factor is not available, we use the last fx rate
                    new_store.add_exchange_rate(*ccy1, *ccy2, *fx);
                }
            }
        }
        Ok(new_store)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_same_currency() {
        let ref_date = Date::new(2021, 1, 1);
        let manager: ExchangeRateStore = ExchangeRateStore::new(ref_date);
        assert_eq!(
            manager
                .get_exchange_rate(Currency::USD, Currency::USD)
                .unwrap(),
            1.0
        );
    }

    #[test]
    fn test_cache() {
        let ref_date = Date::new(2021, 1, 1);
        let manager = ExchangeRateStore {
            reference_date: ref_date,
            volatility_map: HashMap::new(),
            exchange_rate_map: {
                let mut map = HashMap::new();
                map.insert((Currency::USD, Currency::EUR), 0.85);
                map
            },
            exchange_rate_cache: Arc::new(Mutex::new(HashMap::new())),
        };

        assert_eq!(
            manager
                .get_exchange_rate(Currency::USD, Currency::EUR)
                .unwrap(),
            0.85
        );
        assert_eq!(
            manager
                .exchange_rate_cache
                .lock()
                .unwrap()
                .get(&(Currency::USD, Currency::EUR))
                .unwrap(),
            &0.85
        );
    }

    #[test]
    fn test_nonexistent_rate() {
        let ref_date = Date::new(2021, 1, 1);
        let manager: ExchangeRateStore = ExchangeRateStore {
            volatility_map: HashMap::new(),
            reference_date: ref_date,
            exchange_rate_map: HashMap::new(),
            exchange_rate_cache: Arc::new(Mutex::new(HashMap::new())),
        };

        let result = manager.get_exchange_rate(Currency::USD, Currency::EUR);
        assert!(result.is_err());
    }

    #[test]
    fn test_complex_case() {
        let ref_date = Date::new(2021, 1, 1);
        let manager = ExchangeRateStore {
            reference_date: ref_date,
            volatility_map: HashMap::new(),
            exchange_rate_map: {
                let mut map = HashMap::new();
                map.insert((Currency::USD, Currency::EUR), 0.85);
                map.insert((Currency::EUR, Currency::USD), 1.0 / 0.85);
                map
            },
            exchange_rate_cache: Arc::new(Mutex::new(HashMap::new())),
        };

        assert_eq!(
            manager
                .get_exchange_rate(Currency::EUR, Currency::USD)
                .unwrap(),
            1.0 / 0.85
        );
        assert_eq!(
            manager
                .get_exchange_rate(Currency::USD, Currency::EUR)
                .unwrap(),
            0.85
        );
    }

    #[test]
    fn test_triangulation_case() {
        let ref_date = Date::new(2021, 1, 1);
        let mut manager = ExchangeRateStore::new(ref_date);
        manager.add_exchange_rate(Currency::CLP, Currency::USD, 800.0);
        manager.add_exchange_rate(Currency::USD, Currency::EUR, 1.1);

        assert_eq!(
            manager
                .get_exchange_rate(Currency::CLP, Currency::EUR)
                .unwrap(),
            1.1 * 800.0
        );
        assert_eq!(
            manager
                .get_exchange_rate(Currency::EUR, Currency::CLP)
                .unwrap(),
            1.0 / (1.1 * 800.0)
        );
    }
}
