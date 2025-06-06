use core::panic;
use rustatlas::prelude::*;
use scripting::data::termstructure::{TermStructure, TermStructureKey, TermStructureType};
use scripting::models::scriptingmodel::{BlackScholesModel, MonteCarloEngine};
use scripting::prelude::*;
use scripting::utils::errors::Result;
use scripting::visitors::evaluator::{Evaluator, Value};

fn market_data(reference_date: Date) -> HistoricalData {
    let mut store = HistoricalData::new();
    store.mut_exchange_rates().add_exchange_rate(
        reference_date,
        Currency::CLP,
        Currency::USD,
        800.0,
    );

    store
        .mut_volatilities()
        .add_fx_volatility(reference_date, Currency::USD, Currency::CLP, 0.2);

    store
        .mut_volatilities()
        .add_fx_volatility(reference_date, Currency::CLP, Currency::USD, 0.2);

    // general
    let year_fractions = vec![1.0];
    let interpolator = Interpolator::Linear;
    let enable_extrapolation = true;
    let rate_definition = RateDefinition::default();
    let term_structure_type = TermStructureType::FlatForward;

    // CLP term structure
    let clp_ts_key = TermStructureKey::new(Currency::CLP, true, Some("CLP".to_string()));
    let clp_rate = vec![0.03];

    let clp_ts = TermStructure::new(
        clp_ts_key,
        year_fractions.clone(),
        clp_rate,
        interpolator,
        enable_extrapolation,
        rate_definition,
        term_structure_type,
    );

    // USD term structure
    let usd_ts_key = TermStructureKey::new(Currency::USD, true, Some("USD".to_string()));
    let usd_rate = vec![0.02];

    let usd_ts = TermStructure::new(
        usd_ts_key,
        year_fractions.clone(),
        usd_rate,
        interpolator,
        enable_extrapolation,
        rate_definition,
        term_structure_type,
    );

    store
        .mut_term_structures()
        .add_term_structure(reference_date, clp_ts);
    store
        .mut_term_structures()
        .add_term_structure(reference_date, usd_ts);

    store
}

fn main() -> Result<()> {
    let reference_date = Date::new(2025, 1, 1);
    Tape::start_recording();
    let store = market_data(reference_date);
    let mut model = BlackScholesModel::new(reference_date, Currency::CLP, &store);
    model.initialize()?;
    // model.initialize_for_parallelization();

    let s0 = model
        .fx()
        .get(&(Currency::CLP, Currency::USD))
        .ok_or(ScriptingError::InvalidOperation(
            "Spot rate not found".to_string(),
        ))?
        .read()
        .unwrap();

    let binding = model
        .rates()
        .get_by_currency(Currency::CLP)
        .unwrap()
        .nodes();

    let r_clp = binding.get(0).unwrap().1.read().unwrap();

    let binding = model
        .rates()
        .get_by_currency(Currency::USD)
        .unwrap()
        .nodes();
    let r_usd = binding.get(0).unwrap().1.read().unwrap();

    // Scripted payoff of a call option
    let em1 = Date::new(2026, 1, 1);
    let es1 = "opt = 0;s = Spot(\"CLP\", \"USD\");call = max(s-800,0);opt pays call;";

    let em2 = Date::new(2027, 1, 1);
    let es2 = "s = Spot(\"CLP\", \"USD\");call = max(s-800,0);opt pays call;";

    // Build the event stream
    let e1 = CodedEvent::new(em1, es1.to_string());
    let e2 = CodedEvent::new(em2, es2.to_string());
    let events = EventStream::try_from(vec![e1, e2])?;

    // Visit the events to index variables and prepare for evaluation
    let indexer = EventIndexer::new().with_local_currency(model.local_currency());
    indexer.visit_events(&events)?;
    let requests = indexer.get_request();

    let scenarios = model.generate_scenarios(events.event_dates(), &requests, 100_000)?;

    // Evaluate the script under all scenarios
    let var_map = indexer.get_variable_indexes();
    let evaluator = Evaluator::new(indexer.get_variables_size(), &scenarios);
    let vars = evaluator.visit_events(&events, &var_map)?;
    let price_mc = match vars.get("opt") {
        Some(Value::Number(v)) => *v,
        _ => panic!("Option price not found in the evaluated variables"),
    };

    // price_mc.propagate_to_start().unwrap();
    price_mc.backward()?;
    println!("Monte Carlo Price: {}", price_mc);
    println!("Monte Carlo Delta: {}", s0.adjoint()?);
    println!("Monte Carlo Rho CLP: {}", r_clp.adjoint()?);
    println!("Monte Carlo Rho USD: {}", r_usd.adjoint()?);
    Ok(())
}
