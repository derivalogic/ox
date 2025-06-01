use std::collections::HashMap;

use crate::math::ad::genericnumber::Real;
use crate::time::date::Date;
use crate::utils::errors::{AtlasError, Result};

/// Store for asset volatilities. Currently maps currency pairs to constant volatilities.
#[derive(Clone)]
pub struct EquityStore<T: Real> {
    reference_date: Date,
    volatility_map: HashMap<String, T>,
}

impl<T: Real> EquityStore<T> {
    pub fn new(reference_date: Date) -> Self {
        Self {
            reference_date,
            volatility_map: HashMap::new(),
        }
    }

    pub fn reference_date(&self) -> Date {
        self.reference_date
    }

    pub fn add_volatility(&mut self, equity_name: String, vol: T) {
        self.volatility_map.insert(equity_name, vol);
    }

    pub fn get_volatility(&self, equity_name: String) -> Result<T> {
        self.volatility_map
            .get(&equity_name)
            .cloned()
            .ok_or_else(|| {
                AtlasError::ValueNotSetErr(format!("Volatility for {} not set", equity_name))
            })
    }

    pub fn get_volatility_map(&self) -> &HashMap<String, T> {
        &self.volatility_map
    }
}
