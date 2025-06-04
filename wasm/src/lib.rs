mod parsing;
mod utils;
use parsing::{SimulationData, SimulationResults};
use rustatlas::prelude::*;
use scripting::models::scriptingmodel::{BlackScholesModel, MonteCarloEngine};
use scripting::prelude::*;
use scripting::visitors::evaluator::{Evaluator, Value};
use serde_json;
use std::collections::HashMap;
use std::result::Result as StdResult;
use wasm_bindgen::prelude::*;

use crate::parsing::create_historical_data;

#[wasm_bindgen]
pub fn run_simulation(json: &str) -> StdResult<JsValue, JsValue> {
    utils::set_panic_hook();
    Tape::start_recording();
    let data: SimulationData =
        serde_json::from_str(json).map_err(|e| JsValue::from_str(&e.to_string()))?;

    let store: HistoricalData = create_historical_data(&data.market_data);

    let events = EventStream::try_from(data.script_data.events.clone())
        .map_err(|e| JsValue::from_str(&e.to_string()))?;

    let indexer = EventIndexer::new();
    indexer
        .visit_events(&events)
        .map_err(|e| JsValue::from_str(&e.to_string()))?;

    let requests = indexer.get_request();

    Tape::start_recording();
    let mut model = BlackScholesModel::new(
        data.market_data.reference_date,
        data.market_data.local_currency,
        &store,
    );

    model
        .initialize()
        .map_err(|e| JsValue::from_str(&e.to_string()))?;

    let t_handle = model.time_handle();

    let scenarios = model
        .generate_scenarios(events.event_dates(), &requests, 50_000)
        .map_err(|e| {
            JsValue::from_str(&format!("Failed to generate scenarios: {}", e.to_string()))
        })?;

    let var_map = indexer.get_variable_indexes();
    let evaluator = Evaluator::new(indexer.get_variables_size(), &scenarios);
    let vars = evaluator
        .visit_events(&events, &var_map)
        .map_err(|e| JsValue::from_str(&e.to_string()))?;

    let target = match vars.get("opt") {
        Some(Value::Number(v)) => *v,
        _ => {
            return Err(JsValue::from_str(
                "Variable 'opt' not found in events. Ensure the script contains an 'opt' which accumulates the price of the derivative.",
            ))
        }
    };

    target
        .backward()
        .map_err(|e| JsValue::from_str(&e.to_string()))?;

    let theta = t_handle.adjoint().unwrap_or(0.0) * 1.0 / 360.0;

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

    let result = SimulationResults {
        target_value: target.value(),
        theta,
        deltas: vec![deltas],
        rhos: vec![rhos],
    };

    serde_json::to_string(&result)
        .map(|s| JsValue::from_str(&s))
        .map_err(|e| JsValue::from_str(&e.to_string()))
}
