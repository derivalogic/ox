use std::sync::{Arc, RwLock};

use lefi::prelude::*;
use lefi::utils::errors::Result;
use rustatlas::core::marketstore::MarketStore;
use rustatlas::currencies::enums::Currency;
use rustatlas::models::montecarlo::RiskFreeMonteCarloModel;
use rustatlas::models::traits::MonteCarloModel;
use rustatlas::prelude::{FlatForwardTermStructure, OvernightIndex, RateDefinition};
use rustatlas::time::date::Date;

fn create_market_store() -> MarketStore {
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
    // European call option on CLP/USD with strike 900 and maturity in one year
    let maturity = Date::new(2025, 1, 1);
    let script = "
    opt = 0;
    s = Spot(\"CLP\", \"USD\");
    call =  max(s - 900.0, 0); 
    opt pays call;
    ";

    // Create event stream from the scripted payoff
    let coded = CodedEvent::new(maturity, script.to_string());
    let events = EventStream::try_from(vec![coded])?;

    // Index variables and market data requests
    let indexer = EventIndexer::new().with_local_currency(Currency::USD);
    indexer.visit_events(&events)?;

    // Generate Monte Carlo scenarios for required market data
    let store = create_market_store();
    let model = RiskFreeMonteCarloModel::new(&store);
    let requests = indexer.get_market_requests();
    let scenarios_var = model.gen_scenarios(&requests, 10)?;
    let scenarios = RiskFreeMonteCarloModel::scenarios_to_f64(scenarios_var);

    // Evaluate the script under all scenarios and average the result
    let var_map = indexer.get_variable_indexes();
    let evaluator =
        EventStreamEvaluator::new(indexer.get_variables_size()).with_scenarios(&scenarios);
    let vars = evaluator.visit_events(&events, &var_map)?;

    println!("Call price: {:?}", vars);
    Ok(())
}
