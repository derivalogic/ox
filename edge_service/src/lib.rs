use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use serde::{Deserialize, Serialize};
use lefi::nodes::{evaluator::EventStreamEvaluator, indexer::{CodedEvent, EventIndexer, EventStream}};
use lefi::prelude::*;
use rustatlas::models::montecarlo::RiskFreeMonteCarloModel;
use rustatlas::models::traits::MonteCarloModel;
use rustatlas::core::marketstore::MarketStore;
use rustatlas::currencies::enums::Currency;

#[derive(Deserialize)]
pub struct PricingRequest {
    pub events: Vec<CodedEvent>,
    #[serde(default)]
    pub num_scenarios: usize,
}

#[derive(Serialize)]
pub struct PricingResponse {
    pub variables: Vec<Value>,
}

fn create_market_store() -> MarketStore {
    let ref_date = rustatlas::time::date::Date::new(2024, 1, 1);
    let mut store = MarketStore::new(ref_date, Currency::USD);
    store
        .mut_exchange_rate_store()
        .add_exchange_rate(Currency::CLP, Currency::USD, 850.0);
    store
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
    let scenarios_var = model
        .gen_scenarios(&requests, req.num_scenarios.max(1))
        .unwrap_or_default();
    let scenarios = RiskFreeMonteCarloModel::scenarios_to_f64(scenarios_var);
    let evaluator = EventStreamEvaluator::new(indexer.get_variables_size()).with_scenarios(&scenarios);
    let variables = evaluator.visit_events(&event_stream).unwrap_or_default();
    let resp = PricingResponse { variables };
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
