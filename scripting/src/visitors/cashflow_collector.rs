use rustatlas::prelude::*;
use std::{
    cell::RefCell,
    collections::{BTreeMap, HashMap},
};

use crate::prelude::*;
use crate::utils::errors::{Result, ScriptingError};

use super::evaluator::{SingleScenarioEvaluator, Value};

/// Visitor that collects undiscounted cashflows per currency for a single scenario.
pub struct SingleScenarioCashflowCollector<'a> {
    evaluator: SingleScenarioEvaluator<'a>,
    current_event_date: RefCell<Option<Date>>,
    local_currency: Currency,
    cashflows: RefCell<HashMap<Currency, BTreeMap<Date, NumericType>>>,
}

impl<'a> SingleScenarioCashflowCollector<'a> {
    pub fn new(local_currency: Currency) -> Self {
        Self {
            evaluator: SingleScenarioEvaluator::new(),
            current_event_date: RefCell::new(None),
            local_currency,
            cashflows: RefCell::new(HashMap::new()),
        }
    }

    pub fn with_scenario(mut self, scenario: &'a Scenario) -> Self {
        self.evaluator = self.evaluator.with_scenario(scenario);
        self
    }

    pub fn with_variables(mut self, n: usize) -> Self {
        self.evaluator = self.evaluator.with_variables(n);
        self
    }

    pub fn set_current_event(&self, event: usize, date: Date) {
        self.evaluator.set_current_event(event);
        *self.current_event_date.borrow_mut() = Some(date);
    }

    pub fn set_variable(&self, idx: usize, val: Value) {
        self.evaluator.set_variable(idx, val);
    }

    pub fn cashflows(&self) -> HashMap<Currency, BTreeMap<Date, NumericType>> {
        self.cashflows.borrow().clone()
    }
}

impl<'a> NodeConstVisitor for SingleScenarioCashflowCollector<'a> {
    type Output = Result<()>;
    fn const_visit(&self, node: &Node) -> Self::Output {
        match node {
            Node::Pays(data) => {
                data.children
                    .iter()
                    .try_for_each(|child| self.const_visit(child))?;

                let market_data = self.evaluator.current_market_data()?.clone();

                let current_value = self.evaluator.digit_stack.borrow_mut().pop().unwrap();
                let df_id = data.df_id.ok_or(ScriptingError::EvaluationError(
                    "Pays not indexed".to_string(),
                ))?;
                let df = market_data.get_df(df_id)?;
                let numerarie = market_data.numerarie();

                // record undiscounted cashflow
                let pay_date = data.date.unwrap_or(
                    self.current_event_date
                        .borrow()
                        .ok_or(ScriptingError::EvaluationError(
                            "Event date not set".to_string(),
                        ))?,
                );
                let ccy = data.currency.unwrap_or(self.local_currency);
                {
                    let mut map = self.cashflows.borrow_mut();
                    let entry = map.entry(ccy).or_insert_with(BTreeMap::new);
                    let amt = entry.entry(pay_date).or_insert(NumericType::new(0.0));
                    *amt = (*amt + current_value).into();
                }

                let value: NumericType = if data.currency.is_some() {
                    let fx_id = data.spot_id.ok_or(ScriptingError::EvaluationError(
                        "Pays FX not indexed".to_string(),
                    ))?;
                    let fx = market_data.get_fx(fx_id)?;
                    ((current_value * df * fx) / numerarie).into()
                } else {
                    ((current_value * df) / numerarie).into()
                };

                self.evaluator.digit_stack.borrow_mut().push(value);
                Ok(())
            }
            Node::Base(data)
            | Node::Add(data)
            | Node::Subtract(data)
            | Node::Multiply(data)
            | Node::Divide(data)
            | Node::Assign(data)
            | Node::Min(data)
            | Node::Max(data)
            | Node::Exp(data)
            | Node::Pow(data)
            | Node::Ln(data)
            | Node::Fif(data)
            | Node::Cvg(data)
            | Node::Append(data)
            | Node::Mean(data)
            | Node::Std(data)
            | Node::UnaryPlus(data)
            | Node::UnaryMinus(data)
            | Node::Equal(data)
            | Node::NotEqual(data)
            | Node::And(data)
            | Node::Or(data)
            | Node::Not(data)
            | Node::Superior(data)
            | Node::Inferior(data)
            | Node::SuperiorOrEqual(data)
            | Node::InferiorOrEqual(data)
            | Node::Range(data)
            | Node::List(data) => {
                data.children
                    .iter()
                    .try_for_each(|child| self.const_visit(child))?;
                self.evaluator.const_visit(node)
            }
            Node::Index(data) => {
                data.children
                    .iter()
                    .try_for_each(|child| self.const_visit(child))?;
                self.evaluator.const_visit(node)
            }
            Node::ForEach(data) => {
                data.children
                    .iter()
                    .try_for_each(|child| self.const_visit(child))?;
                self.const_visit(&data.node)?;
                self.evaluator.const_visit(node)
            }
            Node::If(data) => {
                data.children
                    .iter()
                    .try_for_each(|child| self.const_visit(child))?;
                self.evaluator.const_visit(node)
            }
            _ => self.evaluator.const_visit(node),
        }
    }
}

impl<'a> SingleScenarioCashflowCollector<'a> {
    pub fn visit_events(
        &self,
        events: &EventStream,
    ) -> Result<HashMap<Currency, BTreeMap<Date, NumericType>>> {
        events.events().iter().enumerate().try_for_each(|(i, ev)| {
            self.set_current_event(i, ev.event_date());
            self.const_visit(ev.expr())
        })?;
        Ok(self.cashflows())
    }
}

pub struct ExpectedCashflows<'a> {
    n_vars: usize,
    scenarios: &'a Vec<Scenario>,
    local_currency: Currency,
}

impl<'a> ExpectedCashflows<'a> {
    pub fn new(n_vars: usize, scenarios: &'a Vec<Scenario>, local_currency: Currency) -> Self {
        Self {
            n_vars,
            scenarios,
            local_currency,
        }
    }

    pub fn visit_events(
        &self,
        events: &EventStream,
    ) -> Result<HashMap<Currency, BTreeMap<Date, NumericType>>> {
        let mut agg: HashMap<Currency, BTreeMap<Date, NumericType>> = HashMap::new();
        for scenario in self.scenarios {
            let collector = SingleScenarioCashflowCollector::new(self.local_currency)
                .with_variables(self.n_vars)
                .with_scenario(scenario);
            let map = collector.visit_events(events)?;
            for (ccy, flows) in map {
                let entry = agg.entry(ccy).or_insert_with(BTreeMap::new);
                for (date, amt) in flows {
                    let e = entry.entry(date).or_insert(NumericType::new(0.0));
                    *e = (*e + amt).into();
                }
            }
        }
        let n = self.scenarios.len() as f64;
        for flows in agg.values_mut() {
            for amt in flows.values_mut() {
                *amt = (*amt / n).into();
            }
        }
        Ok(agg)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pays_local_currency() {
        let mut base = Node::new_base();
        let mut pays = Node::new_pays();
        pays.add_child(Node::new_constant(100.0));
        base.add_child(pays);

        let event_date = Date::new(2024, 1, 1);
        let scenario = vec![SimulationData::new(
            NumericType::new(1.0),
            vec![NumericType::new(1.0)],
            Vec::new(),
            Vec::new(),
            Vec::new(),
        )];

        let indexer = VarIndexer::new()
            .with_event_date(event_date)
            .with_local_currency(Currency::USD);
        let event = Event::new(event_date, base.clone());
        let mut events = EventStream::new().with_events(vec![event]);
        indexer.visit_events(&mut events).unwrap();

        let collector = SingleScenarioCashflowCollector::new(Currency::USD)
            .with_scenario(&scenario)
            .with_variables(indexer.get_variables_size());
        let flows = collector.visit_events(&events).unwrap();
        let amt = flows
            .get(&Currency::USD)
            .unwrap()
            .get(&event_date)
            .cloned()
            .unwrap();
        assert!((amt - 100.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_pays_foreign_currency() {
        let mut base = Node::new_base();
        let mut pays = Node::new_pays();
        pays.add_child(Node::new_constant(100.0));
        if let Node::Pays(ref mut data) = pays {
            data.date = Some(Date::new(2024, 1, 1));
            data.currency = Some(Currency::EUR);
        }
        base.add_child(pays);

        let event_date = Date::new(2024, 1, 1);
        let scenario = vec![SimulationData::new(
            NumericType::new(1.0),
            vec![NumericType::new(1.0)],
            Vec::new(),
            vec![NumericType::new(0.9)],
            Vec::new(),
        )];

        let indexer = VarIndexer::new()
            .with_event_date(event_date)
            .with_local_currency(Currency::USD);
        let event = Event::new(event_date, base.clone());
        let mut events = EventStream::new().with_events(vec![event]);
        indexer.visit_events(&mut events).unwrap();

        let collector = SingleScenarioCashflowCollector::new(Currency::USD)
            .with_scenario(&scenario)
            .with_variables(indexer.get_variables_size());
        let flows = collector.visit_events(&events).unwrap();
        let amt = flows
            .get(&Currency::EUR)
            .unwrap()
            .get(&event_date)
            .cloned()
            .unwrap();
        assert!((amt - 100.0).abs() < f64::EPSILON);
    }
}

