use std::collections::HashMap;

use crate::currencies::enums::Currency;
use crate::math::ad::num::Real;
use crate::time::{date::Date, period::Period};
use crate::utils::errors::{AtlasError, Result};

/// Store for asset volatilities. Currently maps currency pairs to constant volatilities.
#[derive(Clone)]
pub struct EquityStore<T: Real> {
    reference_date: Date,
    vol_map: HashMap<(Currency, Currency), T>,
}

impl<T: Real> EquityStore<T> {
    pub fn new(reference_date: Date) -> Self {
        Self {
            reference_date,
            vol_map: HashMap::new(),
        }
    }

    pub fn reference_date(&self) -> Date {
        self.reference_date
    }

    pub fn add_volatility(&mut self, ccy1: Currency, ccy2: Currency, vol: T) {
        self.vol_map.insert((ccy1, ccy2), vol);
    }

    pub fn volatility(&self, ccy1: Currency, ccy2: Currency) -> Result<T> {
        if let Some(v) = self.vol_map.get(&(ccy1, ccy2)) {
            Ok(*v)
        } else if let Some(v) = self.vol_map.get(&(ccy2, ccy1)) {
            Ok(*v)
        } else {
            Err(AtlasError::NotFoundErr(format!(
                "No volatility for pair {:?}/{:?}",
                ccy1, ccy2
            )))
        }
    }

    pub fn vol_map(&self) -> HashMap<(Currency, Currency), T> {
        self.vol_map.clone()
    }
}

pub trait AdvanceEquityStoreInTime<T: Real> {
    fn advance_to_period(&self, period: Period) -> Result<EquityStore<T>>;
    fn advance_to_date(&self, date: Date) -> Result<EquityStore<T>>;
}

impl<T: Real> AdvanceEquityStoreInTime<T> for EquityStore<T> {
    fn advance_to_period(&self, period: Period) -> Result<EquityStore<T>> {
        let date = self.reference_date + period;
        self.advance_to_date(date)
    }

    fn advance_to_date(&self, date: Date) -> Result<EquityStore<T>> {
        Ok(EquityStore {
            reference_date: date,
            vol_map: self.vol_map.clone(),
        })
    }
}
