
use lefi::prelude::*;
use lefi::utils::errors::Result;
use rustatlas::currencies::enums::Currency;
use rustatlas::math::ad::{backward, reset_tape, Var};
use rustatlas::models::black_scholes::{
    bs_price_delta_gamma_theta, BlackScholesModel,
};
use rustatlas::time::{date::Date, daycounter::DayCounter};


fn main() -> Result<()> {
    // Model parameters
    let ref_date = Date::new(2024, 1, 1);
    let maturity = Date::new(2025, 1, 1);
    let t = DayCounter::Actual365.year_fraction::<f64>(ref_date, maturity);
    let s0 = 850.0;
    let k = 900.0;
    let r = 0.03;
    let vol = 0.2;

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
    let s0_var = Var::new(s0);
    let model = BlackScholesModel::new(s0_var, Var::from(r), Var::from(vol), Var::from(t), ref_date);
    let scenarios = model.gen_scenarios(&requests, 5000, 42)?;

    // Evaluate the script under all scenarios
    let var_map = indexer.get_variable_indexes();
    let evaluator = EventStreamEvaluator::<Var>::new_with_type(indexer.get_variables_size())
        .with_scenarios(&scenarios);
    let vars = evaluator.visit_events(&events, &var_map)?;
    let price_var = match vars.get("opt").unwrap() {
        Value::Number(v) => *v,
        _ => Var::from(0.0),
    };
    let price_mc = price_var.value();

    // Derivative with respect to the spot
    let grad = backward(&price_var);
    let delta_ad = grad[s0_var.id()];

    // Analytic Black-Scholes results
    let (price_bs, delta_bs, gamma_bs, theta_bs) = bs_price_delta_gamma_theta(s0, k, r, vol, t);

    println!("Monte Carlo price: {:.6}", price_mc);
    println!("Black-Scholes price: {:.6}", price_bs);
    println!("Delta analytic vs AD:  {:.6}  {:.6}", delta_bs, delta_ad);
    println!("Gamma analytic: {:.6}", gamma_bs);
    println!("Theta analytic: {:.6}", theta_bs);
    Ok(())
}
