use crate::prelude::*;
use rayon::{
    iter::{IntoParallelIterator, ParallelIterator},
    ThreadPoolBuilder,
};

use rustatlas::prelude::*;
use std::collections::HashMap;

pub fn par_eval(
    events: &mut EventStream,
    reference_date: Date,
    data: &HistoricalData,
    local_currency: Currency,
    n_simulations: usize,
) -> Result<(f64, HashMap<String, f64>, HashMap<String, f64>)> {
    let indexer = VarIndexer::new().with_local_currency(local_currency);
    indexer.visit_events(events)?;
    let request = indexer.get_request();
    let n_vars = indexer.get_variables_size();
    let var_indexes = indexer.get_variable_indexes();

    let thread_pool_builder = ThreadPoolBuilder::new()
        .thread_name(|i| format!("pareval-thread-{}", i));
    let pool = thread_pool_builder.build().unwrap();

    let event_dates = events.event_dates();
    let results: (f64, HashMap<String, f64>, HashMap<String, f64>) = pool.install(|| {
        (0..n_simulations)
            .into_par_iter()
            .map_init(
                || {
                    Tape::start_recording();
                    Tape::set_mark();
                    let mut model =
                        BlackScholesModel::new(reference_date, local_currency, data);
                    model.initialize().unwrap();
                    model.initialize_for_parallelization();
                    model
                },
                |model, _| {
                    let scenario = model
                        .generate_scenario(event_dates.clone(), &request)
                        .unwrap();

                    let evaluator = SingleScenarioEvaluator::new()
                        .with_variables(n_vars)
                        .with_scenario(&scenario);

                    let result = evaluator.visit_events(events, &var_indexes).unwrap();

                    let price = result
                        .get("opt")
                        .and_then(|v| match v {
                            Value::Number(num) => {
                                num.backward().unwrap();
                                Some(num.value())
                            }
                            _ => None,
                        })
                        .unwrap_or(0.0);

                    let deltas = model
                        .fx()
                        .iter()
                        .map(|(pair, rate)| {
                            (
                                format!("{}/{}", pair.0.code(), pair.1.code()),
                                rate.adjoint().unwrap(),
                            )
                        })
                        .collect::<HashMap<_, _>>();

                    let rhos = model
                        .rates()
                        .iter()
                        .map(|c| {
                            (
                                c.key().name().unwrap().clone(),
                                c.values().get(0).unwrap().adjoint().unwrap(),
                            )
                        })
                        .collect::<HashMap<_, _>>();
                    Tape::rewind_to_mark();
                    (price, deltas, rhos)
                },
            )
            .reduce(
                || (0.0, HashMap::new(), HashMap::new()),
                |mut acc, (price, deltas, rhos)| {
                    acc.0 += price;
                    for (k, v) in deltas {
                        *acc.1.entry(k).or_insert(0.0) += v;
                    }
                    for (k, v) in rhos {
                        *acc.2.entry(k).or_insert(0.0) += v;
                    }
                    acc
                },
            )
    });
    // average aggregated results
    let mut total_price = results.0;
    let mut total_deltas = results.1;
    let mut total_rhos = results.2;

    total_price /= n_simulations as f64;
    for value in total_deltas.values_mut() {
        *value /= n_simulations as f64;
    }
    for value in total_rhos.values_mut() {
        *value /= n_simulations as f64;
    }

    Ok((total_price, total_deltas, total_rhos))
}

#[cfg(test)]
mod tests {
    use super::*;

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

        store.mut_volatilities().add_fx_volatility(
            reference_date,
            Currency::EUR,
            Currency::USD,
            0.0,
        );

        store.mut_volatilities().add_fx_volatility(
            reference_date,
            Currency::CLP,
            Currency::USD,
            0.0,
        );

        store.mut_volatilities().add_fx_volatility(
            reference_date,
            Currency::JPY,
            Currency::USD,
            0.0,
        );

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

        // USD term structure
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

    #[test]
    fn test_par_eval() {
        let reference_date = Date::new(2025, 6, 10);
        let data = market_data(reference_date);

        let event_maturity = Date::new(2025, 7, 10);
        let script = "opt = 0;\ns = Spot(\"CLP\",\"USD\");\nopt pays s*1000000 in \"CLP\";";
        let coded = CodedEvent::new(event_maturity, script.to_string());
        let mut event = EventStream::try_from(vec![coded]).unwrap();

        let local_currency = Currency::USD;
        let n_simulations = 100;
        let result = par_eval(
            &mut event,
            reference_date,
            &data,
            local_currency,
            n_simulations,
        );
        assert!(result.is_ok(), "Expected Ok, got {:?}", result);
        let (opt_value, deltas, rhos) = result.unwrap();
        println!("Opt value: {}", opt_value);
        println!("Deltas: {:?}", deltas);
        println!("Rhos: {:?}", rhos);
    }
}
