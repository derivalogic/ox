use crate::data::simulationdatarequest::DiscountFactorRequest;
use crate::prelude::*;
use crate::utils::errors::{Result, ScriptingError};
use rustatlas::prelude::*;
use std::cell::RefCell;
use std::collections::HashMap;
/// # EventIndexer
/// The EventIndexer is a visitor that traverses the expression tree and indexes all the variables, market requests and numerarie requests.
pub struct EventIndexer {
    variables: RefCell<HashMap<String, usize>>,
    market_requests: RefCell<Vec<SimulationDataRequest>>,
    event_date: RefCell<Option<Date>>,
    local_currency: RefCell<Option<Currency>>,
}

impl NodeVisitor for EventIndexer {
    type Output = Result<()>;
    fn visit(&self, node: &mut Node) -> Self::Output {
        match node {
            Node::Base(children) => {
                children
                    .children()
                    .iter_mut()
                    .try_for_each(|child| self.visit(child))?;
                Ok(())
            }
            Node::Add(children) => {
                children
                    .children()
                    .iter_mut()
                    .try_for_each(|child| self.visit(child))?;
                Ok(())
            }
            Node::Subtract(children) => {
                children
                    .children()
                    .iter_mut()
                    .try_for_each(|child| self.visit(child))?;
                Ok(())
            }
            Node::Multiply(children) => {
                children
                    .children()
                    .iter_mut()
                    .try_for_each(|child| self.visit(child))?;
                Ok(())
            }
            Node::Divide(children) => {
                children
                    .children()
                    .iter_mut()
                    .try_for_each(|child| self.visit(child))?;
                Ok(())
            }
            Node::Assign(children) => {
                children
                    .children()
                    .iter_mut()
                    .try_for_each(|child| self.visit(child))?;
                Ok(())
            }
            Node::Min(children) => {
                children
                    .children()
                    .iter_mut()
                    .try_for_each(|child| self.visit(child))?;
                Ok(())
            }
            Node::Max(children) => {
                children
                    .children()
                    .iter_mut()
                    .try_for_each(|child| self.visit(child))?;
                Ok(())
            }
            Node::Exp(children) => {
                children
                    .children()
                    .iter_mut()
                    .try_for_each(|child| self.visit(child))?;
                Ok(())
            }
            Node::Pow(children) => {
                children
                    .children()
                    .iter_mut()
                    .try_for_each(|child| self.visit(child))?;
                Ok(())
            }
            Node::Ln(children) => {
                children
                    .children()
                    .iter_mut()
                    .try_for_each(|child| self.visit(child))?;
                Ok(())
            }
            Node::Fif(children) => {
                children
                    .children()
                    .iter_mut()
                    .try_for_each(|child| self.visit(child))?;
                Ok(())
            }
            Node::Append(children) => {
                children
                    .children()
                    .iter_mut()
                    .try_for_each(|child| self.visit(child))?;
                Ok(())
            }
            Node::Mean(children) => {
                children
                    .children()
                    .iter_mut()
                    .try_for_each(|child| self.visit(child))?;
                Ok(())
            }
            Node::Std(children) => {
                children
                    .children()
                    .iter_mut()
                    .try_for_each(|child| self.visit(child))?;
                Ok(())
            }
            Node::UnaryPlus(children) => {
                children
                    .children()
                    .iter_mut()
                    .try_for_each(|child| self.visit(child))?;
                Ok(())
            }
            Node::UnaryMinus(children) => {
                children
                    .children()
                    .iter_mut()
                    .try_for_each(|child| self.visit(child))?;
                Ok(())
            }
            Node::Equal(children) => {
                children
                    .children()
                    .iter_mut()
                    .try_for_each(|child| self.visit(child))?;
                Ok(())
            }
            Node::NotEqual(children) => {
                children
                    .children()
                    .iter_mut()
                    .try_for_each(|child| self.visit(child))?;
                Ok(())
            }
            Node::And(children) => {
                children
                    .children()
                    .iter_mut()
                    .try_for_each(|child| self.visit(child))?;
                Ok(())
            }
            Node::Or(children) => {
                children
                    .children()
                    .iter_mut()
                    .try_for_each(|child| self.visit(child))?;
                Ok(())
            }
            Node::Not(children) => {
                children
                    .children()
                    .iter_mut()
                    .try_for_each(|child| self.visit(child))?;
                Ok(())
            }
            Node::Superior(children) => {
                children
                    .children()
                    .iter_mut()
                    .try_for_each(|child| self.visit(child))?;
                Ok(())
            }
            Node::Inferior(children) => {
                children
                    .children()
                    .iter_mut()
                    .try_for_each(|child| self.visit(child))?;
                Ok(())
            }
            Node::SuperiorOrEqual(children) => {
                children
                    .children()
                    .iter_mut()
                    .try_for_each(|child| self.visit(child))?;
                Ok(())
            }
            Node::InferiorOrEqual(children) => {
                children
                    .children()
                    .iter_mut()
                    .try_for_each(|child| self.visit(child))?;
                Ok(())
            }
            Node::If(children) => {
                children
                    .children()
                    .iter_mut()
                    .try_for_each(|child| self.visit(child))?;
                Ok(())
            }
            Node::ForEach(data) => {
                self.visit(data.node)?;
                match data.id.get() {
                    Some(id) => {
                        self.variables.borrow_mut().insert(name.clone(), *id);
                    }
                    None => {
                        if self.variables.borrow_mut().contains_key(name) {
                            let size = self.variables.borrow_mut().get(name).unwrap().clone();
                            opt_idx.set(size).unwrap();
                        } else {
                            let size = self.variables.borrow_mut().len();
                            self.variables.borrow_mut().insert(name.clone(), size);
                            opt_idx.set(size).unwrap();
                        }
                    }
                };
                children.iter().try_for_each(|child| self.visit(child))?;
                Ok(())
            }
            Node::Range(children) | Node::List(children) | Node::Index(children) => {
                children.iter().try_for_each(|child| self.visit(child))?;
                Ok(())
            }

            Node::Variable(data) => {
                data.children
                    .iter_mut()
                    .try_for_each(|child| self.visit(child))?;
                match data.id.get() {
                    Some(id) => {
                        self.variables.borrow_mut().insert(data.name.clone(), *id);
                    }
                    None => {
                        // check if the variable is already in the hashmap
                        if self.variables.borrow_mut().contains_key(data.name.as_str()) {
                            let size = self
                                .variables
                                .borrow_mut()
                                .get(data.name.as_str())
                                .unwrap()
                                .clone();
                            // Update the id of the variable
                            data.id = Some(size);
                        } else {
                            let size = self.variables.borrow_mut().len();
                            self.variables.borrow_mut().insert(data.name.clone(), size);
                            // Update the id of the variable
                            data.id = Some(size);
                        }
                    }
                };
                Ok(())
            }
            Node::Spot(data) => {
                match data.id {
                    Some(_) => {}
                    None => {
                        let size = self
                            .market_requests
                            .borrow_mut()
                            .last()
                            .ok_or(ScriptingError::NotFoundError(
                                "No market requests found".to_string(),
                            ))?
                            .fxs()
                            .len();
                        let event_date =
                            self.event_date
                                .borrow()
                                .ok_or(ScriptingError::InvalidSyntax(
                                    "Event date is not set".to_string(),
                                ))?;
                        let ref_date = data.date.unwrap_or(event_date);
                        self.market_requests
                            .borrow_mut()
                            .last_mut()
                            .ok_or(ScriptingError::NotFoundError(
                                "No market requests found".to_string(),
                            ))?
                            .push_fx(ExchangeRateRequest::new(data.first, data.second, ref_date));
                        data.id = Some(size);
                    }
                };
                Ok(())
            }
            Node::Df(data) => {
                match data.id {
                    Some(_) => {}
                    None => {
                        let size = self
                            .market_requests
                            .borrow_mut()
                            .last()
                            .ok_or(ScriptingError::NotFoundError(
                                "No market requests found".to_string(),
                            ))?
                            .dfs()
                            .len();
                        let curve_name = data.curve.clone().unwrap_or_else(|| "local".to_string());
                        self.market_requests
                            .borrow_mut()
                            .last_mut()
                            .ok_or(ScriptingError::NotFoundError(
                                "No market requests found".to_string(),
                            ))?
                            .push_df(DiscountFactorRequest::new(curve_name, data.date));
                        data.id = Some(size);
                    }
                }
                Ok(())
            }
            Node::RateIndex(data) => {
                match data.id {
                    Some(_) => {}
                    None => {
                        let size = self
                            .market_requests
                            .borrow_mut()
                            .last()
                            .ok_or(ScriptingError::NotFoundError(
                                "No market requests found".to_string(),
                            ))?
                            .fwds()
                            .len();
                        let fwd_request = ForwardRateRequest::new(
                            data.name.clone(),
                            data.start,
                            data.start,
                            data.end,
                            Compounding::Simple,
                            Frequency::Annual,
                        );
                        self.market_requests
                            .borrow_mut()
                            .last_mut()
                            .ok_or(ScriptingError::NotFoundError(
                                "No market requests found".to_string(),
                            ))?
                            .push_fwd(fwd_request);
                        data.id = Some(size);
                    }
                }
                Ok(())
            }
            Node::Pays(data) => {
                data.children
                    .iter_mut()
                    .try_for_each(|child| self.visit(child))?;
                match data.df_id {
                    Some(_) => {}
                    None => {
                        let event_date =
                            match data.date {
                                Some(d) => d,
                                None => self.event_date.borrow().ok_or(
                                    ScriptingError::InvalidSyntax(
                                        "Event date is not set".to_string(),
                                    ),
                                )?,
                            };
                        let size = {
                            let mut mr = self.market_requests.borrow_mut();
                            let last = mr.last_mut().ok_or(ScriptingError::NotFoundError(
                                "No market requests found".to_string(),
                            ))?;
                            let size = last.dfs().len();
                            last.push_df(DiscountFactorRequest::new(
                                "local".to_string(),
                                event_date,
                            ));
                            size
                        };
                        data.df_id = Some(size);
                    }
                };

                if let Some(ccy) = data.currency {
                    match data.spot_id {
                        Some(_) => {}
                        None => {
                            let dom = self.local_currency.borrow().ok_or(
                                ScriptingError::InvalidSyntax(
                                    "Local currency is not set".to_string(),
                                ),
                            )?;
                            let event_date = match data.date {
                                Some(d) => d,
                                None => self.event_date.borrow().ok_or(
                                    ScriptingError::InvalidSyntax(
                                        "Event date is not set".to_string(),
                                    ),
                                )?,
                            };
                            let size = {
                                let mut mr = self.market_requests.borrow_mut();
                                let last = mr.last_mut().ok_or(ScriptingError::NotFoundError(
                                    "No market requests found".to_string(),
                                ))?;
                                let size = last.fxs().len();
                                last.push_fx(ExchangeRateRequest::new(dom, ccy, event_date));
                                size
                            };
                            data.spot_id = Some(size);
                        }
                    }
                }
                Ok(())
            }
            _ => Ok(()),
        }
    }
}

impl EventIndexer {
    pub fn new() -> Self {
        EventIndexer {
            variables: RefCell::new(HashMap::new()),
            market_requests: RefCell::new(Vec::new()),
            event_date: RefCell::new(None),
            local_currency: RefCell::new(None),
        }
    }

    /// # with_event_date
    /// Set the event date of the EventIndexer
    pub fn with_event_date(self, date: Date) -> Self {
        *self.event_date.borrow_mut() = Some(date);
        self
    }

    pub fn with_local_currency(self, ccy: Currency) -> Self {
        *self.local_currency.borrow_mut() = Some(ccy);
        self
    }

    /// # current_event_date
    pub fn current_event_date(&self) -> Option<Date> {
        self.event_date.borrow().clone()
    }

    /// # get_variable_index
    /// Get the index of a variable by its name
    pub fn get_variable_index(&self, variable_name: &str) -> Option<usize> {
        self.variables.borrow_mut().get(variable_name).cloned()
    }

    /// # get_variable_name
    /// Get the name of a variable by its index
    pub fn get_variable_name(&self, variable_index: usize) -> Option<String> {
        self.variables
            .borrow_mut()
            .iter()
            .find(|(_, &v)| v == variable_index)
            .map(|(k, _)| k.clone())
    }

    /// # get_variables
    /// Get all the variable names
    pub fn get_variables(&self) -> Vec<String> {
        self.variables.borrow_mut().keys().cloned().collect()
    }

    pub fn get_variable_indexes(&self) -> HashMap<String, usize> {
        self.variables.borrow_mut().clone()
    }

    pub fn get_variables_size(&self) -> usize {
        self.variables.borrow_mut().len()
    }

    pub fn get_request(&self) -> Vec<SimulationDataRequest> {
        self.market_requests.borrow_mut().clone()
    }

    pub fn reset(&self) {
        self.variables.borrow_mut().clear();
        self.market_requests.borrow_mut().clear();
        *self.event_date.borrow_mut() = None;
        *self.local_currency.borrow_mut() = None;
    }

    pub fn visit_events(&self, events: &mut EventStream) -> Result<()> {
        events.mut_events().iter_mut().try_for_each(|event| {
            *self.event_date.borrow_mut() = Some(event.event_date());
            self.market_requests
                .borrow_mut()
                .push(SimulationDataRequest::new());
            self.visit(event.mut_expr())?;
            Ok(())
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::nodes::node::Node;

    #[test]
    fn test_expression_indexer() {
        let indexer = EventIndexer::new();
        let mut node = Node::new_variable("x".to_string());
        indexer.visit(&mut node).unwrap();
        let variables = indexer.get_variable_indexes();
        assert_eq!(variables.get("x"), Some(&0));
        print!("{:?}", node);
    }

    #[test]
    fn test_expression_indexer_multiple() {
        let indexer = EventIndexer::new();
        let mut node = Node::new_variable("x".to_string());
        indexer.visit(&mut node).unwrap();
        let mut node = Node::new_variable("y".to_string());
        indexer.visit(&mut node).unwrap();
        let variables = indexer.get_variable_indexes();
        assert_eq!(variables.get("x"), Some(&0));
        assert_eq!(variables.get("y"), Some(&1));
    }
}

#[cfg(test)]
mod ai_gen_tests {
    use super::*;

    #[test]
    fn test_event_indexer_with_event_date() {
        // Test setting the event date and retrieving it
        let date = Date::empty();
        let indexer = EventIndexer::new().with_event_date(date);
        assert_eq!(indexer.current_event_date(), Some(date));
    }

    #[test]
    fn test_get_variable_index() {
        // Test retrieving the index of a variable
        let indexer = EventIndexer::new();
        let mut node = Node::new_variable("x".to_string());
        indexer.visit(&mut node).unwrap();
        assert_eq!(indexer.get_variable_index("x"), Some(0));
    }

    #[test]
    fn test_get_variable_name() {
        // Test retrieving the name of a variable by its index
        let indexer = EventIndexer::new();
        let mut node = Node::new_variable("x".to_string());
        indexer.visit(&mut node).unwrap();
        assert_eq!(indexer.get_variable_name(0), Some("x".to_string()));
    }

    #[test]
    fn test_get_variables() {
        // Test retrieving all variable names
        let indexer = EventIndexer::new();
        let mut node = Node::new_variable("x".to_string());
        indexer.visit(&mut node).unwrap();
        let mut node = Node::new_variable("y".to_string());
        indexer.visit(&mut node).unwrap();
        let variables = indexer.get_variables();
        assert!(variables.contains(&"x".to_string()));
        assert!(variables.contains(&"y".to_string()));
    }

    #[test]
    fn test_get_variable_indexes() {
        // Test retrieving all variable indexes
        let indexer = EventIndexer::new();
        let mut node = Node::new_variable("x".to_string());
        indexer.visit(&mut node).unwrap();
        let mut node = Node::new_variable("y".to_string());
        indexer.visit(&mut node).unwrap();
        let variable_indexes = indexer.get_variable_indexes();
        assert_eq!(variable_indexes.get("x"), Some(&0));
        assert_eq!(variable_indexes.get("y"), Some(&1));
    }

    #[test]
    fn test_get_variables_size() {
        // Test retrieving the size of the variables hashmap
        let indexer = EventIndexer::new();
        let mut node = Node::new_variable("x".to_string());
        indexer.visit(&mut node).unwrap();
        let mut node = Node::new_variable("y".to_string());
        indexer.visit(&mut node).unwrap();
        assert_eq!(indexer.get_variables_size(), 2);
    }

    #[test]
    fn test_visit_index_node() {
        let script = "arr = [1,2,3]; x = arr[1];";
        let mut expr = Node::try_from(script).unwrap();
        let indexer = EventIndexer::new();
        indexer.visit(&mut expr).unwrap();
        let variable_indexes = indexer.get_variable_indexes();
        assert_eq!(variable_indexes.get("arr"), Some(&0));
        assert_eq!(variable_indexes.get("x"), Some(&1));
    }

    #[test]
    fn test_spot_request_with_date() {
        let script = "x = Spot(\"USD\", \"EUR\", \"2025-06-01\");";
        let expr = Node::try_from(script).unwrap();
        let event = Event::new(Date::new(2025, 1, 1), expr);
        let mut events = EventStream::new().with_events(vec![event]);

        let indexer = EventIndexer::new();
        indexer.visit_events(&mut events).unwrap();

        let req = indexer.get_request();
        let fx = &req[0].fxs()[0];
        assert_eq!(fx.first_currency(), Currency::USD);
        assert_eq!(fx.second_currency(), Currency::EUR);
        assert_eq!(fx.date(), Date::new(2025, 6, 1));
    }

    #[test]
    fn test_spot_request_uses_event_date_when_none() {
        let script = "x = Spot(\"USD\", \"EUR\");";
        let expr = Node::try_from(script).unwrap();
        let event_date = Date::new(2025, 1, 1);
        let event = Event::new(event_date, expr);
        let mut events = EventStream::new().with_events(vec![event]);

        let indexer = EventIndexer::new();
        indexer.visit_events(&mut events).unwrap();

        let req = indexer.get_request();
        let fx = &req[0].fxs()[0];
        assert_eq!(fx.date(), event_date);
    }

    #[test]
    fn test_df_request_with_curve() {
        let script = "x = Df(\"2025-06-01\", \"curve\");";
        let expr = Node::try_from(script).unwrap();
        let event = Event::new(Date::new(2025, 1, 1), expr);
        let mut events = EventStream::new().with_events(vec![event]);

        let indexer = EventIndexer::new();
        indexer.visit_events(&mut events).unwrap();

        let req = indexer.get_request();
        let df = &req[0].dfs()[0];
        assert_eq!(df.curve(), &"curve".to_string());
        assert_eq!(df.date(), Date::new(2025, 6, 1));
    }

    #[test]
    fn test_df_request_without_curve() {
        let script = "x = Df(\"2025-06-01\");";
        let expr = Node::try_from(script).unwrap();
        let event_date = Date::new(2025, 1, 1);
        let event = Event::new(event_date, expr);
        let mut events = EventStream::new().with_events(vec![event]);

        let indexer = EventIndexer::new();
        indexer.visit_events(&mut events).unwrap();

        let req = indexer.get_request();
        let df = &req[0].dfs()[0];
        assert_eq!(df.curve(), &"local".to_string());
        assert_eq!(df.date(), Date::new(2025, 6, 1));
    }
}
