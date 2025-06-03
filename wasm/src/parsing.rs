use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

use rustatlas::prelude::*;
use scripting::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct Curve {
    pub name: String,
    pub currency: Currency,
    pub rate: NumericType,
}

#[derive(Debug, Deserialize)]
pub struct CurrencyParity {
    pub weak: Currency,
    pub strong: Currency,
    pub value: NumericType,
    pub vol: NumericType,
}

#[derive(Debug, Deserialize)]
pub struct MarketData {
    pub reference_date: Date,
    pub local_currency: Currency,
    pub curves: Vec<Curve>,
    pub fx: Vec<CurrencyParity>,
}

#[derive(Debug, Deserialize)]
pub struct ScriptData {
    pub events: Vec<CodedEvent>,
}

#[derive(Debug, Deserialize)]
pub struct SimulationData {
    pub market_data: MarketData,
    pub script_data: ScriptData,
}

#[derive(Debug, Serialize)]
pub struct SimulationResults {
    pub target_value: f64,
    pub theta: f64,
    pub deltas: Vec<HashMap<String, f64>>, // sensitivies to fx as "c1/cc2" from the exchange rate store
    pub rhos: Vec<HashMap<String, f64>>,   // sensitivies to rates as "c1/c2" from the index store
}

pub fn create_market_store(data: &MarketData) -> MarketStore {
    let mut store = MarketStore::new(data.reference_date, data.local_currency);

    // Add local currency
    data.fx.iter().for_each(|parity| {
        store
            .mut_exchange_rate_store()
            .add_exchange_rate(parity.weak, parity.strong, parity.value);
        store
            .mut_exchange_rate_store()
            .add_volatility(parity.weak, parity.strong, parity.vol);
    });

    // Add curves
    data.curves.iter().enumerate().for_each(|(i, curve)| {
        let term_structure = Arc::new(FlatForwardTermStructure::new(
            data.reference_date,
            curve.rate,
            RateDefinition::default(),
        ));
        let index = Arc::new(RwLock::new(
            OvernightIndex::new(data.reference_date)
                .with_name(Some(curve.name.clone()))
                .with_rate_definition(RateDefinition::default())
                .with_term_structure(term_structure),
        ));
        store.mut_index_store().add_index(i, index).unwrap();
        store
            .mut_index_store()
            .add_currency_curve(curve.currency, i);
    });
    store
}
