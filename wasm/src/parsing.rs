use std::collections::HashMap;

use rustatlas::prelude::*;
use scripting::{
    data::termstructure::{TermStructure, TermStructureKey, TermStructureType},
    prelude::*,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct Curve {
    pub name: String,
    pub currency: Currency,
    pub rate: f64,
}

#[derive(Debug, Deserialize)]
pub struct CurrencyParity {
    pub reference_date: Date,
    pub weak: Currency,
    pub strong: Currency,
    pub value: f64,
    pub vol: f64,
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

pub fn create_historical_data(data: &MarketData) -> HistoricalData {
    let mut store = HistoricalData::new();

    // Add exchange rates
    data.fx.iter().for_each(|parity| {
        store.mut_exchange_rates().add_exchange_rate(
            parity.reference_date,
            parity.weak,
            parity.strong,
            parity.value,
        );
        store.mut_volatilities().add_fx_volatility(
            parity.reference_date,
            parity.weak,
            parity.strong,
            parity.vol,
        );
    });

    // Add term structures
    let year_fractions = vec![0.0, 20.0];
    let interpolator = Interpolator::Linear;
    let enable_extrapolation = true;
    let rate_definition = RateDefinition::default();
    let term_structure_type = TermStructureType::FlatForward;

    data.curves.iter().for_each(|curve| {
        let ts_key = TermStructureKey::new(curve.currency, true, Some(curve.name.clone()));
        let ts = TermStructure::new(
            ts_key,
            year_fractions.clone(),
            vec![curve.rate, curve.rate],
            interpolator,
            enable_extrapolation,
            rate_definition.clone(),
            term_structure_type.clone(),
        );
        store
            .mut_term_structures()
            .add_term_structure(data.reference_date, ts);
    });

    store
}
