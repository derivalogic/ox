use std::collections::HashMap;

use lefi::prelude::*;
use wasm_bindgen::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
struct PricingInput {
    script: String,
}

#[derive(Serialize)]
struct PricingOutput {
    variables: HashMap<String, Value<f64>>,
}

#[wasm_bindgen]
pub fn run_pricing(script_json: &str) -> Result<JsValue, JsValue> {
    let input: PricingInput = serde_json::from_str(script_json)
        .map_err(|e| JsValue::from_str(&e.to_string()))?;

    let expr = ExprTree::try_from(input.script)
        .map_err(|e| JsValue::from_str(&format!("{e}")))?;

    let indexer = EventIndexer::new();
    indexer
        .visit(&expr)
        .map_err(|e| JsValue::from_str(&format!("{e}")))?;

    let evaluator = ExprEvaluator::new().with_variables(indexer.get_variables_size());
    evaluator
        .const_visit(expr)
        .map_err(|e| JsValue::from_str(&format!("{e}")))?;

    let mut map = HashMap::new();
    for (name, idx) in indexer.get_variable_indexes() {
        if let Some(val) = evaluator.variables().get(idx) {
            map.insert(name, val.clone());
        }
    }

    serde_wasm_bindgen::to_value(&PricingOutput { variables: map })
        .map_err(|e| JsValue::from_str(&e.to_string()))
}
