use std::cell::RefCell;
use std::collections::HashMap;

use rustatlas::prelude::*;
use serde::{Deserialize, Serialize};

use super::{node::Node, traits::NodeVisitor};
use crate::prelude::*;
use crate::utils::errors::{Result, ScriptingError};

/// # CodedEvent
/// A coded event is a combination of a reference date and a coded expression. Its a precompiled version of an event.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CodedEvent {
    event_date: Date,
    script: String,
}

impl CodedEvent {
    pub fn new(event_date: Date, script: String) -> CodedEvent {
        CodedEvent { event_date, script }
    }

    pub fn event_date(&self) -> Date {
        self.event_date
    }

    pub fn script(&self) -> &String {
        &self.script
    }
}

/// # Event
/// An event is a combination of a reference date and an expression tree. Represents a future action that will happen at a specific date.
#[derive(Debug, Clone, PartialEq)]
pub struct Event {
    event_date: Date,
    expr: ExprTree,
}

impl Event {
    pub fn new(event_date: Date, expr: ExprTree) -> Event {
        Event { event_date, expr }
    }

    pub fn event_date(&self) -> Date {
        self.event_date
    }

    pub fn expr(&self) -> &ExprTree {
        &self.expr
    }
}

impl TryFrom<CodedEvent> for Event {
    type Error = ScriptingError;

    fn try_from(event: CodedEvent) -> Result<Event> {
        let expr = ExprTree::try_from(event.script().clone())?;
        Ok(Event::new(event.event_date(), expr))
    }
}

/// # EventStream
/// An event stream is a collection of events that will happen in the future. An event stream could represent a series of cash flows, for example.
pub struct EventStream {
    id: Option<usize>,
    events: Vec<Event>,
}

impl EventStream {
    pub fn new() -> EventStream {
        EventStream {
            events: Vec::new(),
            id: None,
        }
    }

    pub fn with_id(mut self, id: usize) -> Self {
        self.id = Some(id);
        self
    }

    pub fn with_events(mut self, events: Vec<Event>) -> Self {
        self.events = events;
        self
    }

    pub fn add_event(&mut self, event: Event) {
        self.events.push(event);
    }

    pub fn events(&self) -> &Vec<Event> {
        &self.events
    }

    pub fn event_dates(&self) -> Vec<Date> {
        self.events.iter().map(|e| e.event_date).collect()
    }
}

impl TryFrom<Vec<CodedEvent>> for EventStream {
    type Error = ScriptingError;

    fn try_from(events: Vec<CodedEvent>) -> Result<EventStream> {
        let mut event_stream = EventStream::new();
        events.iter().try_for_each(|event| -> Result<()> {
            let event = Event::try_from(event.clone())?;
            event_stream.add_event(event);
            Ok(())
        })?;
        Ok(event_stream)
    }
}

/// # EventIndexer
/// The EventIndexer is a visitor that traverses the expression tree and indexes all the variables, market requests and numerarie requests.
pub struct EventIndexer {
    variables: RefCell<HashMap<String, usize>>,
    market_requests: RefCell<Vec<MarketRequest>>,
    event_date: RefCell<Option<Date>>,
    local_currency: Option<Currency>,
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
            | Node::If(children, _) => {
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
                        let size = self.market_requests.borrow_mut().len();
                        let exchange_request = ExchangeRateRequest::new(
                            *first,
                            second.or(self.local_currency),
                            self.event_date.borrow().clone(),
                        );
                        let request = MarketRequest::new(size, None, None, Some(exchange_request));
                        self.market_requests.borrow_mut().push(request.clone());
                        opt_idx.set(size).unwrap();
                    }
                };
                Ok(())
            }
            Node::Pays(_, opt_idx) => match opt_idx.get() {
                Some(_) => Ok(()),
                None => {
                    let size = self.market_requests.borrow_mut().len();
                    let request = MarketRequest::new(size, None, None, None); // NumerarieRequest::new(size, None, None, None);
                    self.market_requests.borrow_mut().push(request.clone());
                    opt_idx.set(size).unwrap();
                    Ok(())
                }
            },
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
            local_currency: None,
        }
    }

    /// # with_event_date
    /// Set the event date of the EventIndexer
    pub fn set_event_date(self, date: Date) {
        *self.event_date.borrow_mut() = Some(date);
    }

    /// # with_event_date
    /// Set the event date of the EventIndexer
    pub fn with_event_date(mut self, date: Date) -> Self {
        self.event_date = RefCell::new(Some(date));
        self
    }

    /// # current_event_date
    pub fn current_event_date(&self) -> Option<Date> {
        self.event_date.borrow().clone()
    }

    /// # with_local_currency
    /// Set the local currency of the EventIndexer
    pub fn with_local_currency(mut self, currency: Currency) -> Self {
        self.local_currency = Some(currency);
        self
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

    pub fn get_market_requests(&self) -> Vec<MarketRequest> {
        self.market_requests.borrow_mut().clone()
    }

    pub fn visit_events(&self, events: &EventStream) -> Result<()> {
        events.events().iter().try_for_each(|event| {
            *self.event_date.borrow_mut() = Some(event.event_date());
            self.visit(event.expr())?;
            Ok(())
        })
    }
}

#[cfg(test)]
mod tests {
    use std::sync::OnceLock;

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

    #[test]
    fn test_spot_indexer() {
        let indexer = EventIndexer::new();
        let node = Box::new(Node::Spot(Currency::USD, None, OnceLock::new()));
        indexer.visit(&node).unwrap();
        let market_requests = indexer.get_market_requests();
        assert_eq!(market_requests.len(), 1);

        let request = market_requests.get(0).unwrap();
        assert_eq!(request.id(), 0);
        assert_eq!(request.fx().unwrap().first_currency(), Currency::USD);
        assert_eq!(request.fx().unwrap().second_currency(), None);
        assert_eq!(request.fx().unwrap().reference_date(), None);
    }

    #[test]
    fn test_spot_indexer_multiple() {
        let indexer = EventIndexer::new();
        let node = Box::new(Node::Spot(Currency::USD, None, OnceLock::new()));
        indexer.visit(&node).unwrap();
        let node = Box::new(Node::Spot(Currency::EUR, None, OnceLock::new()));
        indexer.visit(&node).unwrap();
        let market_requests = indexer.get_market_requests();
        assert_eq!(market_requests.len(), 2);

        let request = market_requests.get(0).unwrap();
        assert_eq!(request.id(), 0);
        assert_eq!(request.fx().unwrap().first_currency(), Currency::USD);
        assert_eq!(request.fx().unwrap().second_currency(), None);
        assert_eq!(request.fx().unwrap().reference_date(), None);

        let request = market_requests.get(1).unwrap();
        assert_eq!(request.id(), 1);
        assert_eq!(request.fx().unwrap().first_currency(), Currency::EUR);
        assert_eq!(request.fx().unwrap().second_currency(), None);
        assert_eq!(request.fx().unwrap().reference_date(), None);
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
    fn test_event_indexer_with_local_currency() {
        // Test setting the local currency and retrieving it
        let currency = Currency::USD;
        let indexer = EventIndexer::new().with_local_currency(currency);
        assert_eq!(indexer.local_currency, Some(currency));
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
}
