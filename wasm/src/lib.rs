use std::cell::RefCell;
use std::collections::HashMap;

use std::sync::{Arc, RwLock};
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;
use web_sys::{Request, RequestInit, RequestMode, Response};

use lefi::prelude::*;
use rustatlas::models::deterministicmodel::DeterministicModel;
use rustatlas::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json;
use wasm_bindgen::prelude::*;

// ----------- Structures for market-based pricing -----------
#[derive(Deserialize)]
struct FixingInput {
    date: Date,
    value: f64,
}

#[derive(Deserialize)]
struct CurveIndexInput {
    index_type: String,
    tenor: Period,
    fixings: Vec<FixingInput>,
}

#[derive(Deserialize)]
struct CurveDetailsInput {
    discounts: Vec<FixingInput>,
    day_counter: DayCounter,
    interpolation: Interpolator,
}

#[derive(Deserialize)]
struct CurveInput {
    curve_name: String,
    currency: Currency,
    is_risk_free: bool,
    is_forward_curve: bool,
    id: usize,
    curve_type: String,
    curve_index: CurveIndexInput,
    curve_details: CurveDetailsInput,
}

#[derive(Deserialize)]
struct FxInput {
    strong_ccy: Currency,
    weak_ccy: Currency,
    value: f64,
}

#[derive(Deserialize)]
struct MarketDataInput {
    reference_date: Date,
    fx: Vec<FxInput>,
    curves: Vec<CurveInput>,
}

#[derive(Deserialize)]
struct ScriptEventInput {
    date: Date,
    code: String,
}

#[derive(Deserialize)]
struct ScriptDataInput {
    events: Vec<ScriptEventInput>,
}

#[derive(Deserialize)]
struct PricingRequest {
    market_data: MarketDataInput,
    script_data: ScriptDataInput,
}

#[derive(Deserialize)]
struct PricingInput {
    script: String,
}

#[derive(Serialize)]
struct PricingOutput {
    variables: HashMap<String, Value>,
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

fn build_url(path: &str) -> std::result::Result<String, JsValue> {
    BASE_URL.with(|b| {
        b.borrow()
            .as_ref()
            .map(|base| format!("{}/rest/v1{}", base, path))
            .ok_or_else(|| JsValue::from_str("base url not set"))
    })
}

async fn fetch_json(url: String, api_key: &str) -> std::result::Result<JsValue, JsValue> {
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
pub fn run_pricing(script_json: &str) -> std::result::Result<JsValue, JsValue> {
    let input: PricingInput =
        serde_json::from_str(script_json).map_err(|e| JsValue::from_str(&e.to_string()))?;

    let expr = ExprTree::try_from(input.script).map_err(|e| JsValue::from_str(&format!("{e}")))?;

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
pub fn run_pricing_with_risk(
    script_json: &str,
    target: &str,
) -> std::result::Result<JsValue, JsValue> {
    let input: PricingInput =
        serde_json::from_str(script_json).map_err(|e| JsValue::from_str(&e.to_string()))?;

    let expr =
        ExprTree::try_from(input.script.clone()).map_err(|e| JsValue::from_str(&format!("{e}")))?;

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
    let evaluator_ad =
        ExprEvaluator::<Var>::new_with_type().with_variables(indexer.get_variables_size());
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

    serde_wasm_bindgen::to_value(&RiskOutput {
        price,
        sensitivities: sens,
    })
    .map_err(|e| JsValue::from_str(&e.to_string()))
}

#[wasm_bindgen]
pub fn init_market_data(base_url: &str) {
    BASE_URL.with(|b| {
        *b.borrow_mut() = Some(base_url.to_string());
    });
}

#[wasm_bindgen]
pub async fn get_spot_rate(
    api_key: &str,
    symbol: &str,
    date: &str,
) -> std::result::Result<f64, JsValue> {
    let key = format!("{symbol}:{date}");
    if let Some(rate) = SPOT_CACHE.with(|c| c.borrow().get(&key).copied()) {
        return Ok(rate);
    }

    let url = build_url(&format!(
        "/spot_rates?symbol=eq.{symbol}&date=eq.{date}&select=rate"
    ))?;
    let val = fetch_json(url, api_key).await?;
    let parsed: Vec<serde_json::Value> = serde_wasm_bindgen::from_value(val.clone())
        .map_err(|e| JsValue::from_str(&e.to_string()))?;
    let rate = parsed
        .get(0)
        .and_then(|v| v.get("rate"))
        .and_then(|r| r.as_f64())
        .unwrap_or(0.0);
    SPOT_CACHE.with(|c| {
        c.borrow_mut().insert(key, rate);
    });
    Ok(rate)
}

#[wasm_bindgen]
pub async fn get_curve(api_key: &str, name: &str) -> std::result::Result<JsValue, JsValue> {
    if let Some(data) = CURVE_CACHE.with(|c| c.borrow().get(name).cloned()) {
        return Ok(data);
    }

    let url = build_url(&format!("/curves?name=eq.{name}"))?;
    let val = fetch_json(url, api_key).await?;
    CURVE_CACHE.with(|c| {
        c.borrow_mut().insert(name.to_string(), val.clone());
    });
    Ok(val)
}

fn build_market_store(
    data: MarketDataInput,
) -> std::result::Result<(MarketStore<f64>, Currency), JsValue> {
    let ref_date = data.reference_date;
    let local_ccy = data
        .curves
        .first()
        .map(|c| c.currency)
        .unwrap_or(Currency::USD);

    let mut store = MarketStore::<f64>::new(ref_date, local_ccy);

    for fx in data.fx {
        store
            .mut_exchange_rate_store()
            .add_exchange_rate(fx.weak_ccy, fx.strong_ccy, fx.value);
    }

    for c in data.curves {
        let ccy = c.currency;
        let day_counter = c.curve_details.day_counter;
        let interpolator = c.curve_details.interpolation;
        let dates: Vec<Date> = c.curve_details.discounts.iter().map(|f| f.date).collect();
        let dfs: Vec<f64> = c.curve_details.discounts.iter().map(|f| f.value).collect();
        let ts = Arc::new(
            DiscountTermStructure::new(dates, dfs, day_counter, interpolator, true)
                .map_err(|e| JsValue::from_str(&format!("{e}")))?,
        );

        let fixings: HashMap<Date, f64> = c
            .curve_index
            .fixings
            .iter()
            .map(|f| (f.date, f.value))
            .collect();
        let index: Arc<RwLock<dyn InterestRateIndexTrait<f64>>> =
            match c.curve_index.index_type.as_str() {
                "Ibor" => Arc::new(RwLock::new(
                    IborIndex::new(ref_date)
                        .with_tenor(c.curve_index.tenor)
                        .with_fixings(fixings)
                        .with_term_structure(ts.clone()),
                )),
                _ => Arc::new(RwLock::new(
                    OvernightIndex::new(ref_date)
                        .with_fixings(fixings)
                        .with_term_structure(ts.clone()),
                )),
            };

        store
            .mut_index_store()
            .add_index(c.id, index)
            .map_err(|e| JsValue::from_str(&format!("{e}")))?;
        store.mut_index_store().add_currency_curve(ccy, c.id);
    }

    Ok((store, local_ccy))
}

// ------------------------------------------------------------
// Core evaluation using provided market data and script events
// ------------------------------------------------------------

#[wasm_bindgen]
pub fn run_event_pricing(json: &str) -> std::result::Result<JsValue, JsValue> {
    let input: PricingRequest =
        serde_json::from_str(json).map_err(|e| JsValue::from_str(&e.to_string()))?;

    // ----- Build MarketStore -----
    let (store, local_ccy) = build_market_store(input.market_data)?;

    // ----- Parse events -----
    let coded_events: Vec<CodedEvent> = input
        .script_data
        .events
        .into_iter()
        .map(|e| CodedEvent::new(e.date, e.code))
        .collect();
    let events =
        EventStream::try_from(coded_events).map_err(|e| JsValue::from_str(&format!("{e}")))?;

    let indexer = EventIndexer::new().with_local_currency(local_ccy);
    indexer
        .visit_events(&events)
        .map_err(|e| JsValue::from_str(&format!("{e}")))?;

    let requests = indexer.get_market_requests();
    let model = SimpleModel::new(&store);
    let scenario = model
        .gen_market_data(&requests)
        .map_err(|e| JsValue::from_str(&format!("{e}")))?;
    let scenarios = vec![scenario];

    let evaluator =
        EventStreamEvaluator::new(indexer.get_variables_size()).with_scenarios(&scenarios);
    let vars = evaluator
        .visit_events(&events, &indexer.get_variable_indexes())
        .map_err(|e| JsValue::from_str(&format!("{e}")))?;

    serde_wasm_bindgen::to_value(&PricingOutput { variables: vars })
        .map_err(|e| JsValue::from_str(&e.to_string()))
}
