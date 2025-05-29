use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use lefi::nodes::{evaluator::EventStreamEvaluator, indexer::{CodedEvent, EventIndexer, EventStream}};
use lefi::prelude::*;
use rustatlas::models::montecarlo::RiskFreeMonteCarloModel;
use rustatlas::models::traits::MonteCarloModel;
use rustatlas::core::marketstore::MarketStore;
use rustatlas::core::meta::{MarketData, MarketRequest};
use rustatlas::currencies::enums::Currency;

#[derive(Deserialize)]
pub struct PricingRequest {
    pub events: Vec<CodedEvent>,
    #[serde(default)]
    pub num_scenarios: usize,
}

#[derive(Serialize)]
pub struct PricingResponse {
    pub variables: HashMap<String, Value>,
    pub sensitivities: Vec<Vec<f64>>,
}

fn create_market_store() -> MarketStore<f64> {
    let ref_date = rustatlas::time::date::Date::new(2024, 1, 1);
    let mut store = MarketStore::new(ref_date, Currency::USD);
    store
        .mut_exchange_rate_store()
        .add_exchange_rate(Currency::CLP, Currency::USD, 850.0);
    store
}

fn bump_scenarios(
    scenarios: &[Vec<MarketData<f64>>],
    request: &MarketRequest,
    idx: usize,
    bump: f64,
) -> Vec<Vec<MarketData<f64>>> {
    scenarios
        .iter()
        .map(|sc| {
            sc.iter()
                .enumerate()
                .map(|(i, d)| {
                    if i == idx {
                        let df = d.df().ok().map(|v| if request.df().is_some() { v + bump } else { v });
                        let fwd = d.fwd().ok().map(|v| if request.fwd().is_some() { v + bump } else { v });
                        let fx = d.fx().ok().map(|v| if request.fx().is_some() { v + bump } else { v });
                        let numerarie = if request.df().is_none() && request.fwd().is_none() && request.fx().is_none() {
                            d.numerarie() + bump
                        } else {
                            d.numerarie()
                        };
                        MarketData::new(d.id(), d.reference_date(), df, fwd, fx, numerarie)
                    } else {
                        *d
                    }
                })
                .collect()
        })
        .collect()
}

fn handle_connection(mut stream: TcpStream) {
    let mut buffer = String::new();
    if stream.read_to_string(&mut buffer).is_err() {
        return;
    }
    let body = buffer.split("\r\n\r\n").nth(1).unwrap_or("");
    let req: PricingRequest = match serde_json::from_str(body) {
        Ok(r) => r,
        Err(_) => return,
    };
    let event_stream = match EventStream::try_from(req.events) {
        Ok(es) => es,
        Err(_) => return,
    };
    let indexer = EventIndexer::new().with_local_currency(Currency::USD);
    indexer.visit_events(&event_stream).ok();
    let requests = indexer.get_market_requests();
    let store = create_market_store();
    let model = RiskFreeMonteCarloModel::new(&store);
    let scenarios = model
        .gen_scenarios(&requests, req.num_scenarios.max(1))
        .unwrap_or_default();
    let var_map = indexer.get_variable_indexes();
    let evaluator = EventStreamEvaluator::new(indexer.get_variables_size()).with_scenarios(&scenarios);
    let variables = evaluator
        .visit_events(&event_stream, &var_map)
        .unwrap_or_default();

    let bump = 1e-4;
    let mut sensitivities = vec![vec![0.0; var_map.len()]; requests.len()];
    for (i, req) in requests.iter().enumerate() {
        let bumped = bump_scenarios(&scenarios, req, i, bump);
        let evaluator = EventStreamEvaluator::new(indexer.get_variables_size()).with_scenarios(&bumped);
        let bumped_vars = evaluator
            .visit_events(&event_stream, &var_map)
            .unwrap_or_default();
        for (name, idx) in &var_map {
            if let (Some(Value::Number(base)), Some(Value::Number(bump_val))) = (
                variables.get(name).cloned(),
                bumped_vars.get(name).cloned(),
            ) {
                sensitivities[i][*idx] = (bump_val - base) / bump;
            }
        }
    }

    let resp = PricingResponse { variables, sensitivities };
    let body = serde_json::to_string(&resp).unwrap();
    let response = format!(
        "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nContent-Type: application/json\r\n\r\n{}",
        body.len(), body
    );
    let _ = stream.write_all(response.as_bytes());
}

pub fn serve(addr: &str) -> std::io::Result<()> {
    let listener = TcpListener::bind(addr)?;
    for stream in listener.incoming() {
        if let Ok(stream) = stream {
            handle_connection(stream);
        }
    }
    Ok(())
}
