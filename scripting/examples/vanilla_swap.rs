use std::sync::{Arc, RwLock};

use lefi::prelude::*;
use lefi::utils::errors::Result;
use rustatlas::core::marketstore::MarketStore;
use rustatlas::currencies::enums::Currency;
use rustatlas::models::stochasticvol::RiskFreeMonteCarloModel;
use rustatlas::models::traits::MonteCarloModel;
use rustatlas::prelude::{FlatForwardTermStructure, OvernightIndex, RateDefinition};
use rustatlas::time::date::Date;

fn create_market_store() -> MarketStore<f64> {
    let ref_date = Date::new(2024, 1, 1);
    let mut store = MarketStore::new(ref_date, Currency::USD);
    store
        .mut_exchange_rate_store()
        .add_exchange_rate(Currency::CLP, Currency::USD, 850.0);

    let clp_curve = Arc::new(FlatForwardTermStructure::new(
        ref_date,
        0.05,
        RateDefinition::default(),
    ));
    let clp_index = Arc::new(RwLock::new(
        OvernightIndex::new(ref_date).with_term_structure(clp_curve),
    ));
    let _ = store.mut_index_store().add_index(0, clp_index);
    store.mut_index_store().add_currency_curve(Currency::CLP, 0);

    let usd_curve = Arc::new(FlatForwardTermStructure::new(
        ref_date,
        0.03,
        RateDefinition::default(),
    ));
    let usd_index = Arc::new(RwLock::new(
        OvernightIndex::new(ref_date).with_term_structure(usd_curve),
    ));
    let _ = store.mut_index_store().add_index(1, usd_index);
    store.mut_index_store().add_currency_curve(Currency::USD, 1);
    store
}

fn main() -> Result<()> {
    let fixed_rate = 0.05;
    let notional = 1_000_000.0;

    let script1 = format!(
        "notional = {notional};\n    r = RateIndex(\"0\", \"2024-01-01\", \"2024-07-01\");\n    c = cvg(\"2024-01-01\", \"2024-07-01\", \"Actual360\");\n    pays notional * (r - {fixed_rate}) * c;"
    );

    let script2 = format!(
        "notional = {notional};\n    r = RateIndex(\"0\", \"2024-07-01\", \"2025-01-01\");\n    c = cvg(\"2024-07-01\", \"2025-01-01\", \"Actual360\");\n    pays notional * (r - {fixed_rate}) * c;"
    );

    let events = EventStream::try_from(vec![
        CodedEvent::new(Date::new(2024, 7, 1), script1),
        CodedEvent::new(Date::new(2025, 1, 1), script2),
    ])?;

    let indexer = EventIndexer::new().with_local_currency(Currency::USD);
    indexer.visit_events(&events)?;

    let store = create_market_store();
    let model = RiskFreeMonteCarloModel::new(&store);
    let requests = indexer.get_market_requests();
    let scenarios = model.gen_scenarios(&requests, 10)?;

    let var_map = indexer.get_variable_indexes();
    let evaluator = EventStreamEvaluator::new(indexer.get_variables_size())
        .with_scenarios(&scenarios);
    let vars = evaluator.visit_events(&events, &var_map)?;

    println!("Swap NPV: {:?}", vars);
    Ok(())
}
