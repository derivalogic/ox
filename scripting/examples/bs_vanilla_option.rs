// Example: Vanilla call option priced via scripting engine.
// Compares Monte Carlo script pricing against analytic Black-Scholes results
// and demonstrates automatic differentiation on the analytic formula.
use lefi::prelude::*;
use lefi::utils::errors::Result;
use rustatlas::currencies::enums::Currency;
use rustatlas::math::{ad::{Var, backward, reset_tape}, black_scholes::call_price_greeks};
use rustatlas::models::{black_scholes::BlackScholesModel, traits::MonteCarloModel};
use rustatlas::time::{date::Date, daycounter::DayCounter};

fn main() -> Result<()> {
    let ref_date = Date::new(2024, 1, 1);
    let maturity_date = Date::new(2025, 1, 1);
    let t = DayCounter::Actual365.year_fraction::<f64>(ref_date, maturity_date);
    let s0 = 100.0;
    let k = 100.0;
    let r = 0.05;
    let vol = 0.2;

    let script = "
    opt = 0;
    s = Spot(\"CLP\", \"USD\");
    call = max(s - 100.0, 0);
    opt pays call;
    ";

    let events = EventStream::try_from(vec![CodedEvent::new(maturity_date, script.to_string())])?;
    let indexer = EventIndexer::new().with_local_currency(Currency::USD);
    indexer.visit_events(&events)?;
    let requests = indexer.get_market_requests();

    // Monte Carlo pricing via scripting engine
    let model = BlackScholesModel::new(s0, r, vol, t, ref_date);
    let scenarios = model.gen_scenarios(&requests, 10000)?;
    let var_map = indexer.get_variable_indexes();
    let evaluator = EventStreamEvaluator::new(indexer.get_variables_size()).with_scenarios(&scenarios);
    let vars = evaluator.visit_events(&events, &var_map)?;
    let price_mc = match vars.get("opt").unwrap() { Value::Number(v) => *v, _ => 0.0 };

    // Analytic price and Greeks
    let (price_bs, delta_bs, gamma_bs, theta_bs) = call_price_greeks(s0, k, r, vol, t);

    // Automatic differentiation on analytic formula for delta
    reset_tape();
    let s_var = Var::new(s0);
    let (price_v, _d, _g, _t) = call_price_greeks(s_var, Var::from(k), Var::from(r), Var::from(vol), Var::from(t));
    let grad = backward(&price_v);
    let delta_from_ad = grad[s_var.id()];
    println!("Monte Carlo price: {:.6}", price_mc);
    println!("Black-Scholes price: {:.6}", price_bs);
    println!("Delta analytic: {:.6}", delta_bs);
    println!("Delta AD (analytic formula): {:.6}", delta_from_ad);
    println!("Gamma analytic: {:.6}", gamma_bs);
    println!("Theta analytic: {:.6}", theta_bs);

    Ok(())
}
