// use core::panic;
// use rustatlas::prelude::*;
// use scripting::data::termstructure::{TermStructure, TermStructureKey, TermStructureType};
// use scripting::models::scriptingmodel::{BlackScholesModel, MonteCarloEngine};
// use scripting::prelude::*;
// use scripting::utils::errors::Result;
// use scripting::visitors::evaluator::{Evaluator, Value};
// use scripting::visitors::fuzzy_evaluator;
// use std::collections::HashMap;

// fn market_data(reference_date: Date) -> HistoricalData {
//     let mut store = HistoricalData::new();
//     store.mut_exchange_rates().add_exchange_rate(
//         reference_date,
//         Currency::CLP,
//         Currency::USD,
//         100.0,
//     );

//     store.mut_exchange_rates().add_exchange_rate(
//         reference_date,
//         Currency::JPY,
//         Currency::USD,
//         142.74,
//     );

//     store.mut_exchange_rates().add_exchange_rate(
//         reference_date,
//         Currency::EUR,
//         Currency::USD,
//         0.876,
//     );

//     store
//         .mut_volatilities()
//         .add_fx_volatility(reference_date, Currency::EUR, Currency::USD, 0.2);

//     store
//         .mut_volatilities()
//         .add_fx_volatility(reference_date, Currency::CLP, Currency::USD, 0.2);

//     store
//         .mut_volatilities()
//         .add_fx_volatility(reference_date, Currency::JPY, Currency::USD, 0.2);

//     // general
//     let year_fractions = vec![1.0];
//     let interpolator = Interpolator::Linear;
//     let enable_extrapolation = true;
//     let rate_definition = RateDefinition::default();
//     let term_structure_type = TermStructureType::FlatForward;

//     // CLP term structure
//     let clp_ts_key = TermStructureKey::new(Currency::CLP, true, Some("CLP".to_string()));
//     let clp_rate = vec![0.0];

//     let clp_ts = TermStructure::new(
//         clp_ts_key,
//         year_fractions.clone(),
//         clp_rate,
//         interpolator,
//         enable_extrapolation,
//         rate_definition,
//         term_structure_type,
//     );

//     // USD term structure
//     let usd_ts_key = TermStructureKey::new(Currency::USD, true, Some("USD".to_string()));
//     let usd_rate = vec![0.0];

//     let usd_ts = TermStructure::new(
//         usd_ts_key,
//         year_fractions.clone(),
//         usd_rate,
//         interpolator,
//         enable_extrapolation,
//         rate_definition,
//         term_structure_type,
//     );

//     // USD term structure
//     let eur_ts_key = TermStructureKey::new(Currency::EUR, true, Some("EUR".to_string()));
//     let eur_rate = vec![0.027];

//     let eur_ts = TermStructure::new(
//         eur_ts_key,
//         year_fractions.clone(),
//         eur_rate,
//         interpolator,
//         enable_extrapolation,
//         rate_definition,
//         term_structure_type,
//     );

//     // JPY term structure
//     let jpy_ts_key = TermStructureKey::new(Currency::JPY, true, Some("JPY".to_string()));
//     let jpy_rate = vec![0.027];

//     let jpy_ts = TermStructure::new(
//         jpy_ts_key,
//         year_fractions.clone(),
//         jpy_rate,
//         interpolator,
//         enable_extrapolation,
//         rate_definition,
//         term_structure_type,
//     );

//     store
//         .mut_term_structures()
//         .add_term_structure(reference_date, clp_ts);
//     store
//         .mut_term_structures()
//         .add_term_structure(reference_date, usd_ts);

//     store
//         .mut_term_structures()
//         .add_term_structure(reference_date, eur_ts);

//     store
//         .mut_term_structures()
//         .add_term_structure(reference_date, jpy_ts);

//     store
// }

// fn main() -> Result<()> {
//     Tape::start_recording();
//     let reference_date = Date::new(2025, 6, 1);
//     let store = market_data(reference_date);
//     let mut model = BlackScholesModel::new(reference_date, Currency::USD, &store);
//     model.initialize()?;

//     let time_handle = model.time_handle();
//     // Scripted payoff of a call option

//     let e1 = Date::new(2026, 6, 1);
//     let s1 = "
//     opt = 0;
//     s = Spot(\"CLP\",\"USD\");

//     if s > 100 {
//         opt pays 1;
//     } else {
//         opt pays 0;
//     }
//     ";

//     // Build the event stream
//     let coded1 = CodedEvent::new(e1, s1.to_string());
//     let events = EventStream::try_from(vec![coded1])?;

//     // Visit the events to index variables and prepare for evaluation
//     let indexer = EventIndexer::new().with_local_currency(model.local_currency());
//     indexer.visit_events(&events)?;
//     let requests = indexer.get_request();
//     let scenarios = model.generate_scenarios(events.event_dates(), &requests, 100_000)?;

//     // Evaluate the script under all scenarios
//     let var_map = indexer.get_variable_indexes();
//     let evaluator = Evaluator::new(indexer.get_variables_size(), &scenarios);
//     let vars = evaluator.visit_events(&events, &var_map)?;
//     let price_mc = match vars.get("opt") {
//         Some(Value::Number(v)) => *v,
//         _ => panic!("Option price not found in the evaluated variables"),
//     };

//     price_mc.backward()?;
//     println!("Monte Carlo Price: {}", price_mc);
//     println!("Theta: {}", time_handle.adjoint()? * 1.0 / 360.0);

//     let deltas = model
//         .fx()
//         .iter()
//         .map(|(pair, rate)| {
//             (
//                 format!("{}/{}", pair.0.code(), pair.1.code()),
//                 rate.read().unwrap().adjoint().unwrap_or(0.0),
//             )
//         })
//         .collect::<HashMap<_, _>>();

//     let rhos = model
//         .rates()
//         .iter()
//         .map(|c| {
//             (
//                 c.key().name().unwrap().clone(),
//                 c.values()
//                     .get(0)
//                     .unwrap()
//                     .read()
//                     .unwrap()
//                     .adjoint()
//                     .unwrap_or(0.0)
//                     * 0.01,
//             )
//         })
//         .collect::<HashMap<_, _>>();

//     println!("Deltas: {:?}", deltas);
//     println!("Rhos: {:?}", rhos);

//     let event_maturity = Date::new(2026, 6, 1);
//     let script2 = "
//     opt = 0;
//     s = Spot(\"CLP\",\"USD\");
//     fif(s > 100, opt pays 1, opt pays 0, 1);    
//     ";

//     // Build the event stream
//     let coded2 = CodedEvent::new(event_maturity, script2.to_string());
//     let events2 = EventStream::try_from(vec![coded2])?;
//     // Visit the events to index variables and prepare for evaluation
//     let indexer2 = EventIndexer::new().with_local_currency(model.local_currency());
//     indexer2.visit_events(&events2)?;
//     let requests2 = indexer2.get_request();
//     let scenarios2 = model.generate_scenarios(events2.event_dates(), &requests2, 100_000)?;
//     // Evaluate the script under all scenarios
//     let var_map2 = indexer2.get_variable_indexes();
//     let evaluator2 = Evaluator::new(indexer2.get_variables_size(), &scenarios2);
//     let vars2 = evaluator2.visit_events(&events2, &var_map2)?;
//     let price_mc2 = match vars2.get("opt") {
//         Some(Value::Number(v)) => *v,
//         _ => panic!("Option price not found in the evaluated variables"),
//     };
//     price_mc2.backward()?;
//     println!("Monte Carlo Price with IF: {}", price_mc2);
//     println!("Theta with IF: {}", time_handle.adjoint()? * 1.0 / 360.0);

//     Ok(())
// }
