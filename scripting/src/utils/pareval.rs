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
        .thread_name(|i| format!("pareval-thread-{}", i))
        .start_handler(|_| {
            TAPE.with(|t| {
                t.borrow_mut().active = true;
                ADNumber::set_tape(&mut *t.borrow_mut());
                // println!(
                //     "Status Thread: {}: {}-{:?}",
                //     std::thread::current().name().unwrap_or("unnamed"),
                //     ADNumber::has_tape(),
                //     ADNumber::tape_addr()
                // );
            });
        });
    let pool = thread_pool_builder.build().unwrap();

    let results: Vec<(
        f64,
        HashMap<String, Value>,
        HashMap<String, f64>,
        HashMap<String, f64>,
    )> = pool.install(|| {
        (0..n_simulations)
            .into_par_iter()
            .map(|_| {
                // Create a new model instance for each thread

                // println!(
                //     "Status Thread 2: {}: {}-{:?}- is_active: {}",
                //     std::thread::current().name().unwrap_or("unnamed"),
                //     ADNumber::has_tape(),
                //     ADNumber::tape_addr(),
                //     Tape::is_active()
                // );

                let mut model = BlackScholesModel::new(reference_date, local_currency, data);
                model.initialize().unwrap();

                // Generate random scenario for each simulation
                let scenario = model
                    .generate_scenario(events.event_dates(), &request)
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
                            rate.read().unwrap().adjoint().unwrap_or(0.0),
                        )
                    })
                    .collect::<HashMap<_, _>>();

                let rhos = model
                    .rates()
                    .iter()
                    .map(|c| {
                        (
                            c.key().name().unwrap().clone(),
                            c.values()
                                .get(0)
                                .unwrap()
                                .read()
                                .unwrap()
                                .adjoint()
                                .unwrap_or(0.0),
                        )
                    })
                    .collect::<HashMap<_, _>>();

                Tape::rewind_to_mark();
                (price, result, deltas, rhos)
            })
            .collect()
    });

    // avg all the results and return a single map with the average values

    let mut total_price = 0.0;
    let mut total_deltas: HashMap<String, f64> = HashMap::new();
    let mut total_rhos: HashMap<String, f64> = HashMap::new();
    let n_results = results.len() as f64;
    for (price, _result, deltas, rhos) in results {
        total_price += price;
        for (key, value) in deltas {
            *total_deltas.entry(key).or_insert(0.0) += value;
        }
        for (key, value) in rhos {
            *total_rhos.entry(key).or_insert(0.0) += value;
        }
    }
    total_price /= n_results;
    for value in total_deltas.values_mut() {
        *value /= n_results;
    }
    for value in total_rhos.values_mut() {
        *value /= n_results;
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
            800.0,
        );

        store.mut_exchange_rates().add_exchange_rate(
            reference_date,
            Currency::JPY,
            Currency::USD,
            142.0,
        );

        store.mut_volatilities().add_fx_volatility(
            reference_date,
            Currency::USD,
            Currency::CLP,
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

        store.mut_volatilities().add_fx_volatility(
            reference_date,
            Currency::USD,
            Currency::JPY,
            0.0,
        );

        store.mut_volatilities().add_fx_volatility(
            reference_date,
            Currency::CLP,
            Currency::JPY,
            0.0,
        );

        // general
        let year_fractions = vec![1.0];
        let interpolator = Interpolator::Linear;
        let enable_extrapolation = true;
        let rate_definition = RateDefinition::new(
            DayCounter::Actual360,
            Compounding::Continuous,
            Frequency::Annual,
        );
        let term_structure_type = TermStructureType::FlatForward;

        // CLP term structure
        let clp_ts_key = TermStructureKey::new(Currency::CLP, true, Some("CLP".to_string()));
        let clp_rate = vec![0.03];

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
        let usd_rate = vec![0.02];

        let usd_ts = TermStructure::new(
            usd_ts_key,
            year_fractions.clone(),
            usd_rate,
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

        // JPY term structure
        let jpy_ts_key = TermStructureKey::new(Currency::JPY, true, Some("JPY".to_string()));
        let jpy_rate = vec![0.01];
        let jpy_ts = TermStructure::new(
            jpy_ts_key,
            year_fractions,
            jpy_rate,
            interpolator,
            enable_extrapolation,
            rate_definition,
            term_structure_type,
        );
        store
            .mut_term_structures()
            .add_term_structure(reference_date, jpy_ts);

        store
    }

    #[test]
    fn test_par_eval_1() {
        let data = market_data(Date::new(2023, 10, 1));
        let script = "x=2; opt=x*x;";
        let event_date = Date::new(2023, 10, 1);
        let coded_event = CodedEvent::new(event_date, script.to_string());
        let mut event = EventStream::try_from(vec![coded_event]).unwrap();

        let local_currency = Currency::CLP;
        let n_simulations = 100_000;
        let result = par_eval(
            &mut event,
            Date::new(2023, 10, 1),
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

    #[test]
    fn test_par_eval_2() {
        let data = market_data(Date::new(2023, 10, 1));
        let script = "opt=0; opt pays Spot(\"CLP\",\"USD\")*1000000;";
        let event_date = Date::new(2023, 10, 1);
        let coded_event = CodedEvent::new(event_date, script.to_string());
        let mut event = EventStream::try_from(vec![coded_event]).unwrap();

        let local_currency = Currency::CLP;
        let n_simulations = 100_000;
        let result = par_eval(
            &mut event,
            Date::new(2023, 10, 1),
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
