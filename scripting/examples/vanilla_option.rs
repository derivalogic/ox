use core::panic;
use rustatlas::prelude::*;
use scripting::prelude::*;
use scripting::utils::errors::Result;
use std::sync::{Arc, RwLock};

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
    // Model parameters
    let s0 = NumericType::new(850.0);
    let r_usd = NumericType::new(0.03);
    let r_clp = NumericType::new(0.05);

    // Scripted payoff of a call option
    let maturity = Date::new(2025, 1, 1);
    let script = "
    opt = 0;
    s = Spot(\"CLP\", \"USD\");
    call = max(s - 900.0, 0);
    opt pays call;
    ";

    // Build the event stream
    let coded = CodedEvent::new(maturity, script.to_string());
    let events = EventStream::try_from(vec![coded])?;
    let indexer = EventIndexer::new().with_local_currency(Currency::USD);

    indexer.visit_events(&events)?;
    let requests = indexer.get_market_requests();

    // Monte Carlo scenarios with Black-Scholes dynamics using AD variables

    let store = create_market_store(s0, r_usd, r_clp);
    let vol = store
        .get_exchange_rate_volatility(Currency::CLP, Currency::USD)
        .unwrap();
    let simple = SimpleModel::new(&store);
    let model = BlackScholesModel::new(simple);

    let sims = 10_000;
    let scenarios = (0..sims)
        .into_iter()
        .map(|_| model.gen_scenario(&requests).map_err(|e| e.into()))
        .collect::<Result<Vec<Scenario>>>()?;

    // // Evaluate the script under all scenarios
    let var_map = indexer.get_variable_indexes();
    let evaluator: EventStreamEvaluator =
        EventStreamEvaluator::new(indexer.get_variables_size()).with_scenarios(&scenarios);
    let vars = evaluator.visit_events(&events, &var_map)?;
    let price_mc = match vars.get("opt") {
        Some(Value::Number(v)) => *v,
        _ => panic!("Option price not found in the evaluated variables"),
    };

    price_mc.propagate_to_start().unwrap();

    println!("Monte Carlo Price: {}", price_mc);
    println!("Monte Carlo Delta: {}", s0.adjoint().unwrap());
    println!(
        "Monte Carlo Rho CLP: {}",
        r_clp.adjoint().unwrap() * 0.01 / 100.0
    );
    println!(
        "Monte Carlo Rho USD: {}",
        r_usd.adjoint().unwrap() * 0.01 / 100.0
    );
    println!(
        "Monte Carlo Vega: {}",
        vol.adjoint().unwrap() * 0.01 / 100.0
    );

    Ok(())
}
