use lefi::prelude::*;
use lefi::utils::errors::Result;
use rustatlas::currencies::enums::Currency;
use rustatlas::core::marketstore::MarketStore;
use rustatlas::models::montecarlo::RiskFreeMonteCarloModel;
use rustatlas::models::traits::MonteCarloModel;
use rustatlas::time::date::Date;

fn create_market_store() -> MarketStore {
    let ref_date = Date::new(2024, 1, 1);
    let mut store = MarketStore::new(ref_date, Currency::USD);
    store
        .mut_exchange_rate_store()
        .add_exchange_rate(Currency::CLP, Currency::USD, 850.0);
    store
}

fn main() -> Result<()> {
    // European call option on CLP/USD with strike 900 and maturity in one year
    let maturity = Date::new(2025, 1, 1);
    let script = "call = pays max(spot(\"CLP\", \"USD\") - 900.0, 0);";

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
    let evaluator =
        EventStreamEvaluator::new(indexer.get_variables_size()).with_scenarios(&scenarios);
    let vars = evaluator.visit_events(&events)?;

    println!("Call price: {:?}", vars);
    Ok(())
}
