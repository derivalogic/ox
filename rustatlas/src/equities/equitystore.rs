use std::collections::HashMap;

use crate::prelude::*;

/// Store for asset volatilities. Currently maps currency pairs to constant volatilities.
#[derive(Clone)]
pub struct EquityStore {
    reference_date: Date,
    volatility_map: HashMap<String, NumericType>,
}

impl EquityStore {
    pub fn new(reference_date: Date) -> Self {
        Self {
            reference_date,
            volatility_map: HashMap::new(),
        }
    }

    pub fn reference_date(&self) -> Date {
        self.reference_date
    }

    pub fn add_volatility(&mut self, equity_name: String, vol: NumericType) {
        self.volatility_map.insert(equity_name, vol);
    }

    pub fn get_volatility(&self, equity_name: String) -> Result<NumericType> {
        self.volatility_map
            .get(&equity_name)
            .cloned()
            .ok_or_else(|| {
                AtlasError::ValueNotSetErr(format!("Volatility for {} not set", equity_name))
            })
    }

    pub fn get_volatility_map(&self) -> &HashMap<String, NumericType> {
        &self.volatility_map
    }
}
