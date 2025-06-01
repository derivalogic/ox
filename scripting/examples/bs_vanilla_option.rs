// Example: price and Greeks of a vanilla call via scripting and Black-Scholes
use lefi::prelude::*;
use lefi::utils::errors::Result;
use rustatlas::core::marketstore::MarketStore;
use rustatlas::currencies::enums::Currency;
use rustatlas::math::ad::{backward, reset_tape, Var};
use rustatlas::models::blackscholes::{
    bs_delta, bs_gamma, bs_price, bs_price_delta_gamma_theta, bs_theta, BlackScholesModel,
};
use rustatlas::models::stochasticmodel::MonteCarloModel;
use rustatlas::models::simplemodel::SimpleModel;
use rustatlas::prelude::{FlatForwardTermStructure, OvernightIndex, RateDefinition};
use rustatlas::time::{date::Date, daycounter::DayCounter};
use std::sync::{Arc, RwLock};

fn main() -> Result<()> {
    // Model parameters

    let ref_date = Date::new(2024, 1, 1);
    let maturity_date = Date::new(2025, 1, 1);
    let t = DayCounter::Actual365.year_fraction::<f64>(ref_date, maturity_date);
    let s0 = 100.0;
    let k = 100.0;
    let r = 0.05;
    let vol = 0.2;

    // Scripted payoff of a call option

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

    // Generate scenarios with the Black-Scholes model from the library
    let mut store = MarketStore::new(ref_date, Currency::USD);
    store
        .mut_exchange_rate_store()
        .add_exchange_rate(Currency::CLP, Currency::USD, s0);
    let curve = Arc::new(FlatForwardTermStructure::new(
        ref_date,
        r,
        RateDefinition::default(),
    ));
    let index = Arc::new(RwLock::new(
        OvernightIndex::new(ref_date).with_term_structure(curve),
    ));
    let _ = store.mut_index_store().add_index(0, index);
    store.mut_index_store().add_currency_curve(Currency::USD, 0);
    let simple = SimpleModel::new(&store);
    let model = BlackScholesModel::new(simple);
    let scenarios = model.gen_scenarios(&requests, 100000)?;

    // Evaluate the script under all scenarios
    let var_map = indexer.get_variable_indexes();
    let evaluator =
        EventStreamEvaluator::new(indexer.get_variables_size()).with_scenarios(&scenarios);
    let vars = evaluator.visit_events(&events, &var_map)?;
    let price_mc = match vars.get("opt").unwrap() {
        Value::Number(v) => *v,
        _ => 0.0,
    };

    // Analytic Black-Scholes price and Greeks
    let (price_bs, delta_bs, gamma_bs, theta_bs) = bs_price_delta_gamma_theta(s0, k, r, vol, t);

    // Compute Greeks with AD
    reset_tape();
    let s_var = Var::new(s0);
    let price_var = bs_price(
        s_var,
        Var::from(k),
        Var::from(r),
        Var::from(vol),
        Var::from(t),
    );
    let grad_price = backward(&price_var);
    let delta_ad = grad_price[s_var.id()];

    reset_tape();
    let s_var = Var::new(s0);
    let delta_var = bs_delta(
        s_var,
        Var::from(k),
        Var::from(r),
        Var::from(vol),
        Var::from(t),
    );
    let grad_delta = backward(&delta_var);
    let gamma_ad = grad_delta[s_var.id()];

    reset_tape();
    let t_var = Var::new(t);
    let price_var = bs_price(
        Var::from(s0),
        Var::from(k),
        Var::from(r),
        Var::from(vol),
        t_var,
    );
    let grad_theta = backward(&price_var);
    let theta_ad = grad_theta[t_var.id()];

    println!("Monte Carlo price: {:.6}", price_mc);
    println!("Black-Scholes price: {:.6}", price_bs);
    println!("Delta analytic vs AD:  {:.6}  {:.6}", delta_bs, delta_ad);
    println!("Gamma analytic vs AD:  {:.6}  {:.6}", gamma_bs, gamma_ad);
    println!("Theta analytic vs AD:  {:.6}  {:.6}", theta_bs, theta_ad);
    Ok(())
}
