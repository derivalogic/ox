use lefi::prelude::*;
use lefi::utils::errors::Result;
use rustatlas::core::marketstore::MarketStore;
use rustatlas::currencies::enums::Currency;
use rustatlas::math::ad::real::{backward, merge_thread_tape, reset_tape, Var};
use rustatlas::models::blackscholes::BlackScholesModel;
use rustatlas::models::montecarlomodel::MonteCarloModel;
use rustatlas::models::simplemodel::SimpleModel;
use rustatlas::prelude::{FlatForwardTermStructure, OvernightIndex, RateDefinition};
use rustatlas::time::{date::Date, daycounter::DayCounter};
use std::sync::{Arc, RwLock};

fn create_market_store(s0: Var, r_usd: Var, r_clp: Var) -> MarketStore<Var> {
    let ref_date = Date::new(2024, 1, 1);
    let mut store = MarketStore::new(ref_date, Currency::USD);
    store
        .mut_exchange_rate_store()
        .add_exchange_rate(Currency::CLP, Currency::USD, s0);
    let usd_curve = Arc::new(FlatForwardTermStructure::new(
        ref_date,
        r_usd,
        RateDefinition::default(),
    ));
    let index = Arc::new(RwLock::new(
        OvernightIndex::new(ref_date).with_term_structure(usd_curve),
    ));
    let _ = store.mut_index_store().add_index(0, index);
    store.mut_index_store().add_currency_curve(Currency::USD, 0);

    let clp_curve = Arc::new(FlatForwardTermStructure::new(
        ref_date,
        r_clp,
        RateDefinition::default(),
    ));
    let index_clp = Arc::new(RwLock::new(
        OvernightIndex::new(ref_date).with_term_structure(clp_curve),
    ));
    let _ = store.mut_index_store().add_index(1, index_clp);
    store.mut_index_store().add_currency_curve(Currency::CLP, 1);

    // add volatility
    store
        .mut_equity_store()
        .add_volatility(Currency::CLP, Currency::USD, Var::new(0.2));
    store
}

fn main() -> Result<()> {
    // Model parameters
    let ref_date = Date::new(2024, 1, 1);
    let maturity = Date::new(2025, 1, 1);
    let t = DayCounter::Actual365.year_fraction::<Var>(ref_date, maturity);
    let s0 = Var::new(850.0);

    let r_usd = Var::new(0.03);
    let r_clp = Var::new(0.05);
    let vol = Var::new(0.2);

    // Scripted payoff of a call option
    let script = "
    opt = 0;
    s = Spot(\"CLP\", \"USD\");
    call = max(s - 900.0, 0);
    opt pays call;
    ";

    // Build the event stream
    let coded = CodedEvent::new(maturity, script.to_string());
    let events = EventStream::try_from(vec![coded])?;
    let indexer = EventIndexer::new().with_local_currency(Currency::USD);
    indexer.visit_events(&events)?;
    let requests = indexer.get_market_requests();

    // Monte Carlo scenarios with Black-Scholes dynamics using AD variables
    reset_tape();
    let store = create_market_store(s0, r_usd, r_clp);

    let simple = SimpleModel::new(&store);
    let model = BlackScholesModel::new(simple);
    let scenarios = model.gen_scenarios(&requests, 5000)?;
    // Evaluate the script under all scenarios
    let var_map = indexer.get_variable_indexes();
    let evaluator: EventStreamEvaluator<Var> =
        EventStreamEvaluator::new(indexer.get_variables_size()).with_scenarios(&scenarios);
    let vars = evaluator.visit_events(&events, &var_map)?;
    let price_mc = match vars.get("opt") {
        Some(Value::Number(v)) => *v,
        _ => Var::new(0.0),
    };

    // Compute the Greeks using automatic differentiation
    let result = backward(&price_mc);

    let delta_ad = result.get(s0.id()).unwrap().clone();
    let rho_clp = result.get(r_clp.id()).unwrap().clone();
    let rho_usd = result.get(r_usd.id()).unwrap().clone();
    let theta_ad = result.get(t.id()).unwrap().clone();
    let vega_ad = result.get(vol.id()).unwrap().clone();

    println!("Monte Carlo Price: {}", price_mc);
    println!("Monte Carlo Delta: {}", delta_ad);
    println!("Monte Carlo Rho CLP: {}", rho_clp);
    println!("Monte Carlo Rho USD: {}", rho_usd);
    println!("Monte Carlo Theta: {}", theta_ad);
    println!("Monte Carlo Vega: {}", vega_ad);

    Ok(())
}
