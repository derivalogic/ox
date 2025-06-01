use std::sync::{Arc, RwLock};

use lefi::prelude::*;
use lefi::utils::errors::Result;
use rustatlas::prelude::*;

fn create_market_store(s0: NumericType, r_usd: NumericType, r_clp: NumericType) -> MarketStore {
    let ref_date = Date::new(2024, 1, 1);
    let mut store = MarketStore::new(ref_date, Currency::USD);
    store
        .mut_exchange_rate_store()
        .add_exchange_rate(Currency::CLP, Currency::USD, s0);
    let usd_curve = Arc::new(FlatForwardTermStructure::new(
        ref_date,
        r_usd,
        RateDefinition::default(),
    ));
    let index = Arc::new(RwLock::new(
        OvernightIndex::new(ref_date).with_term_structure(usd_curve),
    ));
    let _ = store.mut_index_store().add_index(0, index);
    store.mut_index_store().add_currency_curve(Currency::USD, 0);

    let clp_curve = Arc::new(FlatForwardTermStructure::new(
        ref_date,
        r_clp,
        RateDefinition::default(),
    ));
    let index_clp = Arc::new(RwLock::new(
        OvernightIndex::new(ref_date).with_term_structure(clp_curve),
    ));
    let _ = store.mut_index_store().add_index(1, index_clp);
    store.mut_index_store().add_currency_curve(Currency::CLP, 1);

    // add volatility
    store.mut_exchange_rate_store().add_volatility(
        Currency::CLP,
        Currency::USD,
        NumericType::new(0.2),
    );
    store
}

fn main() -> Result<()> {
    let s0 = NumericType::new(850.0);
    let r_usd = NumericType::new(0.03);
    let r_clp = NumericType::new(0.05);

    let obs1 = Date::new(2024, 6, 1);
    let obs2 = Date::new(2024, 12, 1);
    let maturity = Date::new(2025, 1, 1);

    let script1 = "
    opt = 0;
    s1 = Spot(\"CLP\", \"USD\");
    ";
    let script2 = "s2 = Spot(\"CLP\", \"USD\");";
    let script_payoff = "
    avg = (s1 + s2) / 2.0;
    opt pays max(avg - 900, 0);
    ";

    let events = EventStream::try_from(vec![
        CodedEvent::new(obs1, script1.to_string()),
        CodedEvent::new(obs2, script2.to_string()),
        CodedEvent::new(maturity, script_payoff.to_string()),
    ])?;

    let indexer = EventIndexer::new().with_local_currency(Currency::USD);
    indexer.visit_events(&events)?;

    let store = create_market_store(s0, r_usd, r_clp);

    let simple = SimpleModel::new(&store);
    let model = BlackScholesModel::new(simple);
    let requests = indexer.get_market_requests();
    let sims = 10_000;
    let scenarios = (0..sims)
        .into_iter()
        .map(|_| model.gen_scenario(&requests).map_err(|e| e.into()))
        .collect::<Result<Vec<Scenario>>>()?;

    let var_map = indexer.get_variable_indexes();
    let evaluator =
        EventStreamEvaluator::new(indexer.get_variables_size()).with_scenarios(&scenarios);
    let vars = evaluator.visit_events(&events, &var_map)?;

    println!("Asian option price: {:?}", vars);
    Ok(())
}
