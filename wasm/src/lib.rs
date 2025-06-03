mod parsing;
mod utils;
use parsing::{create_market_store, SimulationData, SimulationResults};
use rustatlas::prelude::*;
use scripting::prelude::*;
use serde_json;
use std::collections::HashMap;
use std::result::Result as StdResult;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn run_simulation(json: &str) -> StdResult<JsValue, JsValue> {
    utils::set_panic_hook();
    Tape::start_recording();
    let data: SimulationData =
        serde_json::from_str(json).map_err(|e| JsValue::from_str(&e.to_string()))?;

    let store = create_market_store(&data.market_data);

    let events = EventStream::try_from(data.script_data.events.clone())
        .map_err(|e| JsValue::from_str(&e.to_string()))?;

    let indexer = EventIndexer::new().with_local_currency(store.local_currency());
    indexer
        .visit_events(&events)
        .map_err(|e| JsValue::from_str(&e.to_string()))?;

    let requests = indexer.get_market_requests();
    let simple = SimpleModel::new(&store);
    let model = BlackScholesModel::new(simple);
    let t_handle = model.get_time_handle();

    let scenarios: Vec<Scenario> = (0..50_000)
        .map(|_| model.gen_scenario(&requests))
        .collect::<StdResult<_, _>>()
        .map_err(|e| JsValue::from_str(&e.to_string()))?;

    let evaluator =
        EventStreamEvaluator::new(indexer.get_variables_size()).with_scenarios(&scenarios);
    let vars = evaluator
        .visit_events(&events, &indexer.get_variable_indexes())
        .map_err(|e| JsValue::from_str(&e.to_string()))?;

    let target = match vars.get("opt") {
        Some(Value::Number(v)) => *v,
        _ => {
            return Err(JsValue::from_str(
                "Variable 'opt' not found in events. Ensure the script contains an 'opt' which 
        accumulates the price of the derivative.",
            ))
        }
    };

    target
        .propagate_to_start()
        .map_err(|e| JsValue::from_str(&e.to_string()))?;

    let theta = t_handle.adjoint().unwrap_or(0.0) * 1.0 / 360.0;

    let deltas = data
        .market_data
        .fx
        .iter()
        .map(|p| {
            (
                format!("{}/{}", p.weak.code(), p.strong.code()),
                p.value.adjoint().unwrap_or(0.0),
            )
        })
        .collect::<HashMap<_, _>>();

    let rhos = data
        .market_data
        .curves
        .iter()
        .map(|c| {
            (
                c.name.clone(),
                c.rate.adjoint().unwrap_or(0.0) * 0.01 / 100.0,
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
