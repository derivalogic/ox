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
}

impl NodeVisitor for EventIndexer {
    type Output = Result<()>;
    fn visit(&self, node: &Box<Node>) -> Self::Output {
        match node.as_ref() {
            Node::Base(children)
            | Node::Add(children)
            | Node::Subtract(children)
            | Node::Multiply(children)
            | Node::Divide(children)
            | Node::Assign(children)
            | Node::Min(children)
            | Node::Max(children)
            | Node::Exp(children)
            | Node::Pow(children)
            | Node::Ln(children)
            | Node::Append(children)
            | Node::Mean(children)
            | Node::Std(children)
            | Node::UnaryPlus(children)
            | Node::UnaryMinus(children)
            | Node::Equal(children)
            | Node::NotEqual(children)
            | Node::And(children)
            | Node::Or(children)
            | Node::Not(children)
            | Node::Superior(children)
            | Node::Inferior(children)
            | Node::SuperiorOrEqual(children)
            | Node::InferiorOrEqual(children)
            | Node::Pays(children, _)
            | Node::If(children, _) => {
                children.iter().try_for_each(|child| self.visit(child))?;
                Ok(())
            }
            Node::ForEach(name, iter, children, opt_idx) => {
                self.visit(iter)?;
                match opt_idx.get() {
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

            Node::Variable(children, name, opt_idx) => {
                children.iter().try_for_each(|child| self.visit(child))?;
                match opt_idx.get() {
                    Some(id) => {
                        self.variables.borrow_mut().insert(name.clone(), *id);
                    }
                    None => {
                        // check if the variable is already in the hashmap
                        if self.variables.borrow_mut().contains_key(name) {
                            let size = self.variables.borrow_mut().get(name).unwrap().clone();
                            // Update the id of the variable
                            opt_idx.set(size).unwrap();
                        } else {
                            let size = self.variables.borrow_mut().len();
                            self.variables.borrow_mut().insert(name.clone(), size);
                            // Update the id of the variable
                            opt_idx.set(size).unwrap();
                        }
                    }
                };
                Ok(())
            }
            Node::Spot(first, second, opt_idx) => {
                match opt_idx.get() {
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
                        self.market_requests
                            .borrow_mut()
                            .last_mut()
                            .ok_or(ScriptingError::NotFoundError(
                                "No market requests found".to_string(),
                            ))?
                            .push_fx(ExchangeRateRequest::new(*first, *second, event_date));
                        opt_idx.set(size).unwrap();
                    }
                };
                Ok(())
            }
            Node::RateIndex(name, start, end, opt_idx) => {
                match opt_idx.get() {
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
                            name.clone(),
                            *start,
                            *start,
                            *end,
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
                        opt_idx.set(size).unwrap();
                    }
                }
                Ok(())
            }
            // Node::Pays(children, opt_idx) => {
            //     children.iter().try_for_each(|child| self.visit(child))?;
            //     match opt_idx.get() {
            //         Some(_) => Ok(()),
            //         None => {
            //             let size = self.market_requests.borrow_mut().len();
            //             let event_date = match self.event_date.borrow().clone() {
            //                 Some(date) => date,
            //                 None => {
            //                     return Err(ScriptingError::InvalidSyntax(
            //                         "Event date is not set".to_string(),
            //                     ));
            //                 }
            //             };
            //             let numerarie_request = NumerarieRequest::new(size, event_date);
            //             let request =
            //                 MarketRequest::new(size, None, None, None, Some(numerarie_request));
            //             self.market_requests.borrow_mut().push(request.clone());
            //             opt_idx.set(size).unwrap();
            //             Ok(())
            //         }
            //     }
            // }
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
        }
    }

    /// # with_event_date
    /// Set the event date of the EventIndexer
    pub fn with_event_date(self, date: Date) -> Self {
        *self.event_date.borrow_mut() = Some(date);
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
    }

    pub fn visit_events(&self, events: &EventStream) -> Result<()> {
        events.events().iter().try_for_each(|event| {
            *self.event_date.borrow_mut() = Some(event.event_date());
            self.market_requests
                .borrow_mut()
                .push(SimulationDataRequest::new());
            self.visit(event.expr())?;
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
        let node = Box::new(Node::new_variable("x".to_string()));
        indexer.visit(&node).unwrap();
        let variables = indexer.get_variable_indexes();
        assert_eq!(variables.get("x"), Some(&0));
        print!("{:?}", node);
    }

    #[test]
    fn test_expression_indexer_multiple() {
        let indexer = EventIndexer::new();
        let node = Box::new(Node::new_variable("x".to_string()));
        indexer.visit(&node).unwrap();
        let node = Box::new(Node::new_variable("y".to_string()));
        indexer.visit(&node).unwrap();
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
        let node = Box::new(Node::new_variable("x".to_string()));
        indexer.visit(&node).unwrap();
        assert_eq!(indexer.get_variable_index("x"), Some(0));
    }

    #[test]
    fn test_get_variable_name() {
        // Test retrieving the name of a variable by its index
        let indexer = EventIndexer::new();
        let node = Box::new(Node::new_variable("x".to_string()));
        indexer.visit(&node).unwrap();
        assert_eq!(indexer.get_variable_name(0), Some("x".to_string()));
    }

    #[test]
    fn test_get_variables() {
        // Test retrieving all variable names
        let indexer = EventIndexer::new();
        let node = Box::new(Node::new_variable("x".to_string()));
        indexer.visit(&node).unwrap();
        let node = Box::new(Node::new_variable("y".to_string()));
        indexer.visit(&node).unwrap();
        let variables = indexer.get_variables();
        assert!(variables.contains(&"x".to_string()));
        assert!(variables.contains(&"y".to_string()));
    }

    #[test]
    fn test_get_variable_indexes() {
        // Test retrieving all variable indexes
        let indexer = EventIndexer::new();
        let node = Box::new(Node::new_variable("x".to_string()));
        indexer.visit(&node).unwrap();
        let node = Box::new(Node::new_variable("y".to_string()));
        indexer.visit(&node).unwrap();
        let variable_indexes = indexer.get_variable_indexes();
        assert_eq!(variable_indexes.get("x"), Some(&0));
        assert_eq!(variable_indexes.get("y"), Some(&1));
    }

    #[test]
    fn test_get_variables_size() {
        // Test retrieving the size of the variables hashmap
        let indexer = EventIndexer::new();
        let node = Box::new(Node::new_variable("x".to_string()));
        indexer.visit(&node).unwrap();
        let node = Box::new(Node::new_variable("y".to_string()));
        indexer.visit(&node).unwrap();
        assert_eq!(indexer.get_variables_size(), 2);
    }

    #[test]
    fn test_visit_index_node() {
        let script = "arr = [1,2,3]; x = arr[1];";
        let expr = ExprTree::try_from(script).unwrap();
        let indexer = EventIndexer::new();
        indexer.visit(&expr).unwrap();
        let variable_indexes = indexer.get_variable_indexes();
        assert_eq!(variable_indexes.get("arr"), Some(&0));
        assert_eq!(variable_indexes.get("x"), Some(&1));
    }
}
