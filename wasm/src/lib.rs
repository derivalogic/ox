use std::collections::HashMap;

use lefi::prelude::*;
use rustatlas::math::ad::{backward, reset_tape, Var};
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

#[derive(Serialize)]
struct RiskOutput {
    price: f64,
    sensitivities: HashMap<String, f64>,
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

#[wasm_bindgen]
pub fn run_pricing_with_risk(script_json: &str, target: &str) -> Result<JsValue, JsValue> {
    let input: PricingInput = serde_json::from_str(script_json)
        .map_err(|e| JsValue::from_str(&e.to_string()))?;

    let expr = ExprTree::try_from(input.script.clone())
        .map_err(|e| JsValue::from_str(&format!("{e}")))?;

    // Index variables
    let indexer = EventIndexer::new();
    indexer
        .visit(&expr)
        .map_err(|e| JsValue::from_str(&format!("{e}")))?;
    let var_indexes = indexer.get_variable_indexes();
    let target_idx = var_indexes
        .get(target)
        .ok_or_else(|| JsValue::from_str("target variable not found"))?
        .to_owned();

    // First pass with f64 to obtain values
    let evaluator = ExprEvaluator::new().with_variables(indexer.get_variables_size());
    evaluator
        .const_visit(expr.clone())
        .map_err(|e| JsValue::from_str(&format!("{e}")))?;
    let values = evaluator.variables();

    // Setup AD variables using the values from first pass
    reset_tape();
    let evaluator_ad = ExprEvaluator::<Var>::new_with_type()
        .with_variables(indexer.get_variables_size());
    for (_, idx) in &var_indexes {
        if let Some(Value::Number(v)) = values.get(*idx) {
            evaluator_ad.set_variable(*idx, Value::Number(Var::new(*v)));
        } else {
            evaluator_ad.set_variable(*idx, Value::Number(Var::new(0.0)));
        }
    }

    evaluator_ad
        .const_visit(expr)
        .map_err(|e| JsValue::from_str(&format!("{e}")))?;
    let vars_ad = evaluator_ad.variables();
    let price_var = match vars_ad.get(target_idx) {
        Some(Value::Number(v)) => *v,
        _ => return Err(JsValue::from_str("target variable not numeric")),
    };
    let price = price_var.value();
    let grad = backward(&price_var);

    let mut sens = HashMap::new();
    for (name, idx) in var_indexes {
        if let Some(Value::Number(v)) = evaluator_ad.variables().get(idx) {
            sens.insert(name, grad[v.id()]);
        }
    }

    serde_wasm_bindgen::to_value(&RiskOutput { price, sensitivities: sens })
        .map_err(|e| JsValue::from_str(&e.to_string()))
}
