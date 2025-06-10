use core::panic;
use rustatlas::prelude::*;
use scripting::prelude::*;
use std::collections::HashMap;

fn market_data(reference_date: Date) -> HistoricalData {
    let mut store = HistoricalData::new();
    store.mut_exchange_rates().add_exchange_rate(
        reference_date,
        Currency::CLP,
        Currency::USD,
        936.405795,
    );

    store.mut_exchange_rates().add_exchange_rate(
        reference_date,
        Currency::JPY,
        Currency::USD,
        142.74,
    );

    store.mut_exchange_rates().add_exchange_rate(
        reference_date,
        Currency::EUR,
        Currency::USD,
        0.876,
    );

    store
        .mut_volatilities()
        .add_fx_volatility(reference_date, Currency::EUR, Currency::USD, 0.2);

    store
        .mut_volatilities()
        .add_fx_volatility(reference_date, Currency::CLP, Currency::USD, 0.2);

    store
        .mut_volatilities()
        .add_fx_volatility(reference_date, Currency::JPY, Currency::USD, 0.2);

    // general
    let year_fractions = vec![1.0];
    let interpolator = Interpolator::Linear;
    let enable_extrapolation = true;
    let rate_definition = RateDefinition::default();
    let term_structure_type = TermStructureType::FlatForward;

    // CLP term structure
    let clp_ts_key = TermStructureKey::new(Currency::CLP, true, Some("CLP".to_string()));
    let clp_rate = vec![0.046];

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
    let usd_rate = vec![0.036];

    let usd_ts = TermStructure::new(
        usd_ts_key,
        year_fractions.clone(),
        usd_rate,
        interpolator,
        enable_extrapolation,
        rate_definition,
        term_structure_type,
    );

    // USD term structure
    let eur_ts_key = TermStructureKey::new(Currency::EUR, true, Some("EUR".to_string()));
    let eur_rate = vec![0.027];

    let eur_ts = TermStructure::new(
        eur_ts_key,
        year_fractions.clone(),
        eur_rate,
        interpolator,
        enable_extrapolation,
        rate_definition,
        term_structure_type,
    );

    // JPY term structure
    let jpy_ts_key = TermStructureKey::new(Currency::JPY, true, Some("JPY".to_string()));
    let jpy_rate = vec![0.027];

    let jpy_ts = TermStructure::new(
        jpy_ts_key,
        year_fractions.clone(),
        jpy_rate,
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
        .mut_term_structures()
        .add_term_structure(reference_date, eur_ts);

    store
        .mut_term_structures()
        .add_term_structure(reference_date, jpy_ts);

    store
}

fn main() -> Result<()> {
    Tape::start_recording();
    let reference_date = Date::new(2025, 6, 10);
    let store = market_data(reference_date);
    let mut model = BlackScholesModel::new(reference_date, Currency::USD, &store);
    model.initialize()?;

    let time_handle = model.time_handle();
    // Scripted payoff of a call option
    let event_maturity = Date::new(2025, 7, 10);
    let script = "opt = 0;\ns = Spot(\"CLP\",\"USD\");\nopt pays s in \"CLP\";";

    // Build the event stream
    let coded = CodedEvent::new(event_maturity, script.to_string());
    let mut events = EventStream::try_from(vec![coded])?;

    // Visit the events to index variables and prepare for evaluation
    let indexer = VarIndexer::new().with_local_currency(model.local_currency());
    indexer.visit_events(&mut events)?;
    let requests = indexer.get_request();

    let scenarios = model.generate_scenarios(events.event_dates(), &requests, 1000)?;

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
    println!("Theta: {}", time_handle.adjoint()? * 1.0 / 360.0);

    let deltas = model
        .fx()
        .iter()
        .map(|(pair, rate)| {
            (
                format!("{}/{}", pair.0.code(), pair.1.code()),
                rate.read().unwrap().adjoint().unwrap_or(0.0),
            )
        })
        .collect::<HashMap<_, _>>();

    let rhos = model
        .rates()
        .iter()
        .map(|c| {
            (
                c.key().name().unwrap().clone(),
                c.values()
                    .get(0)
                    .unwrap()
                    .read()
                    .unwrap()
                    .adjoint()
                    .unwrap_or(0.0)
                    * 0.01,
            )
        })
        .collect::<HashMap<_, _>>();

    println!("Deltas: {:?}", deltas);
    println!("Rhos: {:?}", rhos);

    Ok(())
}
