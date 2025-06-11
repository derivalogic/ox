use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rustatlas::prelude::*;
use scripting::prelude::*;
use scripting::utils::pareval::par_eval;

fn market_data(reference_date: Date) -> HistoricalData {
    let mut store = HistoricalData::new();
    store.mut_exchange_rates().add_exchange_rate(
        reference_date,
        Currency::CLP,
        Currency::USD,
        936.405795,
    );
    store.mut_exchange_rates().add_exchange_rate(
        reference_date,
        Currency::JPY,
        Currency::USD,
        142.74,
    );
    store.mut_exchange_rates().add_exchange_rate(
        reference_date,
        Currency::EUR,
        Currency::USD,
        0.876,
    );

    store
        .mut_volatilities()
        .add_fx_volatility(reference_date, Currency::EUR, Currency::USD, 0.0);
    store
        .mut_volatilities()
        .add_fx_volatility(reference_date, Currency::CLP, Currency::USD, 0.0);
    store
        .mut_volatilities()
        .add_fx_volatility(reference_date, Currency::JPY, Currency::USD, 0.0);

    // general
    let year_fractions = vec![1.0];
    let interpolator = Interpolator::Linear;
    let enable_extrapolation = true;
    let rate_definition = RateDefinition::default();
    let term_structure_type = TermStructureType::FlatForward;

    // CLP term structure
    let clp_ts_key = TermStructureKey::new(Currency::CLP, true, Some("CLP".to_string()));
    let clp_rate = vec![0.046];
    let clp_ts = TermStructure::new(
        clp_ts_key,
        year_fractions.clone(),
        clp_rate,
        interpolator,
        enable_extrapolation,
        rate_definition,
        term_structure_type,
    );

    // USD term structure
    let usd_ts_key = TermStructureKey::new(Currency::USD, true, Some("USD".to_string()));
    let usd_rate = vec![0.036];
    let usd_ts = TermStructure::new(
        usd_ts_key,
        year_fractions.clone(),
        usd_rate,
        interpolator,
        enable_extrapolation,
        rate_definition,
        term_structure_type,
    );

    // EUR term structure
    let eur_ts_key = TermStructureKey::new(Currency::EUR, true, Some("EUR".to_string()));
    let eur_rate = vec![0.027];
    let eur_ts = TermStructure::new(
        eur_ts_key,
        year_fractions.clone(),
        eur_rate,
        interpolator,
        enable_extrapolation,
        rate_definition,
        term_structure_type,
    );

    // JPY term structure
    let jpy_ts_key = TermStructureKey::new(Currency::JPY, true, Some("JPY".to_string()));
    let jpy_rate = vec![0.027];
    let jpy_ts = TermStructure::new(
        jpy_ts_key,
        year_fractions.clone(),
        jpy_rate,
        interpolator,
        enable_extrapolation,
        rate_definition,
        term_structure_type,
    );

    store
        .mut_term_structures()
        .add_term_structure(reference_date, clp_ts);
    store
        .mut_term_structures()
        .add_term_structure(reference_date, usd_ts);
    store
        .mut_term_structures()
        .add_term_structure(reference_date, eur_ts);
    store
        .mut_term_structures()
        .add_term_structure(reference_date, jpy_ts);

    store
}

fn vanilla_event(event_maturity: Date) -> Event {
    let script = "opt = 0;\ns = Spot(\"CLP\",\"USD\");\nopt pays s*1000000 in \"CLP\";";
    let coded = CodedEvent::new(event_maturity, script.to_string());
    Event::try_from(coded).expect("Failed to create event")
}

fn seq_eval(
    events: &mut EventStream,
    reference_date: Date,
    data: &HistoricalData,
    local_currency: Currency,
    n_sim: usize,
) -> Result<f64> {
    let indexer = VarIndexer::new().with_local_currency(local_currency);
    indexer.visit_events(events)?;
    let request = indexer.get_request();
    let n_vars = indexer.get_variables_size();
    let var_indexes = indexer.get_variable_indexes();

    let mut model = BlackScholesModel::new(reference_date, local_currency, data);
    model.initialize()?;

    Tape::start_recording();
    Tape::set_mark();
    let mut total = 0.0;
    for _ in 0..n_sim {
        let scenario = model.generate_scenario(events.event_dates(), &request)?;
        let evaluator = SingleScenarioEvaluator::new()
            .with_variables(n_vars)
            .with_scenario(&scenario);
        let result = evaluator.visit_events(events, &var_indexes)?;
        if let Some(Value::Number(v)) = result.get("opt") {
            total += v.value();
        }
        Tape::rewind_to_mark();
    }
    Ok(total / n_sim as f64)
}

fn bench_sequential(c: &mut Criterion) {
    let reference_date = Date::new(2025, 6, 10);
    let data = market_data(reference_date);
    let event_maturity = Date::new(2025, 7, 10);
    let template = vanilla_event(event_maturity);
    let local_currency = Currency::USD;
    let n_sim = 10_000;
    c.bench_function("sequential valuation", |b| {
        b.iter(|| {
            let mut events = EventStream::new().with_events(vec![template.clone()]);
            let price = seq_eval(&mut events, reference_date, &data, local_currency, n_sim)
                .expect("seq eval failed");
            black_box(price);
        })
    });
}

fn bench_parallel(c: &mut Criterion) {
    let reference_date = Date::new(2025, 6, 10);
    let data = market_data(reference_date);
    let event_maturity = Date::new(2025, 7, 10);
    let template = vanilla_event(event_maturity);
    let local_currency = Currency::USD;
    let n_sim = 10_000;
    c.bench_function("parallel pareval", |b| {
        b.iter(|| {
            let mut events = EventStream::new().with_events(vec![template.clone()]);
            let price = par_eval(&mut events, reference_date, &data, local_currency, n_sim)
                .expect("par eval failed");
            black_box(price);
        })
    });
}

criterion_group!(benches, bench_sequential, bench_parallel);
criterion_main!(benches);
