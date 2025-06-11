mod parsing;
mod utils;

use parsing::{SimulationData, SimulationResults};
use rustatlas::prelude::*;
use scripting::prelude::*;
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
    let mut events = EventStream::try_from(data.script_data.events.clone())
        .map_err(|e| JsValue::from_str(&e.to_string()))?;
    let indexer = VarIndexer::new().with_local_currency(data.market_data.local_currency);
    indexer
        .visit_events(&mut events)
        .map_err(|e| JsValue::from_str(&e.to_string()))?;

    let requests = indexer.get_request();

    let mut model = BlackScholesModel::new(
        data.market_data.reference_date,
        data.market_data.local_currency,
        &store,
    );

    model.use_sobol(64, 42);
    model
        .initialize()
        .map_err(|e| JsValue::from_str(&e.to_string()))?;

    let t_handle = model.time_handle();

    let scenarios = model
        .generate_scenarios(events.event_dates(), &requests, 100_000)
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
                    .unwrap_or(0.0),
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

#[cfg(test)]
pub mod tests {
    use std::{fs::File, io::Read, path::Path};

    use super::*;

    #[test]
    fn test_run_simulation() {
        // Load `data.json` located next to this example. Skip test if not present.
        let path = Path::new("wasm/src/data.json");
        let Ok(mut file) = File::open(&path) else {
            return;
        };
        let mut json = String::new();
        file.read_to_string(&mut json).unwrap();

        // Execute the pricing routine exposed by the wasm crate
        let result = run_simulation(&json).expect("simulation failed");

        // `run_simulation` returns a JsValue containing a JSON string
        let output = result
            .as_string()
            .unwrap_or_else(|| "<non-string result>".to_string());
        println!("{output}");
    }
}
