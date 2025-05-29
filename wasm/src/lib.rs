use std::collections::HashMap;
use std::cell::RefCell;

use wasm_bindgen_futures::JsFuture;
use web_sys::{Request, RequestInit, RequestMode, Response};
use wasm_bindgen::JsCast;

use lefi::prelude::*;
use rustatlas::math::ad::{backward, reset_tape, Var};
use wasm_bindgen::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json;

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

thread_local! {
    static BASE_URL: RefCell<Option<String>> = RefCell::new(None);
    static SPOT_CACHE: RefCell<HashMap<String, f64>> = RefCell::new(HashMap::new());
    static CURVE_CACHE: RefCell<HashMap<String, JsValue>> = RefCell::new(HashMap::new());
}

fn build_url(path: &str) -> Result<String, JsValue> {
    BASE_URL.with(|b| {
        b.borrow()
            .as_ref()
            .map(|base| format!("{}/rest/v1{}", base, path))
            .ok_or_else(|| JsValue::from_str("base url not set"))
    })
}

async fn fetch_json(url: String, api_key: &str) -> Result<JsValue, JsValue> {
    let mut opts = RequestInit::new();
    opts.method("GET");
    opts.mode(RequestMode::Cors);

    let request = Request::new_with_str_and_init(&url, &opts)?;
    request.headers().set("apikey", api_key)?;
    request
        .headers()
        .set("Authorization", &format!("Bearer {}", api_key))?;

    let window = web_sys::window().ok_or_else(|| JsValue::from_str("no window"))?;
    let resp_value = JsFuture::from(window.fetch_with_request(&request)).await?;
    let resp: Response = resp_value.dyn_into()?;
    JsFuture::from(resp.json()?).await
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

#[wasm_bindgen]
pub fn init_market_data(base_url: &str) {
    BASE_URL.with(|b| {
        *b.borrow_mut() = Some(base_url.to_string());
    });
}

#[wasm_bindgen]
pub async fn get_spot_rate(api_key: &str, symbol: &str, date: &str) -> Result<f64, JsValue> {
    let key = format!("{symbol}:{date}");
    if let Some(rate) = SPOT_CACHE.with(|c| c.borrow().get(&key).copied()) {
        return Ok(rate);
    }

    let url = build_url(&format!("/spot_rates?symbol=eq.{symbol}&date=eq.{date}&select=rate"))?;
    let val = fetch_json(url, api_key).await?;
    let parsed: Vec<serde_json::Value> = serde_wasm_bindgen::from_value(val.clone())
        .map_err(|e| JsValue::from_str(&e.to_string()))?;
    let rate = parsed
        .get(0)
        .and_then(|v| v.get("rate"))
        .and_then(|r| r.as_f64())
        .unwrap_or(0.0);
    SPOT_CACHE.with(|c| { c.borrow_mut().insert(key, rate); });
    Ok(rate)
}

#[wasm_bindgen]
pub async fn get_curve(api_key: &str, name: &str) -> Result<JsValue, JsValue> {
    if let Some(data) = CURVE_CACHE.with(|c| c.borrow().get(name).cloned()) {
        return Ok(data);
    }

    let url = build_url(&format!("/curves?name=eq.{name}"))?;
    let val = fetch_json(url, api_key).await?;
    CURVE_CACHE.with(|c| { c.borrow_mut().insert(name.to_string(), val.clone()); });
    Ok(val)
}
