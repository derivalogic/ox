use rustatlas::utils::num::Real;
use rustatlas::prelude::*;
use serde::{Deserialize, Serialize};

use std::{
    collections::HashMap,
    ops::{Add, AddAssign, Div, Mul, Sub, SubAssign},
    sync::Mutex,
};

use crate::prelude::*;
use crate::utils::errors::{Result, ScriptingError};

/// # Value
/// Enum representing the possible values of a variable
/// in the scripting language. We could say that this language
/// is dynamically typed.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Value<T: Real = f64> {
    Bool(bool),
    Number(T),
    String(String),
    Null,
}

#[cfg(test)]
mod autodiff_tests {
    use super::*;
    use rustatlas::math::ad::{backward, reset_tape, Var};

    #[test]
    fn test_autodiff_square() {
        reset_tape();
        let script = "y = x * x;".to_string();
        let tokens = Lexer::new(script).tokenize().unwrap();
        let nodes = Parser::new(tokens).parse().unwrap();

        let indexer = EventIndexer::new();
        indexer.visit(&nodes).unwrap();
        let var_map = indexer.get_variable_indexes();
        let x_idx = *var_map.get("x").unwrap();
        let y_idx = *var_map.get("y").unwrap();

        let x_var = Var::new(3.0);
        let mut evaluator =
            ExprEvaluator::<Var>::new_with_type().with_variables(indexer.get_variables_size());
        evaluator.variables.lock().unwrap()[x_idx] = Value::Number(x_var);

        evaluator.const_visit(nodes).unwrap();
        let vars = evaluator.variables();
        let y_var = match vars.get(y_idx).unwrap() {
            Value::Number(v) => *v,
            _ => panic!("y not number"),
        };
        let grad = backward(&y_var);
        assert!((grad[x_var.id()] - 6.0).abs() < 1e-12);
    }

    #[test]
    fn test_autodiff_exp() {
        reset_tape();
        let script = "y = exp(x);".to_string();
        let tokens = Lexer::new(script).tokenize().unwrap();
        let nodes = Parser::new(tokens).parse().unwrap();

        let indexer = EventIndexer::new();
        indexer.visit(&nodes).unwrap();
        let var_map = indexer.get_variable_indexes();
        let x_idx = *var_map.get("x").unwrap();
        let y_idx = *var_map.get("y").unwrap();

        let x_val = 0.7;
        let x_var = Var::new(x_val);
        let mut evaluator =
            ExprEvaluator::<Var>::new_with_type().with_variables(indexer.get_variables_size());
        evaluator.variables.lock().unwrap()[x_idx] = Value::Number(x_var);

        evaluator.const_visit(nodes).unwrap();
        let vars = evaluator.variables();
        let y_var = match vars.get(y_idx).unwrap() {
            Value::Number(v) => *v,
            _ => panic!("y not number"),
        };
        let grad = backward(&y_var);
        assert!((grad[x_var.id()] - x_val.exp()).abs() < 1e-12);
    }
}

impl Add for Value {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        match (self, other) {
            (Value::Number(a), Value::Number(b)) => Value::Number(a + b),
            (Value::String(a), Value::String(b)) => Value::String(a + &b),
            _ => Value::Null,
        }
    }
}

impl AddAssign for Value {
    fn add_assign(&mut self, other: Self) {
        match (self, other) {
            (Value::Number(a), Value::Number(b)) => *a += b,
            (Value::String(a), Value::String(b)) => *a += &b,
            _ => (),
        }
    }
}

impl Sub for Value {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        match (self, other) {
            (Value::Number(a), Value::Number(b)) => Value::Number(a - b),
            _ => Value::Null,
        }
    }
}

impl SubAssign for Value {
    fn sub_assign(&mut self, other: Self) {
        match (self, other) {
            (Value::Number(a), Value::Number(b)) => *a -= b,
            _ => (),
        }
    }
}

impl Mul for Value {
    type Output = Self;

    fn mul(self, other: Self) -> Self {
        match (self, other) {
            (Value::Number(a), Value::Number(b)) => Value::Number(a * b),
            _ => Value::Null,
        }
    }
}

impl Div for Value {
    type Output = Self;

    fn div(self, other: Self) -> Self {
        match (self, other) {
            (Value::Number(a), Value::Number(b)) => Value::Number(a / b),
            _ => Value::Null,
        }
    }
}

pub type Scenario<T = f64> = Vec<MarketData<T>>;
pub type Numeraries<T = f64> = Vec<T>;

/// # ExprEvaluator
/// Visitor that evaluates the expression tree
pub struct ExprEvaluator<'a, T: Real = f64> {
    variables: Mutex<Vec<Value<T>>>,
    digit_stack: Mutex<Vec<T>>,
    boolean_stack: Mutex<Vec<bool>>,
    string_stack: Mutex<Vec<String>>,
    is_lhs_variable: Mutex<bool>,
    lhs_variable: Mutex<Option<Box<Node>>>,

    scenario: Option<&'a Scenario<T>>,
}

impl<'a, T: Real> ExprEvaluator<'a, T> {
    pub fn new_with_type() -> Self {
        ExprEvaluator {
            variables: Mutex::new(Vec::new()),
            digit_stack: Mutex::new(Vec::new()),
            boolean_stack: Mutex::new(Vec::new()),
            string_stack: Mutex::new(Vec::new()),
            is_lhs_variable: Mutex::new(false),
            lhs_variable: Mutex::new(None),
            scenario: None,
        }
    }

    pub fn with_scenario(mut self, scenario: &'a Scenario<T>) -> Self {
        self.scenario = Some(scenario);
        self
    }

    pub fn with_variables(self, n: usize) -> Self {
        self.variables.lock().unwrap().resize(n, Value::Null);
        self
    }

    pub fn variables(&self) -> Vec<Value<T>> {
        self.variables.lock().unwrap().clone()
    }

    pub fn set_variable(&self, idx: usize, val: Value<T>) {
        let mut vars = self.variables.lock().unwrap();
        if idx >= vars.len() {
            vars.resize(idx + 1, Value::Null);
        }
        vars[idx] = val;
    }

    pub fn digit_stack(&self) -> Vec<T> {
        self.digit_stack.lock().unwrap().clone()
    }

    pub fn boolean_stack(&self) -> Vec<bool> {
        self.boolean_stack.lock().unwrap().clone()
    }
}

impl<'a> ExprEvaluator<'a> {
    pub fn new() -> Self {
        ExprEvaluator::<'a, f64>::new_with_type()
    }
}

impl<'a, T: Real> NodeConstVisitor for ExprEvaluator<'a, T> {
    type Output = Result<()>;
    fn const_visit(&self, node: Box<Node>) -> Self::Output {
        let eval: Result<()> = match node.as_ref() {
            Node::Base(children) => {
                children
                    .iter()
                    .try_for_each(|child| self.const_visit(child.clone()))?;
                Ok(())
            }
            Node::Variable(_, name, index) => {
                if *self.is_lhs_variable.lock().unwrap() {
                    *self.lhs_variable.lock().unwrap() = Some(node.clone());
                    Ok(())
                } else {
                    match index.get() {
                        None => {
                            return Err(ScriptingError::EvaluationError(format!(
                                "Variable {} not indexed",
                                name
                            )))
                        }
                        Some(id) => {
                            let vars = self.variables.lock().unwrap();
                            let value = vars.get(*id).unwrap();
                            match value {
                                Value::Number(v) => self.digit_stack.lock().unwrap().push(*v),
                                Value::Bool(v) => self.boolean_stack.lock().unwrap().push(*v),
                                Value::String(v) => {
                                    self.string_stack.lock().unwrap().push(v.clone())
                                }
                                Value::Null => {
                                    return Err(ScriptingError::EvaluationError(format!(
                                        "Variable {} not initialized",
                                        name
                                    )))
                                }
                            }
                            Ok(())
                        }
                    }
                }
            }
            Node::Spot(_, _, index) => {
                let id = index.get().ok_or(ScriptingError::EvaluationError(
                    "Spot not indexed".to_string(),
                ))?;

                let market_data = self
                    .scenario
                    .ok_or(ScriptingError::EvaluationError(
                        "No scenario set".to_string(),
                    ))?
                    .get(*id)
                    .ok_or(ScriptingError::EvaluationError(
                        "Spot not found".to_string(),
                    ))?;

                self.digit_stack
                    .lock()
                    .unwrap()
                    .push(T::from(market_data.fx()?));
                Ok(())
            }
            Node::RateIndex(_, _, _, index) => {
                let id = index.get().ok_or(ScriptingError::EvaluationError(
                    "RateIndex not indexed".to_string(),
                ))?;

                let market_data = self
                    .scenario
                    .ok_or(ScriptingError::EvaluationError(
                        "No scenario set".to_string(),
                    ))?
                    .get(*id)
                    .ok_or(ScriptingError::EvaluationError(
                        "RateIndex not found".to_string(),
                    ))?;

                self.digit_stack
                    .lock()
                    .unwrap()
                    .push(T::from(market_data.fwd()?));
                Ok(())
            }
            Node::Pays(children, index) => {
                children
                    .iter()
                    .try_for_each(|child| self.const_visit(child.clone()))?;
                let id = index
                    .get()
                    .ok_or(ScriptingError::EvaluationError("No event set".to_string()))?;

                let market_data = self
                    .scenario
                    .ok_or(ScriptingError::EvaluationError(
                        "No scenario set".to_string(),
                    ))?
                    .get(*id)
                    .ok_or(ScriptingError::EvaluationError(
                        "Event not found".to_string(),
                    ))?
                    .clone();

                let current_value = self.digit_stack.lock().unwrap().pop().unwrap();
                self.digit_stack
                    .lock()
                    .unwrap()
                    .push(current_value / market_data.numerarie());
                Ok(())
            }
            Node::Constant(value) => {
                self.digit_stack.lock().unwrap().push(T::from(*value));
                Ok(())
            }
            Node::String(value) => {
                self.string_stack.lock().unwrap().push(value.clone());
                Ok(())
            }
            Node::Add(children) => {
                children
                    .iter()
                    .try_for_each(|child| self.const_visit(child.clone()))?;

                let right = self.digit_stack.lock().unwrap().pop().unwrap();
                let left = self.digit_stack.lock().unwrap().pop().unwrap();
                self.digit_stack.lock().unwrap().push(left + right);
                Ok(())
            }
            Node::Subtract(children) => {
                children
                    .iter()
                    .try_for_each(|child| self.const_visit(child.clone()))?;

                let right = self.digit_stack.lock().unwrap().pop().unwrap();
                let left = self.digit_stack.lock().unwrap().pop().unwrap();
                self.digit_stack.lock().unwrap().push(left - right);
                Ok(())
            }
            Node::Multiply(children) => {
                children
                    .iter()
                    .try_for_each(|child| self.const_visit(child.clone()))?;

                let right = self.digit_stack.lock().unwrap().pop().unwrap();
                let left = self.digit_stack.lock().unwrap().pop().unwrap();
                self.digit_stack.lock().unwrap().push(left * right);
                Ok(())
            }
            Node::Divide(children) => {
                children
                    .iter()
                    .try_for_each(|child| self.const_visit(child.clone()))?;

                let right = self.digit_stack.lock().unwrap().pop().unwrap();
                let left = self.digit_stack.lock().unwrap().pop().unwrap();
                self.digit_stack.lock().unwrap().push(left / right);
                Ok(())
            }
            Node::Assign(children) => {
                *self.is_lhs_variable.lock().unwrap() = true;
                self.const_visit(children.get(0).unwrap().clone())?;

                *self.is_lhs_variable.lock().unwrap() = false;
                self.const_visit(children.get(1).unwrap().clone())?;

                let v = self.lhs_variable.lock().unwrap().clone().unwrap();
                let variable = v.as_ref();
                match variable {
                    Node::Variable(_, name, index) => match index.get() {
                        None => {
                            return Err(ScriptingError::EvaluationError(format!(
                                "Variable {} not indexed",
                                name
                            )))
                        }
                        Some(id) => {
                            let mut variables = self.variables.lock().unwrap();
                            if !self.boolean_stack.lock().unwrap().is_empty() {
                                // Pop from boolean stack and store the boolean value
                                let value = self.boolean_stack.lock().unwrap().pop().unwrap();
                                variables[*id] = Value::Bool(value);
                                Ok(())
                            } else if !self.string_stack.lock().unwrap().is_empty() {
                                // Pop from string stack and store the string value
                                let value = self.string_stack.lock().unwrap().pop().unwrap();
                                variables[*id] = Value::String(value);
                                Ok(())
                            } else {
                                // Pop from digit stack and store the numeric value
                                let value = self.digit_stack.lock().unwrap().pop().unwrap();
                                variables[*id] = Value::Number(value);

                                Ok(())
                            }
                        }
                    },
                    _ => {
                        return Err(ScriptingError::EvaluationError(
                            "Invalid variable assignment".to_string(),
                        ))
                    }
                }
            }
            Node::NotEqual(children) => {
                children
                    .iter()
                    .try_for_each(|child| self.const_visit(child.clone()))?;

                let right = self.digit_stack.lock().unwrap().pop().unwrap();
                let left = self.digit_stack.lock().unwrap().pop().unwrap();
                self.boolean_stack
                    .lock()
                    .unwrap()
                    .push((right - left).abs() >= T::from(f64::EPSILON));

                Ok(())
            }
            Node::And(children) => {
                children
                    .iter()
                    .try_for_each(|child| self.const_visit(child.clone()))?;

                let right = self.boolean_stack.lock().unwrap().pop().unwrap();
                let left = self.boolean_stack.lock().unwrap().pop().unwrap();
                self.boolean_stack.lock().unwrap().push(left && right);

                Ok(())
            }
            Node::Or(children) => {
                children
                    .iter()
                    .try_for_each(|child| self.const_visit(child.clone()))?;

                let right = self.boolean_stack.lock().unwrap().pop().unwrap();
                let left = self.boolean_stack.lock().unwrap().pop().unwrap();
                self.boolean_stack.lock().unwrap().push(left || right);

                Ok(())
            }
            Node::Not(children) => {
                children
                    .iter()
                    .try_for_each(|child| self.const_visit(child.clone()))?;

                let value = self.boolean_stack.lock().unwrap().pop().unwrap();
                self.boolean_stack.lock().unwrap().push(!value);

                Ok(())
            }
            Node::Superior(children) => {
                children
                    .iter()
                    .try_for_each(|child| self.const_visit(child.clone()))?;

                let right = self.digit_stack.lock().unwrap().pop().unwrap();
                let left = self.digit_stack.lock().unwrap().pop().unwrap();
                self.boolean_stack.lock().unwrap().push(left > right);

                Ok(())
            }
            Node::Inferior(children) => {
                children
                    .iter()
                    .try_for_each(|child| self.const_visit(child.clone()))?;

                let right = self.digit_stack.lock().unwrap().pop().unwrap();
                let left = self.digit_stack.lock().unwrap().pop().unwrap();
                self.boolean_stack.lock().unwrap().push(left < right);

                Ok(())
            }
            Node::SuperiorOrEqual(children) => {
                children
                    .iter()
                    .try_for_each(|child| self.const_visit(child.clone()))?;

                let right = self.digit_stack.lock().unwrap().pop().unwrap();
                let left = self.digit_stack.lock().unwrap().pop().unwrap();
                self.boolean_stack.lock().unwrap().push(left >= right);

                Ok(())
            }
            Node::InferiorOrEqual(children) => {
                children
                    .iter()
                    .try_for_each(|child| self.const_visit(child.clone()))?;

                let right = self.digit_stack.lock().unwrap().pop().unwrap();
                let left = self.digit_stack.lock().unwrap().pop().unwrap();
                self.boolean_stack.lock().unwrap().push(left <= right);

                Ok(())
            }
            Node::True => {
                self.boolean_stack.lock().unwrap().push(true);

                Ok(())
            }
            Node::False => {
                self.boolean_stack.lock().unwrap().push(false);

                Ok(())
            }
            Node::Equal(children) => {
                children
                    .iter()
                    .try_for_each(|child| self.const_visit(child.clone()))?;

                let right = self.digit_stack.lock().unwrap().pop().unwrap();
                let left = self.digit_stack.lock().unwrap().pop().unwrap();

                self.boolean_stack
                    .lock()
                    .unwrap()
                    .push((right - left).abs() < T::from(f64::EPSILON));

                Ok(())
            }
            Node::UnaryPlus(children) => {
                children
                    .iter()
                    .try_for_each(|child| self.const_visit(child.clone()))?;

                Ok(())
            }
            Node::UnaryMinus(children) => {
                children
                    .iter()
                    .try_for_each(|child| self.const_visit(child.clone()))?;

                let top = self.digit_stack.lock().unwrap().pop().unwrap();
                self.digit_stack.lock().unwrap().push(-top);

                Ok(())
            }
            Node::Min(children) => {
                children
                    .iter()
                    .try_for_each(|child| self.const_visit(child.clone()))?;

                let right = self.digit_stack.lock().unwrap().pop().unwrap();
                let left = self.digit_stack.lock().unwrap().pop().unwrap();
                self.digit_stack.lock().unwrap().push(left.min(right));

                Ok(())
            }
            Node::Max(children) => {
                children
                    .iter()
                    .try_for_each(|child| self.const_visit(child.clone()))?;

                let right = self.digit_stack.lock().unwrap().pop().unwrap();
                let left = self.digit_stack.lock().unwrap().pop().unwrap();
                self.digit_stack.lock().unwrap().push(left.max(right));

                Ok(())
            }
            Node::Pow(children) => {
                children
                    .iter()
                    .try_for_each(|child| self.const_visit(child.clone()))?;

                let right = self.digit_stack.lock().unwrap().pop().unwrap();
                let left = self.digit_stack.lock().unwrap().pop().unwrap();
                self.digit_stack.lock().unwrap().push(left.powf(right));

                Ok(())
            }
            Node::Ln(children) => {
                children
                    .iter()
                    .try_for_each(|child| self.const_visit(child.clone()))?;

                let top = self.digit_stack.lock().unwrap().pop().unwrap();
                self.digit_stack.lock().unwrap().push(top.ln());

                Ok(())
            }
            Node::Exp(children) => {
                children
                    .iter()
                    .try_for_each(|child| self.const_visit(child.clone()))?;

                let top = self.digit_stack.lock().unwrap().pop().unwrap();
                self.digit_stack.lock().unwrap().push(top.exp());
                Ok(())
            }
            Node::Cvg(children) => {
                children
                    .iter()
                    .try_for_each(|child| self.const_visit(child.clone()))?;

                let basis_str = self.string_stack.lock().unwrap().pop().unwrap();
                let end_str = self.string_stack.lock().unwrap().pop().unwrap();
                let start_str = self.string_stack.lock().unwrap().pop().unwrap();

                let start = Date::from_str(&start_str, "%Y-%m-%d")?;
                let end = Date::from_str(&end_str, "%Y-%m-%d")?;
                let basis = DayCounter::try_from(basis_str)?;
                let yf = basis.year_fraction(start, end);
                self.digit_stack.lock().unwrap().push(yf);
                Ok(())
            }
            Node::If(children, first_else) => {
                // Evaluate the condition
                children.get(0).unwrap().const_accept(self);
                // Pop the condition result
                let is_true = self.boolean_stack.lock().unwrap().pop().unwrap();

                // Find the first else node
                if is_true {
                    // then, the following expressions are either conditions or
                    // the logic block
                    let last_condition = if first_else.is_none() {
                        children.len()
                    } else {
                        first_else.unwrap()
                    };

                    // Evaluate the conditions
                    for i in 1..last_condition {
                        children.get(i).unwrap().const_accept(self);
                    }
                }
                // Evaluate the else block
                else if first_else.is_some() {
                    // the following conditions are the else block
                    for i in first_else.unwrap()..children.len() {
                        children.get(i).unwrap().const_accept(self);
                    }
                }
                Ok(())
            }
        };
        eval
    }
}

/// # EventStreamEvaluator
/// Visitor that evaluates the event stream
pub struct EventStreamEvaluator<'a, T: Real = f64> {
    n_vars: usize,
    scenarios: Option<&'a Vec<Scenario<T>>>,
}

impl<'a, T: Real> EventStreamEvaluator<'a, T> {
    pub fn new_with_type(n_vars: usize) -> Self {
        EventStreamEvaluator {
            n_vars,
            scenarios: None,
        }
    }

    pub fn with_scenarios(mut self, scenarios: &'a Vec<Scenario<T>>) -> Self {
        self.scenarios = Some(scenarios);
        self
    }

    pub fn visit_events(
        &self,
        event_stream: &EventStream,
        var_indexes: &HashMap<String, usize>,
    ) -> Result<HashMap<String, Value<T>>> {
        let scenarios = self.scenarios.ok_or(ScriptingError::EvaluationError(
            "No scenarios set".to_string(),
        ))?;

        // Evaluate the events to get the variables using the first scenario
        let mut evaluator = ExprEvaluator::<'_, T>::new_with_type().with_variables(self.n_vars);
        if let Some(first) = scenarios.first() {
            evaluator = evaluator.with_scenario(first);
        }
        event_stream
            .events()
            .iter()
            .try_for_each(|event| -> Result<()> {
                evaluator.const_visit(event.expr().clone())?;
                Ok(())
            })?;

        let v: Vec<Value<T>> = evaluator
            .variables()
            .iter()
            .map(|v| match v {
                Value::Number(_) => Value::Number(T::from(0.0)),
                _ => v.clone(),
            })
            .collect();

        let global_variables = Mutex::new(v);

        scenarios.iter().try_for_each(|scenario| -> Result<()> {
            let evaluator = ExprEvaluator::<T>::new_with_type()
                .with_variables(self.n_vars)
                .with_scenario(scenario);

            event_stream
                .events()
                .iter()
                .try_for_each(|event| -> Result<()> {
                    evaluator.const_visit(event.expr().clone())?;
                    Ok(())
                })?;

            let local_variables = evaluator.variables();
            let mut vars = global_variables.lock().unwrap();
            vars.iter_mut()
                .zip(local_variables.iter())
                .for_each(|(g, l)| match (g, l) {
                    (Value::Number(g), Value::Number(l)) => *g = *g + *l,
                    _ => (),
                });

            Ok(())
        })?;

        //avg
        let mut vars = global_variables.lock().unwrap();
        let len = T::from(scenarios.len() as f64);

        vars.iter_mut().for_each(|v| match v {
            Value::Number(v) => *v = *v / len,
            _ => (),
        });

        let mut map = HashMap::new();
        for (name, idx) in var_indexes.iter() {
            if let Some(v) = vars.get(*idx) {
                map.insert(name.clone(), v.clone());
            }
        }

        Ok(map)
    }
}

impl<'a> EventStreamEvaluator<'a> {
    pub fn new(n_vars: usize) -> Self {
        EventStreamEvaluator::<'a, f64>::new_with_type(n_vars)
    }
}

#[cfg(test)]
mod general_tests {
    use super::*;

    #[test]
    fn test_add_node() {
        let mut base = Box::new(Node::new_base());
        let mut add = Box::new(Node::new_add());

        let c1 = Box::new(Node::new_constant(1.0));
        let c2 = Box::new(Node::new_constant(1.0));

        add.add_child(c1);
        add.add_child(c2);
        base.add_child(add);

        let evaluator = ExprEvaluator::new();
        evaluator.const_visit(base).unwrap();

        assert_eq!(evaluator.digit_stack().pop().unwrap(), 2.0);
    }

    #[test]
    fn test_subtract_node() {
        let mut base = Box::new(Node::new_base());
        let mut subtract = Node::new_subtract();

        let c1 = Node::new_constant(1.0);
        let c2 = Node::new_constant(1.0);

        subtract.add_child(Box::new(c1));
        subtract.add_child(Box::new(c2));
        base.add_child(Box::new(subtract));

        let evaluator = ExprEvaluator::new();
        evaluator.const_visit(base).unwrap();

        assert_eq!(evaluator.digit_stack().pop().unwrap(), 0.0);
    }

    #[test]
    fn test_multiply_node() {
        let mut base = Box::new(Node::new_base());
        let mut multiply = Node::new_multiply();

        let c1 = Node::new_constant(2.0);
        let c2 = Node::new_constant(2.0);

        multiply.add_child(Box::new(c1));
        multiply.add_child(Box::new(c2));
        base.add_child(Box::new(multiply));

        let evaluator = ExprEvaluator::new();
        evaluator.const_visit(base).unwrap();

        assert_eq!(evaluator.digit_stack().pop().unwrap(), 4.0);
    }

    #[test]
    fn test_divide_node() {
        let mut base = Box::new(Node::new_base());
        let mut divide = Node::new_divide();

        let c1 = Node::new_constant(4.0);
        let c2 = Node::new_constant(2.0);

        divide.add_child(Box::new(c1));
        divide.add_child(Box::new(c2));
        base.add_child(Box::new(divide));

        let evaluator = ExprEvaluator::new();
        evaluator.const_visit(base).unwrap();

        assert_eq!(evaluator.digit_stack().pop().unwrap(), 2.0);
    }

    #[test]
    fn test_variable_assign_node() {
        let mut base = Box::new(Node::new_base());

        let c1 = Box::new(Node::new_constant(1.0));
        let v1 = Box::new(Node::new_variable_with_id("x".to_string(), 0));

        let mut assign = Box::new(Node::new_assign());
        assign.add_child(v1);
        assign.add_child(c1);

        base.add_child(assign);

        let evaluator = ExprEvaluator::new().with_variables(1);
        evaluator.const_visit(base).unwrap();
        assert_eq!(evaluator.variables().pop().unwrap(), Value::Number(1.0));
    }

    #[test]
    fn test_assign_boolean() {
        let base = Box::new(Node::Base(vec![
            Box::new(Node::Assign(vec![
                Box::new(Node::Variable(Vec::new(), "x".to_string(), 0.into())),
                Box::new(Node::True),
            ])),
            Box::new(Node::Assign(vec![
                Box::new(Node::Variable(Vec::new(), "y".to_string(), 1.into())),
                Box::new(Node::False),
            ])),
            Box::new(Node::Assign(vec![
                Box::new(Node::Variable(Vec::new(), "z".to_string(), 2.into())),
                Box::new(Node::And(vec![
                    Box::new(Node::Variable(Vec::new(), "x".to_string(), 0.into())),
                    Box::new(Node::Variable(Vec::new(), "y".to_string(), 1.into())),
                ])),
            ])),
        ]));

        let evaluator = ExprEvaluator::new().with_variables(3);
        evaluator.const_visit(base).unwrap();

        assert_eq!(evaluator.variables().get(0).unwrap(), &Value::Bool(true));
        assert_eq!(evaluator.variables().get(1).unwrap(), &Value::Bool(false));
        assert_eq!(evaluator.variables().get(2).unwrap(), &Value::Bool(false));
    }

    #[test]
    fn test_variable_use_node() {
        let mut base = Box::new(Node::new_base());

        let c1 = Box::new(Node::new_constant(1.0));
        let v1 = Box::new(Node::new_variable_with_id("x".to_string(), 0));

        let mut add = Box::new(Node::new_add());
        add.add_child(v1);
        add.add_child(c1);

        base.add_child(add);

        let evaluator = ExprEvaluator::new().with_variables(1);
        assert!(evaluator.const_visit(base).is_err());
    }

    #[test]
    fn test_nested_expression() {
        let mut base = Box::new(Node::new_base());

        let c1 = Box::new(Node::new_constant(1.0));
        let c2 = Box::new(Node::new_constant(2.0));
        let x = Box::new(Node::new_variable_with_id("x".to_string(), 0));
        let y = Box::new(Node::new_variable_with_id("y".to_string(), 1));
        let z = Box::new(Node::new_variable_with_id("z".to_string(), 2));

        let mut assign_x = Box::new(Node::new_assign());
        assign_x.add_child(x.clone());
        assign_x.add_child(c1);

        let mut assign_y = Box::new(Node::new_assign());
        assign_y.add_child(y.clone());
        assign_y.add_child(c2);

        let mut add = Box::new(Node::new_add());
        add.add_child(x.clone());
        add.add_child(y.clone());

        let mut assign_z = Box::new(Node::new_assign());
        assign_z.add_child(z);
        assign_z.add_child(add);

        base.add_child(assign_x);
        base.add_child(assign_y);
        base.add_child(assign_z);

        let evaluator = ExprEvaluator::new().with_variables(3);
        evaluator.const_visit(base).unwrap();
        assert_eq!(evaluator.variables().pop().unwrap(), Value::Number(3.0));
    }

    #[test]
    fn test_equal() {
        let mut base = Box::new(Node::new_base());

        let c1 = Box::new(Node::new_constant(1.0));
        let c2 = Box::new(Node::new_constant(1.0));

        let mut equal = Box::new(Node::new_equal());
        equal.add_child(c1);
        equal.add_child(c2);

        base.add_child(equal);

        let evaluator = ExprEvaluator::new().with_variables(1);
        evaluator.const_visit(base).unwrap();
        assert_eq!(evaluator.boolean_stack().pop().unwrap(), true);
    }

    #[test]
    fn test_superior() {
        let mut base = Box::new(Node::new_base());

        let c1 = Box::new(Node::new_constant(2.0));
        let c2 = Box::new(Node::new_constant(1.0));

        let mut and = Box::new(Node::new_superior());
        and.add_child(c1);
        and.add_child(c2);

        base.add_child(and);

        let evaluator = ExprEvaluator::new();
        evaluator.const_visit(base).unwrap();
        assert_eq!(evaluator.boolean_stack().pop().unwrap(), true);
    }

    #[test]
    fn test_inferior() {
        let mut base = Box::new(Node::new_base());

        let c1 = Box::new(Node::new_constant(1.0));
        let c2 = Box::new(Node::new_constant(2.0));

        let mut and = Box::new(Node::new_inferior());
        and.add_child(c1);
        and.add_child(c2);

        base.add_child(and);

        let evaluator = ExprEvaluator::new();
        evaluator.const_visit(base).unwrap();

        assert_eq!(evaluator.boolean_stack().pop().unwrap(), true);
    }

    #[test]
    fn test_superior_or_equal() {
        let mut base = Box::new(Node::new_base());

        let c1 = Box::new(Node::new_constant(2.0));
        let c2 = Box::new(Node::new_constant(1.0));

        let mut and = Box::new(Node::new_superior_or_equal());
        and.add_child(c1);
        and.add_child(c2);

        base.add_child(and);

        let evaluator = ExprEvaluator::new();
        evaluator.const_visit(base).unwrap();
        assert_eq!(evaluator.boolean_stack().pop().unwrap(), true);
    }

    #[test]
    fn test_inferior_or_equal() {
        let mut base = Box::new(Node::new_base());

        let c1 = Box::new(Node::new_constant(1.0));
        let c2 = Box::new(Node::new_constant(2.0));

        let mut and = Box::new(Node::new_inferior_or_equal());
        and.add_child(c1);
        and.add_child(c2);

        base.add_child(and);

        let evaluator = ExprEvaluator::new();
        evaluator.const_visit(base).unwrap();
        assert_eq!(evaluator.boolean_stack().pop().unwrap(), true);
    }

    #[test]
    fn test_and() {
        let mut base = Box::new(Node::new_base());

        let c1 = Box::new(Node::new_constant(1.0));
        let c2 = Box::new(Node::new_constant(1.0));

        let mut equal_1 = Box::new(Node::new_equal());
        equal_1.add_child(c1.clone());
        equal_1.add_child(c2.clone());

        let mut equal_2 = Box::new(Node::new_equal());
        equal_2.add_child(c1.clone());
        equal_2.add_child(c2.clone());

        let mut and = Box::new(Node::new_and());
        and.add_child(equal_1.clone());
        and.add_child(equal_2.clone());

        base.add_child(equal_1.clone());
        base.add_child(equal_2.clone());
        base.add_child(and);

        let evaluator = ExprEvaluator::new();
        evaluator.const_visit(base).unwrap();
        assert_eq!(evaluator.boolean_stack().pop().unwrap(), true);
    }

    #[test]
    fn test_or() {
        let mut base = Box::new(Node::new_base());

        let c1 = Box::new(Node::new_constant(1.0));
        let c2 = Box::new(Node::new_constant(1.0));

        let mut equal_1 = Box::new(Node::new_equal());
        equal_1.add_child(c1.clone());
        equal_1.add_child(c2.clone());

        let mut equal_2 = Box::new(Node::new_equal());
        equal_2.add_child(c1.clone());
        equal_2.add_child(c2.clone());

        let mut or = Box::new(Node::new_or());
        or.add_child(equal_1.clone());
        or.add_child(equal_2.clone());

        base.add_child(equal_1.clone());
        base.add_child(equal_2.clone());
        base.add_child(or);

        let evaluator = ExprEvaluator::new();
        evaluator.const_visit(base).unwrap();
        assert_eq!(evaluator.boolean_stack().pop().unwrap(), true);
    }

    #[test]
    fn test_not() {
        let mut base = Box::new(Node::new_base());

        let c1 = Box::new(Node::new_constant(1.0));
        let c2 = Box::new(Node::new_constant(1.0));

        let mut equal = Box::new(Node::new_equal());
        equal.add_child(c1.clone());
        equal.add_child(c2.clone());

        let mut not = Box::new(Node::new_not());
        not.add_child(equal.clone());

        base.add_child(equal.clone());
        base.add_child(not.clone());

        let evaluator = ExprEvaluator::new();
        evaluator.const_visit(base).unwrap();
        assert_eq!(evaluator.boolean_stack().pop().unwrap(), false);
    }

    #[test]
    fn test_if() {
        let mut base = Box::new(Node::new_base());

        let x = Box::new(Node::new_variable_with_id("x".to_string(), 0));
        let c1 = Box::new(Node::new_constant(1.0));

        let mut assing_x = Box::new(Node::new_assign());
        assing_x.add_child(x.clone());
        assing_x.add_child(c1.clone());

        let mut if_node = Box::new(Node::new_if());

        let mut equal = Box::new(Node::new_equal());

        equal.add_child(x.clone());
        equal.add_child(c1.clone());

        if_node.add_child(equal.clone());

        let mut add = Box::new(Node::new_add());
        add.add_child(x.clone());
        add.add_child(c1.clone());
        let mut assing_x_2 = Box::new(Node::new_assign());
        assing_x_2.add_child(x);
        assing_x_2.add_child(add);

        if_node.add_child(assing_x_2.clone());

        base.add_child(assing_x);
        base.add_child(if_node);

        let evaluator = ExprEvaluator::new().with_variables(1);
        evaluator.const_visit(base).unwrap();
        assert_eq!(evaluator.variables().pop().unwrap(), Value::Number(2.0));
    }

    #[test]
    fn test_if_new_variable() {
        let base = Box::new(Node::Base(vec![
            Box::new(Node::Assign(vec![
                Box::new(Node::Variable(Vec::new(), "x".to_string(), 0.into())),
                Box::new(Node::Constant(2.0)),
            ])),
            Box::new(Node::If(
                vec![
                    Box::new(Node::Equal(vec![
                        Box::new(Node::Variable(Vec::new(), "x".to_string(), 0.into())),
                        Box::new(Node::Constant(1.0)),
                    ])),
                    Box::new(Node::Assign(vec![
                        Box::new(Node::Variable(Vec::new(), "z".to_string(), 1.into())),
                        Box::new(Node::Constant(3.0)),
                    ])),
                    Box::new(Node::Assign(vec![
                        Box::new(Node::Variable(Vec::new(), "w".to_string(), 2.into())),
                        Box::new(Node::Constant(4.0)),
                    ])),
                ],
                None,
            )),
        ]));

        let evaluator = ExprEvaluator::new().with_variables(3);
        evaluator.const_visit(base).unwrap();

        assert_eq!(evaluator.variables().get(0).unwrap(), &Value::Number(2.0));
        assert_eq!(evaluator.variables().get(1).unwrap(), &Value::Null);
        assert_eq!(evaluator.variables().get(2).unwrap(), &Value::Null);
    }
}

#[cfg(test)]
mod expr_evaluator_tests {
    use crate::{
        nodes::{
            evaluator::Value,
            indexer::EventIndexer,
            traits::{NodeConstVisitor, NodeVisitor},
        },
        parsers::{lexer::Lexer, parser::Parser},
    };

    use super::ExprEvaluator;

    #[test]
    fn test_simple_addition() {
        let script = "
            x = 1;
            y = 2;
            z = x + y;
        "
        .to_string();

        let tokens = Lexer::new(script).tokenize().unwrap();
        let nodes = Parser::new(tokens).parse().unwrap();

        let indexer = EventIndexer::new();
        indexer.visit(&nodes).unwrap();

        let evaluator = ExprEvaluator::new().with_variables(indexer.get_variables_size());
        evaluator.const_visit(nodes).unwrap();

        assert_eq!(*evaluator.variables().get(0).unwrap(), Value::Number(1.0));
        assert_eq!(*evaluator.variables().get(1).unwrap(), Value::Number(2.0));
        assert_eq!(*evaluator.variables().get(2).unwrap(), Value::Number(3.0));
    }

    #[test]
    fn test_simple_if_condition() {
        let script = "
            x = 2;
            y = 2;
            z = x + y;
            if x == 1 {
                z = 3;
            }
        "
        .to_string();

        let tokens = Lexer::new(script).tokenize().unwrap();
        let nodes = Parser::new(tokens).parse().unwrap();

        let indexer = EventIndexer::new();
        indexer.visit(&nodes).unwrap();

        let evaluator = ExprEvaluator::new().with_variables(indexer.get_variables_size());
        evaluator.const_visit(nodes).unwrap();

        assert_eq!(*evaluator.variables().get(0).unwrap(), Value::Number(2.0));
        assert_eq!(*evaluator.variables().get(1).unwrap(), Value::Number(2.0));
        assert_eq!(*evaluator.variables().get(2).unwrap(), Value::Number(4.0));
    }

    #[test]
    fn test_if_else_condition() {
        let script = "
            x = 2;
            y = 2;
            z = x + y;
            if x == 1 {
                z = 3;
            } else {
                z = 4;
            }
        "
        .to_string();

        let tokens = Lexer::new(script).tokenize().unwrap();
        let nodes = Parser::new(tokens).parse().unwrap();

        let indexer = EventIndexer::new();
        indexer.visit(&nodes).unwrap();

        let evaluator = ExprEvaluator::new().with_variables(indexer.get_variables_size());
        evaluator.const_visit(nodes).unwrap();

        assert_eq!(*evaluator.variables().get(0).unwrap(), Value::Number(2.0));
        assert_eq!(*evaluator.variables().get(1).unwrap(), Value::Number(2.0));
        assert_eq!(*evaluator.variables().get(2).unwrap(), Value::Number(4.0));
    }

    #[test]
    fn test_nested_if_else_conditions() {
        let script = "
            x = 2;
            y = 2;
            z = x + y;
            if x == 1 {
                z = 3;
            } else {
                if y == 1 {
                    z = 4;
                } else {
                    z = 5;
                }
            }
        "
        .to_string();

        let tokens = Lexer::new(script).tokenize().unwrap();
        let nodes = Parser::new(tokens).parse().unwrap();

        let indexer = EventIndexer::new();
        indexer.visit(&nodes).unwrap();

        let evaluator = ExprEvaluator::new().with_variables(indexer.get_variables_size());
        evaluator.const_visit(nodes).unwrap();

        assert_eq!(*evaluator.variables().get(0).unwrap(), Value::Number(2.0));
        assert_eq!(*evaluator.variables().get(1).unwrap(), Value::Number(2.0));
        assert_eq!(*evaluator.variables().get(2).unwrap(), Value::Number(5.0));
    }

    #[test]
    fn test_new_variable_in_if_condition() {
        let script = "
            x = 2;
            y = 2;
            z = x + y;
            if x == 1 {
                z = 3;
                w = 4;
            }
        "
        .to_string();

        let tokens = Lexer::new(script).tokenize().unwrap();
        let nodes = Parser::new(tokens).parse().unwrap();

        let indexer = EventIndexer::new();
        indexer.visit(&nodes).unwrap();

        let evaluator = ExprEvaluator::new().with_variables(indexer.get_variables_size());
        evaluator.const_visit(nodes).unwrap();

        assert_eq!(*evaluator.variables().get(0).unwrap(), Value::Number(2.0));
        assert_eq!(*evaluator.variables().get(1).unwrap(), Value::Number(2.0));
        assert_eq!(*evaluator.variables().get(2).unwrap(), Value::Number(4.0));
        assert_eq!(*evaluator.variables().get(3).unwrap(), Value::Null);

        let script = "
            x = 2;
            y = 2;
            z = x + y;
            if x == 2 {
                z = 3;
                w = 4;
            }
        "
        .to_string();

        let tokens = Lexer::new(script).tokenize().unwrap();
        let nodes = Parser::new(tokens).parse().unwrap();

        let indexer = EventIndexer::new();
        indexer.visit(&nodes).unwrap();

        let evaluator = ExprEvaluator::new().with_variables(indexer.get_variables_size());
        evaluator.const_visit(nodes).unwrap();

        assert_eq!(*evaluator.variables().get(0).unwrap(), Value::Number(2.0));
        assert_eq!(*evaluator.variables().get(1).unwrap(), Value::Number(2.0));
        assert_eq!(*evaluator.variables().get(2).unwrap(), Value::Number(3.0));
        assert_eq!(*evaluator.variables().get(3).unwrap(), Value::Number(4.0));
    }

    #[test]
    fn test_nested_if_else_conditions_2() {
        let script = "
            x = 2;
            y = 2;
            z = x + y;
            if x == 1 {
                z = 3;
            }
            if y == 1 {
                z = 4;
            } else {
                z = 5;
            }
        "
        .to_string();

        let tokens = Lexer::new(script).tokenize().unwrap();
        let nodes = Parser::new(tokens).parse().unwrap();

        let indexer = EventIndexer::new();
        indexer.visit(&nodes).unwrap();

        let evaluator = ExprEvaluator::new().with_variables(indexer.get_variables_size());
        evaluator.const_visit(nodes).unwrap();

        assert_eq!(*evaluator.variables().get(0).unwrap(), Value::Number(2.0));
        assert_eq!(*evaluator.variables().get(1).unwrap(), Value::Number(2.0));
        assert_eq!(*evaluator.variables().get(2).unwrap(), Value::Number(5.0));
    }

    #[test]
    fn test_string_assignment() {
        let script = "
            x = \"Hello world\";
        "
        .to_string();

        let tokens = Lexer::new(script).tokenize().unwrap();
        let nodes = Parser::new(tokens).parse().unwrap();

        let indexer = EventIndexer::new();
        indexer.visit(&nodes).unwrap();

        let evaluator = ExprEvaluator::new().with_variables(indexer.get_variables_size());
        evaluator.const_visit(nodes).unwrap();

        assert_eq!(
            *evaluator.variables().get(0).unwrap(),
            Value::String("Hello world".to_string())
        );
    }

    #[test]
    fn test_variable_reassignment() {
        let script = "
            x = 1;
            x = \"String\";
        "
        .to_string();

        let tokens = Lexer::new(script).tokenize().unwrap();
        let nodes = Parser::new(tokens).parse().unwrap();

        let indexer = EventIndexer::new();
        indexer.visit(&nodes).unwrap();

        let evaluator = ExprEvaluator::new().with_variables(indexer.get_variables_size());
        evaluator.const_visit(nodes).unwrap();

        assert_eq!(
            *evaluator.variables().get(0).unwrap(),
            Value::String("String".to_string())
        );
    }

    #[test]
    fn test_boolean_assignment_from_expression() {
        let script = "
            x = 1 < 2;
        "
        .to_string();

        let tokens = Lexer::new(script).tokenize().unwrap();
        let nodes = Parser::new(tokens).parse().unwrap();

        let indexer = EventIndexer::new();
        indexer.visit(&nodes).unwrap();

        let evaluator = ExprEvaluator::new().with_variables(indexer.get_variables_size());
        evaluator.const_visit(nodes).unwrap();

        assert_eq!(
            *evaluator.variables().get(0).unwrap(),
            Value::Bool(true)
        );
    }
}

#[cfg(test)]
mod event_stream_evaluator_tests {
    use super::*;

    #[test]
    fn test_event_stream_evaluator() {
        let event = "
            x = 1;
            y = 2;
            z = x + y;
        "
        .to_string();
        let event_date = Date::new(2021, 1, 1);
        let expr = event.try_into().unwrap();
        let event = Event::new(event_date, expr);
        let events = EventStream::new().with_events(vec![event]);
        // Index expressions and initialize evaluator (adjust according to your actual logic)
        let indexer = EventIndexer::new();
        indexer.visit_events(&events).unwrap();
        let var_map = indexer.get_variable_indexes();

        let scenarios = vec![Scenario::new()];
        let evaluator =
            EventStreamEvaluator::new(indexer.get_variables_size()).with_scenarios(&scenarios);
        let results = evaluator.visit_events(&events, &var_map).unwrap();

        assert_eq!(results.get("x"), Some(&Value::Number(1.0)));
        assert_eq!(results.get("y"), Some(&Value::Number(2.0)));
        assert_eq!(results.get("z"), Some(&Value::Number(3.0)));
    }

    #[test]
    fn test_event_stream_evaluator_multiple_scenarios() {
        let event = "
            x = 1;
            y = 2;
            z = x + y;
        "
        .to_string();
        let event_date = Date::new(2021, 1, 1);
        let expr = event.try_into().unwrap();
        let event = Event::new(event_date, expr);
        let events = EventStream::new().with_events(vec![event]);
        // Index expressions and initialize evaluator (adjust according to your actual logic)
        let indexer = EventIndexer::new();
        indexer.visit_events(&events).unwrap();

        let var_map = indexer.get_variable_indexes();

        let scenarios = vec![Scenario::new(); 10];
        let evaluator =
            EventStreamEvaluator::new(indexer.get_variables_size()).with_scenarios(&scenarios);
        let results = evaluator.visit_events(&events, &var_map).unwrap();

        assert_eq!(results.get("x"), Some(&Value::Number(1.0)));
        assert_eq!(results.get("y"), Some(&Value::Number(2.0)));
        assert_eq!(results.get("z"), Some(&Value::Number(3.0)));
    }
}

#[cfg(test)]
mod ai_gen_tests {
    use super::*;
    #[test]
    fn test_unary_plus_node() {
        // Test the UnaryPlus node to ensure it correctly processes the value without changing it.
        let mut base = Box::new(Node::new_base());
        let mut unary_plus = Box::new(Node::new_unary_plus());

        let c1 = Node::new_constant(1.0);

        unary_plus.add_child(Box::new(c1));
        base.add_child(unary_plus);

        let evaluator = ExprEvaluator::new();
        evaluator.const_visit(base).unwrap();

        assert_eq!(evaluator.digit_stack().pop().unwrap(), 1.0);
    }

    #[test]
    fn test_unary_minus_node() {
        // Test the UnaryMinus node to ensure it correctly negates the value.
        let mut base = Box::new(Node::new_base());
        let mut unary_minus = Box::new(Node::new_unary_minus());

        let c1 = Node::new_constant(1.0);

        unary_minus.add_child(Box::new(c1));
        base.add_child(unary_minus);

        let evaluator = ExprEvaluator::new();
        evaluator.const_visit(base).unwrap();

        assert_eq!(evaluator.digit_stack().pop().unwrap(), -1.0);
    }

    #[test]
    fn test_min_node() {
        // Test the Min node to ensure it correctly finds the minimum value between two constants.
        let mut base = Box::new(Node::new_base());
        let mut min = Box::new(Node::new_min());

        let c1 = Node::new_constant(1.0);
        let c2 = Node::new_constant(2.0);

        min.add_child(Box::new(c1));
        min.add_child(Box::new(c2));
        base.add_child(min);

        let evaluator = ExprEvaluator::new();
        evaluator.const_visit(base).unwrap();

        assert_eq!(evaluator.digit_stack().pop().unwrap(), 1.0);
    }

    #[test]
    fn test_max_node() {
        // Test the Max node to ensure it correctly finds the maximum value between two constants.
        let mut base = Box::new(Node::new_base());
        let mut max = Box::new(Node::new_max());

        let c1 = Node::new_constant(1.0);
        let c2 = Node::new_constant(2.0);

        max.add_child(Box::new(c1));
        max.add_child(Box::new(c2));
        base.add_child(max);

        let evaluator = ExprEvaluator::new();
        evaluator.const_visit(base).unwrap();

        assert_eq!(evaluator.digit_stack().pop().unwrap(), 2.0);
    }

    #[test]
    fn test_pow_node() {
        // Test the Pow node to ensure it correctly calculates the power of one constant to another.
        let mut base = Box::new(Node::new_base());
        let mut pow = Box::new(Node::new_pow());

        let c1 = Node::new_constant(2.0);
        let c2 = Node::new_constant(3.0);

        pow.add_child(Box::new(c1));
        pow.add_child(Box::new(c2));
        base.add_child(pow);

        let evaluator = ExprEvaluator::new();
        evaluator.const_visit(base).unwrap();

        assert_eq!(evaluator.digit_stack().pop().unwrap(), 8.0);
    }

    #[test]
    fn test_ln_node() {
        // Test the Ln node to ensure it correctly calculates the natural logarithm of a constant.
        let mut base = Box::new(Node::new_base());
        let mut ln = Box::new(Node::new_ln());

        let c1 = Node::new_constant(2.718281828459045);

        ln.add_child(Box::new(c1));
        base.add_child(ln);

        let evaluator = ExprEvaluator::new();
        evaluator.const_visit(base).unwrap();

        assert!((evaluator.digit_stack().pop().unwrap() - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_exp_node() {
        // Test the Exp node to ensure it correctly calculates the exponential of a constant.
        let mut base = Box::new(Node::new_base());
        let mut exp = Box::new(Node::new_exp());

        let c1 = Node::new_constant(1.0);

        exp.add_child(Box::new(c1));
        base.add_child(exp);

        let evaluator = ExprEvaluator::new();
        evaluator.const_visit(base).unwrap();

        assert!((evaluator.digit_stack().pop().unwrap() - 2.718281828459045).abs() < f64::EPSILON);
    }

    #[test]
    fn test_cvg_node() {
        let mut base = Box::new(Node::new_base());
        let mut cvg = Box::new(Node::new_cvg());
        cvg.add_child(Box::new(Node::String("2020-01-01".to_string())));
        cvg.add_child(Box::new(Node::String("2020-06-01".to_string())));
        cvg.add_child(Box::new(Node::String("Actual360".to_string())));
        base.add_child(cvg);

        let evaluator = ExprEvaluator::new();
        evaluator.const_visit(base).unwrap();

        assert!((evaluator.digit_stack().pop().unwrap() - (152.0 / 360.0)).abs() < f64::EPSILON);
    }

    #[test]
    fn test_cvg_with_pays() {
        let mut base = Box::new(Node::new_base());
        let mut pays = Box::new(Node::new_pays());
        let mut cvg = Box::new(Node::new_cvg());
        cvg.add_child(Box::new(Node::String("2020-01-01".to_string())));
        cvg.add_child(Box::new(Node::String("2020-06-01".to_string())));
        cvg.add_child(Box::new(Node::String("Actual360".to_string())));
        pays.add_child(cvg);
        base.add_child(pays);

        let event_date = Date::from_str("2020-06-01", "%Y-%m-%d").unwrap();
        let scenario = vec![MarketData::new(0, event_date, None, None, None, 2.0)];

        let indexer = EventIndexer::new();
        indexer.visit(&base).unwrap();

        let evaluator = ExprEvaluator::new().with_scenario(&scenario);
        evaluator.const_visit(base).unwrap();

        assert!(
            (evaluator.digit_stack().pop().unwrap() - (152.0 / 360.0) / 2.0).abs() < f64::EPSILON
        );
    }

    #[test]
    fn test_pays_node_discount() {
        // Pays should discount the evaluated value by the scenario numerarie
        let mut base = Box::new(Node::new_base());
        let mut pays = Box::new(Node::new_pays());
        pays.add_child(Box::new(Node::new_constant(100.0)));
        base.add_child(pays);

        let event_date = Date::new(2024, 1, 1);
        let scenario = vec![MarketData::new(0, event_date, None, None, None, 2.0)];

        let indexer = EventIndexer::new();
        indexer.visit(&base).unwrap();

        let evaluator = ExprEvaluator::new().with_scenario(&scenario);
        evaluator.const_visit(base).unwrap();

        assert_eq!(evaluator.digit_stack().pop().unwrap(), 50.0);
    }

    #[test]
    fn test_rate_index_eval() {
        let mut base = Box::new(Node::new_base());
        let rate = Node::new_rate_index(
            "0".to_string(),
            Date::new(2024, 1, 1),
            Date::new(2024, 2, 1),
        );
        base.add_child(Box::new(rate));

        let scenario = vec![MarketData::new(
            0,
            Date::new(2024, 1, 1),
            None,
            Some(0.05),
            None,
            1.0,
        )];

        let indexer = EventIndexer::new();
        indexer.visit(&base).unwrap();

        let evaluator = ExprEvaluator::new().with_scenario(&scenario);
        evaluator.const_visit(base).unwrap();

        assert!((evaluator.digit_stack().pop().unwrap() - 0.05).abs() < f64::EPSILON);
    }

    #[test]
    fn test_not_equal_node() {
        // Test the NotEqual node to ensure it correctly evaluates the inequality of two constants.
        let mut base = Box::new(Node::new_base());
        let mut not_equal = Box::new(Node::new_not_equal());

        let c1 = Node::new_constant(1.0);
        let c2 = Node::new_constant(2.0);

        not_equal.add_child(Box::new(c1));
        not_equal.add_child(Box::new(c2));
        base.add_child(not_equal);

        let evaluator = ExprEvaluator::new();
        evaluator.const_visit(base).unwrap();

        assert_eq!(evaluator.boolean_stack().pop().unwrap(), true);
    }

    #[test]
    fn test_add_assign_number() {
        // Test the AddAssign trait for Value::Number to ensure it correctly adds two numbers.
        let mut a = Value::Number(1.0);
        let b = Value::Number(2.0);
        a += b;
        assert_eq!(a, Value::Number(3.0));
    }

    #[test]
    fn test_add_assign_string() {
        // Test the AddAssign trait for Value::String to ensure it correctly concatenates two strings.
        let mut a = Value::String("Hello".to_string());
        let b = Value::String(" World".to_string());
        a += b;
        assert_eq!(a, Value::String("Hello World".to_string()));
    }

    #[test]
    fn test_sub_assign_number() {
        // Test the SubAssign trait for Value::Number to ensure it correctly subtracts two numbers.
        let mut a = Value::Number(3.0);
        let b = Value::Number(1.0);
        a -= b;
        assert_eq!(a, Value::Number(2.0));
    }

    #[test]
    fn test_add_number_and_string() {
        // Test the Add trait for Value to ensure it returns Value::Null when adding a number and a string.
        let a = Value::Number(1.0);
        let b = Value::String("Hello".to_string());
        let result = a + b;
        assert_eq!(result, Value::Null);
    }

    #[test]
    fn test_sub_number_and_string() {
        // Test the Sub trait for Value to ensure it returns Value::Null when subtracting a string from a number.
        let a = Value::Number(1.0);
        let b = Value::String("Hello".to_string());
        let result = a - b;
        assert_eq!(result, Value::Null);
    }

    #[test]
    fn test_mul_number_and_string() {
        // Test the Mul trait for Value to ensure it returns Value::Null when multiplying a number and a string.
        let a = Value::Number(1.0);
        let b = Value::String("Hello".to_string());
        let result = a * b;
        assert_eq!(result, Value::Null);
    }

    #[test]
    fn test_div_number_and_string() {
        // Test the Div trait for Value to ensure it returns Value::Null when dividing a number by a string.
        let a = Value::Number(1.0);
        let b = Value::String("Hello".to_string());
        let result = a / b;
        assert_eq!(result, Value::Null);
    }

    #[test]
    fn test_add_bool_and_number() {
        // Test the Add trait for Value to ensure it returns Value::Null when adding a boolean and a number.
        let a = Value::Bool(true);
        let b = Value::Number(1.0);
        let result = a + b;
        assert_eq!(result, Value::Null);
    }

    #[test]
    fn test_sub_bool_and_number() {
        // Test the Sub trait for Value to ensure it returns Value::Null when subtracting a number from a boolean.
        let a = Value::Bool(true);
        let b = Value::Number(1.0);
        let result = a - b;
        assert_eq!(result, Value::Null);
    }

    #[test]
    fn test_mul_bool_and_number() {
        // Test the Mul trait for Value to ensure it returns Value::Null when multiplying a boolean and a number.
        let a = Value::Bool(true);
        let b = Value::Number(1.0);
        let result = a * b;
        assert_eq!(result, Value::Null);
    }

    #[test]
    fn test_div_bool_and_number() {
        // Test the Div trait for Value to ensure it returns Value::Null when dividing a boolean by a number.
        let a = Value::Bool(true);
        let b = Value::Number(1.0);
        let result = a / b;
        assert_eq!(result, Value::Null);
    }

    #[test]
    fn test_add_null_and_number() {
        // Test the Add trait for Value to ensure it returns Value::Null when adding a null and a number.
        let a = Value::Null;
        let b = Value::Number(1.0);
        let result = a + b;
        assert_eq!(result, Value::Null);
    }

    #[test]
    fn test_sub_null_and_number() {
        // Test the Sub trait for Value to ensure it returns Value::Null when subtracting a number from a null.
        let a = Value::Null;
        let b = Value::Number(1.0);
        let result = a - b;
        assert_eq!(result, Value::Null);
    }

    #[test]
    fn test_mul_null_and_number() {
        // Test the Mul trait for Value to ensure it returns Value::Null when multiplying a null and a number.
        let a = Value::Null;
        let b = Value::Number(1.0);
        let result = a * b;
        assert_eq!(result, Value::Null);
    }

    #[test]
    fn test_div_null_and_number() {
        // Test the Div trait for Value to ensure it returns Value::Null when dividing a null by a number.
        let a = Value::Null;
        let b = Value::Number(1.0);
        let result = a / b;
        assert_eq!(result, Value::Null);
    }

    #[test]
    fn test_event_stream_evaluator_no_scenarios() {
        // Test the EventStreamEvaluator to ensure it returns an error when no scenarios are set.
        let evaluator = EventStreamEvaluator::new(1);
        let event_stream = EventStream::new();
        let var_map: HashMap<String, usize> = HashMap::new();
        let result = evaluator.visit_events(&event_stream, &var_map);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            ScriptingError::EvaluationError("No scenarios set".to_string()).to_string()
        );
    }

    #[test]
    fn test_expr_evaluator_with_variables() {
        // Test the ExprEvaluator to ensure it correctly resizes the variables.
        let evaluator = ExprEvaluator::new().with_variables(3);
        assert_eq!(evaluator.variables().len(), 3);
        assert_eq!(
            evaluator.variables(),
            vec![Value::Null, Value::Null, Value::Null]
        );
    }

    #[test]
    fn test_expr_evaluator_digit_stack() {
        // Test the ExprEvaluator to ensure it correctly returns the digit stack.
        let evaluator = ExprEvaluator::new();
        evaluator.digit_stack.lock().unwrap().push(1.0);
        assert_eq!(evaluator.digit_stack(), vec![1.0]);
    }

    #[test]
    fn test_expr_evaluator_boolean_stack() {
        // Test the ExprEvaluator to ensure it correctly returns the boolean stack.
        let evaluator = ExprEvaluator::new();
        evaluator.boolean_stack.lock().unwrap().push(true);
        assert_eq!(evaluator.boolean_stack(), vec![true]);
    }

    #[test]
    fn test_expr_evaluator_is_lhs_variable() {
        // Test the ExprEvaluator to ensure it correctly sets and gets the is_lhs_variable flag.
        let evaluator = ExprEvaluator::new();
        *evaluator.is_lhs_variable.lock().unwrap() = true;
        assert_eq!(*evaluator.is_lhs_variable.lock().unwrap(), true);
    }

    #[test]
    fn test_expr_evaluator_lhs_variable() {
        // Test the ExprEvaluator to ensure it correctly sets and gets the lhs_variable.
        let evaluator = ExprEvaluator::new();
        let node = Box::new(Node::new_constant(1.0));
        *evaluator.lhs_variable.lock().unwrap() = Some(node.clone());
        assert_eq!(*evaluator.lhs_variable.lock().unwrap(), Some(node));
    }

    #[test]
    fn test_expr_evaluator_with_scenario_none() {
        // Test the ExprEvaluator to ensure it correctly handles None scenario.
        let evaluator = ExprEvaluator::new();
        assert!(evaluator.scenario.is_none());
    }

    #[test]
    fn test_event_stream_evaluator_with_scenarios() {
        // Test the EventStreamEvaluator to ensure it correctly evaluates with scenarios set.
        let scenario: Scenario = vec![];
        let scenarios = vec![scenario];
        let evaluator = EventStreamEvaluator::new(1).with_scenarios(&scenarios);
        let event_stream = EventStream::new();
        let var_map: HashMap<String, usize> = HashMap::new();
        let result = evaluator.visit_events(&event_stream, &var_map);
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }
}
