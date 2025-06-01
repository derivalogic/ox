use std::{
    collections::HashMap,
    fs,
    sync::{Arc, RwLock},
};

use lefi::prelude::*;
use rustatlas::models::blackscholes::BlackScholesModel;
use rustatlas::models::stochasticmodel::MonteCarloModel;
use rustatlas::prelude::*;
use serde::Deserialize;

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

type GenResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;

fn build_market_store(data: MarketDataInput) -> GenResult<(MarketStore<f64>, Currency)> {
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
        let ts = Arc::new(DiscountTermStructure::new(
            dates,
            dfs,
            day_counter,
            interpolator,
            true,
        )?);

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

        store.mut_index_store().add_index(c.id, index)?;
        store.mut_index_store().add_currency_curve(ccy, c.id);
    }

    Ok((store, local_ccy))
}

fn main() -> GenResult<()> {
    let path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "request.json".to_string());
    let json = fs::read_to_string(path)?;
    let input: PricingRequest = serde_json::from_str(&json)?;

    let (store, local_ccy) = build_market_store(input.market_data)?;

    let coded_events: Vec<CodedEvent> = input
        .script_data
        .events
        .into_iter()
        .map(|e| CodedEvent::new(e.date, e.code))
        .collect();
    let events = EventStream::try_from(coded_events)?;

    let indexer = EventIndexer::new().with_local_currency(local_ccy);
    indexer.visit_events(&events)?;

    let requests = indexer.get_market_requests();
    let simple = SimpleModel::new(&store);
    let model = BlackScholesModel::new(simple);
    let scenarios = model.gen_scenarios(&requests, 10_000)?;

    let evaluator =
        EventStreamEvaluator::new(indexer.get_variables_size()).with_scenarios(&scenarios);
    let vars = evaluator.visit_events(&events, &indexer.get_variable_indexes())?;

    println!("{}", serde_json::to_string_pretty(&vars)?);
    Ok(())
}
