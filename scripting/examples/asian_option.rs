use std::sync::{Arc, RwLock};

use rustatlas::prelude::*;
use scripting::prelude::*;

fn create_market_store(
    local_ccy: Currency,
    s0_clpusd: NumericType,
    s0_useeur: NumericType,
    r_usd: NumericType,
    r_clp: NumericType,
    r_eur: NumericType,
) -> MarketStore {
    let ref_date = Date::new(2025, 6, 2);
    let mut store = MarketStore::new(ref_date, local_ccy);
    store
        .mut_exchange_rate_store()
        .add_exchange_rate(Currency::CLP, Currency::USD, s0_clpusd);

    store
        .mut_exchange_rate_store()
        .add_exchange_rate(Currency::USD, Currency::EUR, s0_useeur);
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

    let eur_curve = Arc::new(FlatForwardTermStructure::new(
        ref_date,
        r_eur,
        RateDefinition::default(),
    ));
    let index_eur = Arc::new(RwLock::new(
        OvernightIndex::new(ref_date).with_term_structure(eur_curve),
    ));
    let _ = store.mut_index_store().add_index(2, index_eur);
    store.mut_index_store().add_currency_curve(Currency::EUR, 2);

    // add volatility
    store.mut_exchange_rate_store().add_volatility(
        Currency::CLP,
        Currency::USD,
        NumericType::new(0.2),
    );
    // add volatility
    store.mut_exchange_rate_store().add_volatility(
        Currency::EUR,
        Currency::USD,
        NumericType::new(0.2),
    );

    store.mut_exchange_rate_store().add_volatility(
        Currency::EUR,
        Currency::CLP,
        NumericType::new(0.2),
    );
    store
}

fn main() -> scripting::utils::errors::Result<()> {
    Tape::start_recording();
    let s0_clpusd = NumericType::new(850.0);
    let s0_useeur = NumericType::new(1.1);
    let r_usd = NumericType::new(0.03);
    let r_clp = NumericType::new(0.01);
    let r_eur = NumericType::new(0.02);

    let obs1 = Date::new(2025, 6, 2);
    let obs2 = Date::new(2025, 6, 30);
    let maturity = Date::new(2025, 7, 31);

    let script1 = "
        opt = 0;
        spot_1 = Spot(\"CLP\",\"USD\");
    ";
    let script2 = "spot_2 = Spot(\"CLP\",\"USD\");";
    let script_payoff = "
        final_spot = (spot_1+spot_2)/2;
        opt pays max(final_spot - 900, 0);
    ";

    let events = EventStream::try_from(vec![
        CodedEvent::new(obs1, script1.to_string()),
        CodedEvent::new(obs2, script2.to_string()),
        CodedEvent::new(maturity, script_payoff.to_string()),
    ])?;

    let store = create_market_store(Currency::CLP, s0_clpusd, s0_useeur, r_usd, r_clp, r_eur);

    let indexer = EventIndexer::new().with_local_currency(store.local_currency());
    indexer.visit_events(&events)?;

    let simple = SimpleModel::new(&store);
    let model = BlackScholesModel::new(simple);
    let t = model.get_time_handle();
    let requests = indexer.get_market_requests();
    let sims = 10_000;
    let scenarios = (0..sims)
        .into_iter()
        .map(|_| model.gen_scenario(&requests).map_err(|e| e.into()))
        .collect::<Result<Vec<Scenario>>>()?;

    let var_map = indexer.get_variable_indexes();
    let evaluator =
        EventStreamEvaluator::new(indexer.get_variables_size()).with_scenarios(&scenarios);
    let vars = evaluator.visit_events(&events, &var_map)?;
    let price_mc = match vars.get("opt") {
        Some(Value::Number(v)) => *v,
        _ => panic!("Option price not found in the evaluated variables"),
    };

    price_mc.propagate_to_start().unwrap();

    println!("Monte Carlo Price: {}", price_mc);
    println!(
        "Monte Carlo Rho EUR: {}",
        r_eur.adjoint().unwrap() * 0.01 / 100.0
    );
    println!(
        "Monte Carlo Rho USD: {}",
        r_usd.adjoint().unwrap() * 0.01 / 100.0
    );
    println!(
        "Monte Carlo Rho CLP: {}",
        r_clp.adjoint().unwrap() * 0.01 / 100.0
    );
    println!(
        "Monte Carlo Delta CLP/USD: {}",
        s0_clpusd.adjoint().unwrap()
    );
    println!(
        "Monte Carlo Delta USD/EUR: {}",
        s0_useeur.adjoint().unwrap() / 10000.0
    );
    println!("Monte Carlo Theta: {}", t.adjoint().unwrap() * 1.0 / 360.0);
    Ok(())
}
