use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use rustatlas::prelude::*;
use serde::{Deserialize, Serialize};

use std::{
    cell::RefCell,
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
pub enum Value {
    Bool(bool),
    Number(NumericType),
    String(String),
    Array(Vec<Value>),
    Null,
}

impl Add for Value {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        match (self, other) {
            (Value::Number(a), Value::Number(b)) => Value::Number((a + b).into()),
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
            (Value::Number(a), Value::Number(b)) => Value::Number((a - b).into()),
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
            (Value::Number(a), Value::Number(b)) => Value::Number((a * b).into()),
            _ => Value::Null,
        }
    }
}

impl Div for Value {
    type Output = Self;

    fn div(self, other: Self) -> Self {
        match (self, other) {
            (Value::Number(a), Value::Number(b)) => Value::Number((a / b).into()),
            _ => Value::Null,
        }
    }
}

/// # SingleScenarioEvaluator
/// Visitor that evaluates the expression tree
pub struct SingleScenarioEvaluator<'a> {
    variables: RefCell<Vec<Value>>,
    digit_stack: RefCell<Vec<NumericType>>,
    boolean_stack: RefCell<Vec<bool>>,
    string_stack: RefCell<Vec<String>>,
    array_stack: RefCell<Vec<Vec<Value>>>,
    is_lhs_variable: RefCell<bool>,
    lhs_variable: RefCell<Option<Box<Node>>>,
    scenario: Option<&'a Scenario>,
    current_event: RefCell<usize>,
}

impl<'a> SingleScenarioEvaluator<'a> {
    pub fn new() -> Self {
        SingleScenarioEvaluator {
            variables: RefCell::new(Vec::new()),
            digit_stack: RefCell::new(Vec::new()),
            boolean_stack: RefCell::new(Vec::new()),
            string_stack: RefCell::new(Vec::new()),
            array_stack: RefCell::new(Vec::new()),
            is_lhs_variable: RefCell::new(false),
            lhs_variable: RefCell::new(None),
            scenario: None,
            current_event: RefCell::new(0),
        }
    }

    pub fn with_scenario(mut self, scenario: &'a Scenario) -> Self {
        self.scenario = Some(scenario);
        self
    }

    pub fn with_variables(self, n: usize) -> Self {
        self.variables.borrow_mut().resize(n, Value::Null);
        self
    }

    pub fn with_current_event(self, event: usize) -> Self {
        *self.current_event.borrow_mut() = event;
        self
    }

    pub fn current_event(&self) -> usize {
        *self.current_event.borrow_mut()
    }

    pub fn set_current_event(&self, event: usize) {
        *self.current_event.borrow_mut() = event;
    }

    pub fn variables(&self) -> Vec<Value> {
        self.variables.borrow_mut().clone()
    }

    pub fn set_variable(&self, idx: usize, val: Value) {
        let mut vars = self.variables.borrow_mut();
        if idx >= vars.len() {
            vars.resize(idx + 1, Value::Null);
        }
        vars[idx] = val;
    }

    pub fn digit_stack(&self) -> Vec<NumericType> {
        self.digit_stack.borrow_mut().clone()
    }

    pub fn boolean_stack(&self) -> Vec<bool> {
        self.boolean_stack.borrow_mut().clone()
    }
}

impl<'a> NodeConstVisitor for SingleScenarioEvaluator<'a> {
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
                if *self.is_lhs_variable.borrow_mut() {
                    *self.lhs_variable.borrow_mut() = Some(node.clone());
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
                            let vars = self.variables.borrow_mut();
                            let value = vars.get(*id).unwrap();
                            match value {
                                Value::Number(v) => self.digit_stack.borrow_mut().push(*v),
                                Value::Bool(v) => self.boolean_stack.borrow_mut().push(*v),
                                Value::String(v) => self.string_stack.borrow_mut().push(v.clone()),
                                Value::Array(a) => self.array_stack.borrow_mut().push(a.clone()),
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
            Node::Spot(_, _, _, index) => {
                let id = index.get().ok_or(ScriptingError::EvaluationError(
                    "Spot not indexed".to_string(),
                ))?;

                let market_data = self
                    .scenario
                    .ok_or(ScriptingError::EvaluationError(
                        "No scenario set".to_string(),
                    ))?
                    .get(*self.current_event.borrow_mut())
                    .ok_or(ScriptingError::EvaluationError(
                        "Spot not found".to_string(),
                    ))?;

                self.digit_stack.borrow_mut().push(market_data.get_fx(*id)?);
                Ok(())
            }
            Node::Df(_, _, index) => {
                let id = index.get().ok_or(ScriptingError::EvaluationError(
                    "Df not indexed".to_string(),
                ))?;

                let market_data = self
                    .scenario
                    .ok_or(ScriptingError::EvaluationError(
                        "No scenario set".to_string(),
                    ))?
                    .get(*self.current_event.borrow_mut())
                    .ok_or(ScriptingError::EvaluationError("Df not found".to_string()))?;

                self.digit_stack.borrow_mut().push(market_data.get_df(*id)?);
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
                    .get(*self.current_event.borrow_mut())
                    .ok_or(ScriptingError::EvaluationError(
                        "RateIndex not found".to_string(),
                    ))?;

                self.digit_stack
                    .borrow_mut()
                    .push(market_data.get_fwd(*id)?);
                Ok(())
            }
            Node::Pays(children, _, currency, df_index, fx_index) => {
                children
                    .iter()
                    .try_for_each(|child| self.const_visit(child.clone()))?;

                let market_data = self
                    .scenario
                    .ok_or(ScriptingError::EvaluationError(
                        "No scenario set".to_string(),
                    ))?
                    .get(*self.current_event.borrow_mut())
                    .ok_or(ScriptingError::EvaluationError(
                        "Event not found".to_string(),
                    ))?
                    .clone();

                let current_value = self.digit_stack.borrow_mut().pop().unwrap();
                let df_id = df_index.get().ok_or(ScriptingError::EvaluationError(
                    "Pays not indexed".to_string(),
                ))?;
                let df = market_data.get_df(*df_id)?;
                let numerarie = market_data.numerarie();

                let value: NumericType = if let Some(_) = currency {
                    let fx_id = fx_index.get().ok_or(ScriptingError::EvaluationError(
                        "Pays FX not indexed".to_string(),
                    ))?;
                    let fx = market_data.get_fx(*fx_id)?;
                    ((current_value * df * fx) / numerarie).into()
                } else {
                    ((current_value * df) / numerarie).into()
                };

                self.digit_stack.borrow_mut().push(value);
                Ok(())
            }
            Node::Constant(value) => {
                self.digit_stack.borrow_mut().push(*value);
                Ok(())
            }
            Node::String(value) => {
                self.string_stack.borrow_mut().push(value.clone());
                Ok(())
            }
            Node::Add(children) => {
                children
                    .iter()
                    .try_for_each(|child| self.const_visit(child.clone()))?;

                let right = self.digit_stack.borrow_mut().pop().unwrap();
                let left = self.digit_stack.borrow_mut().pop().unwrap();
                self.digit_stack.borrow_mut().push((left + right).into());
                Ok(())
            }
            Node::Subtract(children) => {
                children
                    .iter()
                    .try_for_each(|child| self.const_visit(child.clone()))?;

                let right = self.digit_stack.borrow_mut().pop().unwrap();
                let left = self.digit_stack.borrow_mut().pop().unwrap();
                self.digit_stack.borrow_mut().push((left - right).into());
                Ok(())
            }
            Node::Multiply(children) => {
                children
                    .iter()
                    .try_for_each(|child| self.const_visit(child.clone()))?;

                let right = self.digit_stack.borrow_mut().pop().unwrap();
                let left = self.digit_stack.borrow_mut().pop().unwrap();
                self.digit_stack.borrow_mut().push((left * right).into());
                Ok(())
            }
            Node::Divide(children) => {
                children
                    .iter()
                    .try_for_each(|child| self.const_visit(child.clone()))?;

                let right = self.digit_stack.borrow_mut().pop().unwrap();
                let left = self.digit_stack.borrow_mut().pop().unwrap();
                self.digit_stack.borrow_mut().push((left / right).into());
                Ok(())
            }
            Node::Assign(children) => {
                *self.is_lhs_variable.borrow_mut() = true;
                self.const_visit(children.get(0).unwrap().clone())?;

                *self.is_lhs_variable.borrow_mut() = false;
                self.const_visit(children.get(1).unwrap().clone())?;

                let v = self.lhs_variable.borrow_mut().clone().unwrap();
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
                            let mut variables = self.variables.borrow_mut();
                            if !self.boolean_stack.borrow_mut().is_empty() {
                                let value = self.boolean_stack.borrow_mut().pop().unwrap();
                                variables[*id] = Value::Bool(value);
                                Ok(())
                            } else if !self.string_stack.borrow_mut().is_empty() {
                                let value = self.string_stack.borrow_mut().pop().unwrap();
                                variables[*id] = Value::String(value);
                                Ok(())
                            } else if !self.array_stack.borrow_mut().is_empty() {
                                let value = self.array_stack.borrow_mut().pop().unwrap();
                                variables[*id] = Value::Array(value);
                                Ok(())
                            } else {
                                let value = self.digit_stack.borrow_mut().pop().unwrap();
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

                let right = self.digit_stack.borrow_mut().pop().unwrap();
                let left = self.digit_stack.borrow_mut().pop().unwrap();
                self.boolean_stack
                    .borrow_mut()
                    .push((right - left).abs() >= f64::EPSILON);

                Ok(())
            }
            Node::And(children) => {
                children
                    .iter()
                    .try_for_each(|child| self.const_visit(child.clone()))?;

                let right = self.boolean_stack.borrow_mut().pop().unwrap();
                let left = self.boolean_stack.borrow_mut().pop().unwrap();
                self.boolean_stack.borrow_mut().push(left && right);

                Ok(())
            }
            Node::Or(children) => {
                children
                    .iter()
                    .try_for_each(|child| self.const_visit(child.clone()))?;

                let right = self.boolean_stack.borrow_mut().pop().unwrap();
                let left = self.boolean_stack.borrow_mut().pop().unwrap();
                self.boolean_stack.borrow_mut().push(left || right);

                Ok(())
            }
            Node::Not(children) => {
                children
                    .iter()
                    .try_for_each(|child| self.const_visit(child.clone()))?;

                let value = self.boolean_stack.borrow_mut().pop().unwrap();
                self.boolean_stack.borrow_mut().push(!value);

                Ok(())
            }
            Node::Superior(children) => {
                children
                    .iter()
                    .try_for_each(|child| self.const_visit(child.clone()))?;

                let right = self.digit_stack.borrow_mut().pop().unwrap();
                let left = self.digit_stack.borrow_mut().pop().unwrap();
                self.boolean_stack.borrow_mut().push(left > right);

                Ok(())
            }
            Node::Inferior(children) => {
                children
                    .iter()
                    .try_for_each(|child| self.const_visit(child.clone()))?;

                let right = self.digit_stack.borrow_mut().pop().unwrap();
                let left = self.digit_stack.borrow_mut().pop().unwrap();
                self.boolean_stack.borrow_mut().push(left < right);

                Ok(())
            }
            Node::SuperiorOrEqual(children) => {
                children
                    .iter()
                    .try_for_each(|child| self.const_visit(child.clone()))?;

                let right = self.digit_stack.borrow_mut().pop().unwrap();
                let left = self.digit_stack.borrow_mut().pop().unwrap();
                self.boolean_stack.borrow_mut().push(left >= right);

                Ok(())
            }
            Node::InferiorOrEqual(children) => {
                children
                    .iter()
                    .try_for_each(|child| self.const_visit(child.clone()))?;

                let right = self.digit_stack.borrow_mut().pop().unwrap();
                let left = self.digit_stack.borrow_mut().pop().unwrap();
                self.boolean_stack.borrow_mut().push(left <= right);

                Ok(())
            }
            Node::True => {
                self.boolean_stack.borrow_mut().push(true);
                Ok(())
            }
            Node::False => {
                self.boolean_stack.borrow_mut().push(false);
                Ok(())
            }
            Node::Equal(children) => {
                children
                    .iter()
                    .try_for_each(|child| self.const_visit(child.clone()))?;

                let right = self.digit_stack.borrow_mut().pop().unwrap();
                let left = self.digit_stack.borrow_mut().pop().unwrap();

                self.boolean_stack
                    .borrow_mut()
                    .push((right - left).abs() < f64::EPSILON);

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

                let top = self.digit_stack.borrow_mut().pop().unwrap();
                self.digit_stack.borrow_mut().push((-top).into());

                Ok(())
            }
            Node::Min(children) => {
                children
                    .iter()
                    .try_for_each(|child| self.const_visit(child.clone()))?;

                let right = self.digit_stack.borrow_mut().pop().unwrap();
                let left = self.digit_stack.borrow_mut().pop().unwrap();
                self.digit_stack.borrow_mut().push(left.min(right).into());

                Ok(())
            }
            Node::Max(children) => {
                children
                    .iter()
                    .try_for_each(|child| self.const_visit(child.clone()))?;

                let right = self.digit_stack.borrow_mut().pop().unwrap();
                let left = self.digit_stack.borrow_mut().pop().unwrap();
                self.digit_stack.borrow_mut().push(left.max(right).into());

                Ok(())
            }

            #[cfg(feature = "adnumber")]
            Node::Pow(children) => {
                children
                    .iter()
                    .try_for_each(|child| self.const_visit(child.clone()))?;

                let right = self.digit_stack.borrow_mut().pop().unwrap();
                let left = self.digit_stack.borrow_mut().pop().unwrap();
                self.digit_stack
                    .borrow_mut()
                    .push(left.pow_expr(right).into());

                Ok(())
            }
            #[cfg(feature = "f64")]
            Node::Pow(children) => {
                children
                    .iter()
                    .try_for_each(|child| self.const_visit(child.clone()))?;

                let right = self.digit_stack.borrow_mut().pop().unwrap();
                let left = self.digit_stack.borrow_mut().pop().unwrap();
                self.digit_stack
                    .lock()
                    .unwrap()
                    .push(left.powf(right).into());

                Ok(())
            }

            Node::Ln(children) => {
                children
                    .iter()
                    .try_for_each(|child| self.const_visit(child.clone()))?;

                let top = self.digit_stack.borrow_mut().pop().unwrap();
                self.digit_stack.borrow_mut().push(top.ln().into());

                Ok(())
            }
            Node::Fif(children) => {
                children
                    .iter()
                    .try_for_each(|child| self.const_visit(child.clone()))?;

                let eps = self.digit_stack.borrow_mut().pop().unwrap();
                let b = self.digit_stack.borrow_mut().pop().unwrap();
                let a = self.digit_stack.borrow_mut().pop().unwrap();
                let x = self.digit_stack.borrow_mut().pop().unwrap();

                let half = eps.clone() * 0.5;
                let inner = (x + half).min(eps.clone()).max(NumericType::zero());
                let res = b.clone() + ((a - b) / eps) * inner;
                self.digit_stack.borrow_mut().push(res.into());

                Ok(())
            }
            Node::Exp(children) => {
                children
                    .iter()
                    .try_for_each(|child| self.const_visit(child.clone()))?;

                let top = self.digit_stack.borrow_mut().pop().unwrap();
                self.digit_stack.borrow_mut().push(top.exp().into());
                Ok(())
            }
            Node::Cvg(children) => {
                children
                    .iter()
                    .try_for_each(|child| self.const_visit(child.clone()))?;

                let basis_str = self.string_stack.borrow_mut().pop().unwrap();
                let end_str = self.string_stack.borrow_mut().pop().unwrap();
                let start_str = self.string_stack.borrow_mut().pop().unwrap();

                let start = Date::from_str(&start_str, "%Y-%m-%d")?;
                let end = Date::from_str(&end_str, "%Y-%m-%d")?;
                let basis = DayCounter::try_from(basis_str)?;
                let yf = basis.year_fraction(start, end);
                self.digit_stack.borrow_mut().push(yf);
                Ok(())
            }
            Node::Append(children) => {
                *self.is_lhs_variable.borrow_mut() = true;
                self.const_visit(children.get(0).unwrap().clone())?;
                *self.is_lhs_variable.borrow_mut() = false;
                self.const_visit(children.get(1).unwrap().clone())?;

                let var_node = self.lhs_variable.borrow_mut().clone().unwrap();
                if let Node::Variable(_, name, idx) = var_node.as_ref() {
                    let id = idx.get().ok_or(ScriptingError::EvaluationError(format!(
                        "Variable {} not indexed",
                        name
                    )))?;
                    let mut vars = self.variables.borrow_mut();
                    let val = if !self.boolean_stack.borrow().is_empty() {
                        Value::Bool(self.boolean_stack.borrow_mut().pop().unwrap())
                    } else if !self.string_stack.borrow().is_empty() {
                        Value::String(self.string_stack.borrow_mut().pop().unwrap())
                    } else if !self.array_stack.borrow().is_empty() {
                        Value::Array(self.array_stack.borrow_mut().pop().unwrap())
                    } else {
                        Value::Number(self.digit_stack.borrow_mut().pop().unwrap())
                    };
                    match vars.get_mut(*id).unwrap() {
                        Value::Array(ref mut arr) => arr.push(val),
                        Value::Null => {
                            *vars.get_mut(*id).unwrap() = Value::Array(vec![val]);
                        }
                        _ => {
                            return Err(ScriptingError::EvaluationError(
                                "Append on non-array".to_string(),
                            ));
                        }
                    }
                    Ok(())
                } else {
                    Err(ScriptingError::EvaluationError(
                        "Invalid append target".to_string(),
                    ))
                }
            }
            Node::Mean(children) => {
                children
                    .iter()
                    .try_for_each(|child| self.const_visit(child.clone()))?;
                let array = self.array_stack.borrow_mut().pop().unwrap_or_default();
                let mut sum = NumericType::new(0.0);
                let mut count = 0.0;
                for v in array {
                    if let Value::Number(n) = v {
                        sum += n;
                        count += 1.0;
                    }
                }
                if count == 0.0 {
                    return Err(ScriptingError::EvaluationError(
                        "mean of empty array".to_string(),
                    ));
                }
                self.digit_stack.borrow_mut().push((sum / count).into());
                Ok(())
            }
            Node::Std(children) => {
                children
                    .iter()
                    .try_for_each(|child| self.const_visit(child.clone()))?;
                let array = self.array_stack.borrow_mut().pop().unwrap_or_default();
                let mut sum = NumericType::new(0.0);
                let mut count = 0.0;
                let mut nums = Vec::new();
                for v in array {
                    if let Value::Number(n) = v {
                        sum += n;
                        nums.push(n);
                        count += 1.0;
                    }
                }
                if count == 0.0 {
                    return Err(ScriptingError::EvaluationError(
                        "std of empty array".to_string(),
                    ));
                }
                let mean = sum / count;
                let mut var = NumericType::new(0.0);
                for n in nums {
                    let diff = n - mean.clone();
                    var += diff.clone() * diff;
                }
                let std = (var / count).sqrt();
                self.digit_stack.borrow_mut().push(std.into());
                Ok(())
            }
            Node::Range(children) => {
                children
                    .iter()
                    .try_for_each(|child| self.const_visit(child.clone()))?;
                let end = self.digit_stack.borrow_mut().pop().unwrap();
                let start = self.digit_stack.borrow_mut().pop().unwrap();
                let mut vec = Vec::new();
                let s = start.value().round() as i64;
                let e = end.value().round() as i64;
                for i in s..=e {
                    vec.push(Value::Number((i as f64).into()));
                }
                self.array_stack.borrow_mut().push(vec);
                Ok(())
            }
            Node::List(children) => {
                let mut array = Vec::new();
                for child in children {
                    self.const_visit(child.clone())?;
                    if !self.boolean_stack.borrow().is_empty() {
                        let v = self.boolean_stack.borrow_mut().pop().unwrap();
                        array.push(Value::Bool(v));
                    } else if !self.string_stack.borrow().is_empty() {
                        let v = self.string_stack.borrow_mut().pop().unwrap();
                        array.push(Value::String(v));
                    } else if !self.array_stack.borrow().is_empty() {
                        let v = self.array_stack.borrow_mut().pop().unwrap();
                        array.push(Value::Array(v));
                    } else {
                        let v = self.digit_stack.borrow_mut().pop().unwrap();
                        array.push(Value::Number(v));
                    }
                }
                self.array_stack.borrow_mut().push(array);
                Ok(())
            }
            Node::Index(children) => {
                children
                    .iter()
                    .try_for_each(|child| self.const_visit(child.clone()))?;
                let idx_val = self.digit_stack.borrow_mut().pop().unwrap();
                let array = self.array_stack.borrow_mut().pop().unwrap_or_default();
                let idx = idx_val.value().round() as usize;
                if idx >= array.len() {
                    return Err(ScriptingError::EvaluationError(
                        "Index out of bounds".to_string(),
                    ));
                }
                match array[idx].clone() {
                    Value::Bool(v) => self.boolean_stack.borrow_mut().push(v),
                    Value::Number(v) => self.digit_stack.borrow_mut().push(v),
                    Value::String(v) => self.string_stack.borrow_mut().push(v),
                    Value::Array(a) => self.array_stack.borrow_mut().push(a),
                    Value::Null => self.array_stack.borrow_mut().push(Vec::new()),
                }
                Ok(())
            }
            Node::ForEach(_, iter, body, index) => {
                iter.const_accept(self);
                let array = self.array_stack.borrow_mut().pop().unwrap_or_default();
                let idx = index.get().ok_or(ScriptingError::EvaluationError(
                    "Loop variable not indexed".to_string(),
                ))?;
                for val in array {
                    self.set_variable(*idx, val);
                    for child in body {
                        child.const_accept(self);
                    }
                }
                Ok(())
            }
            Node::If(children, first_else) => {
                // Evaluate the condition
                children.get(0).unwrap().const_accept(self);
                // Pop the condition result
                let is_true = self.boolean_stack.borrow_mut().pop().unwrap();

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

impl<'a> SingleScenarioEvaluator<'a> {
    pub fn visit_events(
        &self,
        event_stream: &EventStream,
        var_indexes: &HashMap<String, usize>,
    ) -> Result<HashMap<String, Value>> {
        event_stream
            .events()
            .iter()
            .enumerate()
            .try_for_each(|(i, event)| -> Result<()> {
                self.set_current_event(i);
                self.const_visit(event.expr().clone())?;
                Ok(())
            })?;
        let v = self.variables.borrow_mut().clone();
        let mut map = HashMap::new();
        for (name, idx) in var_indexes.iter() {
            if let Some(v) = v.get(*idx) {
                map.insert(name.clone(), v.clone());
            }
        }
        Ok(map)
    }
}

pub struct Evaluator<'a> {
    n_vars: usize,
    scenarios: &'a Vec<Scenario>,
}

impl<'a> Evaluator<'a> {
    pub fn new(n_vars: usize, scenarios: &'a Vec<Scenario>) -> Self {
        Evaluator { n_vars, scenarios }
    }

    pub fn visit_events(
        &self,
        event_stream: &EventStream,
        var_indexes: &HashMap<String, usize>,
    ) -> Result<HashMap<String, Value>> {
        let tmp: Result<Vec<HashMap<String, Value>>> = self
            .scenarios
            .iter()
            .map(|scenario| {
                let evaluator = SingleScenarioEvaluator::new()
                    .with_variables(self.n_vars)
                    .with_scenario(scenario);

                evaluator.visit_events(event_stream, var_indexes)
            })
            .collect();

        let results = tmp?;

        // Combine results from all scenarios
        let n_scenarios = self.scenarios.len() as f64;
        let mut combined_results = HashMap::new();
        results.iter().for_each(|result| {
            result.iter().for_each(|(key, value)| {
                let entry = combined_results
                    .entry(key.clone())
                    .or_insert(Value::Number(NumericType::new(0.0)));
                match (entry, value) {
                    (Value::Number(a), Value::Number(b)) => *a = (*a + *b / n_scenarios).into(),
                    _ => (),
                }
            });
        });
        Ok(combined_results)
    }

    pub fn par_visit_events(
        &self,
        event_stream: &EventStream,
        var_indexes: &HashMap<String, usize>,
    ) -> Result<HashMap<String, Value>> {
        let tmp: Result<Vec<HashMap<String, Value>>> = self
            .scenarios
            .par_iter()
            .map(|scenario| {
                let evaluator = SingleScenarioEvaluator::new()
                    .with_variables(self.n_vars)
                    .with_scenario(scenario);

                evaluator.visit_events(event_stream, var_indexes)
            })
            .collect();

        let results = tmp?;

        // Combine results from all scenarios
        let n_scenarios = self.scenarios.len() as f64;
        let mut combined_results = HashMap::new();
        results.iter().for_each(|result| {
            result.iter().for_each(|(key, value)| {
                let entry = combined_results
                    .entry(key.clone())
                    .or_insert(Value::Number(NumericType::new(0.0)));
                match (entry, value) {
                    (Value::Number(a), Value::Number(b)) => *a = (*a + *b / n_scenarios).into(),
                    _ => (),
                }
            });
        });
        Ok(combined_results)
    }
}

/// # EventStreamEvaluator
/// Visitor that evaluates the event stream
pub struct EventStreamEvaluator<'a> {
    n_vars: usize,
    scenarios: Option<&'a Vec<Scenario>>,
}

impl<'a> EventStreamEvaluator<'a> {
    pub fn new(n_vars: usize) -> Self {
        EventStreamEvaluator {
            n_vars,
            scenarios: None,
        }
    }

    pub fn with_scenarios(mut self, scenarios: &'a Vec<Scenario>) -> Self {
        self.scenarios = Some(scenarios);
        self
    }

    pub fn visit_events(
        &self,
        event_stream: &EventStream,
        var_indexes: &HashMap<String, usize>,
    ) -> Result<HashMap<String, Value>> {
        let scenarios = self.scenarios.ok_or(ScriptingError::EvaluationError(
            "No scenarios set".to_string(),
        ))?;

        // Evaluate the events to get the variables using the first scenario
        let mut evaluator = SingleScenarioEvaluator::new().with_variables(self.n_vars);
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

        let v: Vec<Value> = evaluator
            .variables()
            .iter()
            .map(|v| match v {
                Value::Number(_) => Value::Number(NumericType::new(0.0)),
                _ => v.clone(),
            })
            .collect();

        let global_variables = Mutex::new(v);

        scenarios.iter().try_for_each(|scenario| -> Result<()> {
            let evaluator = SingleScenarioEvaluator::new()
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
                    (Value::Number(g), Value::Number(l)) => *g = (*g + *l).into(),
                    _ => (),
                });

            Ok(())
        })?;

        //avg
        let mut vars = global_variables.lock().unwrap();
        let len = scenarios.len() as f64;

        vars.iter_mut().for_each(|v| match v {
            Value::Number(v) => *v = (*v / len).into(),
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

#[cfg(test)]
mod general_tests {

    use super::*;

    #[test]
    fn test_add_node() {
        let mut base = Box::new(Node::new_base());
        let mut add = Box::new(Node::new_add());

        let c1 = Box::new(Node::new_constant(NumericType::new(1.0)));
        let c2 = Box::new(Node::new_constant(NumericType::new(1.0)));

        add.add_child(c1);
        add.add_child(c2);
        base.add_child(add);

        let evaluator = SingleScenarioEvaluator::new();
        evaluator.const_visit(base).unwrap();

        assert_eq!(evaluator.digit_stack().pop().unwrap(), 2.0);
    }

    #[test]
    fn test_subtract_node() {
        let mut base = Box::new(Node::new_base());
        let mut subtract = Node::new_subtract();

        let c1 = Node::new_constant(NumericType::new(1.0));
        let c2 = Node::new_constant(NumericType::new(1.0));

        subtract.add_child(Box::new(c1));
        subtract.add_child(Box::new(c2));
        base.add_child(Box::new(subtract));

        let evaluator = SingleScenarioEvaluator::new();
        evaluator.const_visit(base).unwrap();

        assert_eq!(evaluator.digit_stack().pop().unwrap(), 0.0);
    }

    #[test]
    fn test_multiply_node() {
        let mut base = Box::new(Node::new_base());
        let mut multiply = Node::new_multiply();

        let c1 = Node::new_constant(NumericType::new(1.0));
        let c2 = Node::new_constant(NumericType::new(2.0));

        multiply.add_child(Box::new(c1));
        multiply.add_child(Box::new(c2));
        base.add_child(Box::new(multiply));

        let evaluator = SingleScenarioEvaluator::new();
        evaluator.const_visit(base).unwrap();

        assert_eq!(evaluator.digit_stack().pop().unwrap(), 2.0);
    }

    #[test]
    fn test_divide_node() {
        let mut base = Box::new(Node::new_base());
        let mut divide = Node::new_divide();

        let c1 = Node::new_constant(NumericType::new(4.0));
        let c2 = Node::new_constant(NumericType::new(2.0));

        divide.add_child(Box::new(c1));
        divide.add_child(Box::new(c2));
        base.add_child(Box::new(divide));

        let evaluator = SingleScenarioEvaluator::new();
        evaluator.const_visit(base).unwrap();

        assert_eq!(evaluator.digit_stack().pop().unwrap(), 2.0);
    }

    #[test]
    fn test_variable_assign_node() {
        let mut base = Box::new(Node::new_base());

        let c1 = Box::new(Node::new_constant(NumericType::new(1.0)));
        let v1 = Box::new(Node::new_variable_with_id("x".to_string(), 0));

        let mut assign = Box::new(Node::new_assign());
        assign.add_child(v1);
        assign.add_child(c1);

        base.add_child(assign);

        let evaluator = SingleScenarioEvaluator::new().with_variables(1);
        evaluator.const_visit(base).unwrap();
        assert_eq!(
            evaluator.variables().pop().unwrap(),
            Value::Number(NumericType::new(1.0))
        );
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

        let evaluator = SingleScenarioEvaluator::new().with_variables(3);
        evaluator.const_visit(base).unwrap();

        assert_eq!(evaluator.variables().get(0).unwrap(), &Value::Bool(true));
        assert_eq!(evaluator.variables().get(1).unwrap(), &Value::Bool(false));
        assert_eq!(evaluator.variables().get(2).unwrap(), &Value::Bool(false));
    }

    #[test]
    fn test_variable_use_node() {
        let mut base = Box::new(Node::new_base());

        let c1 = Box::new(Node::new_constant(NumericType::new(1.0)));
        let v1 = Box::new(Node::new_variable_with_id("x".to_string(), 0));

        let mut add = Box::new(Node::new_add());
        add.add_child(v1);
        add.add_child(c1);

        base.add_child(add);

        let evaluator = SingleScenarioEvaluator::new().with_variables(1);
        assert!(evaluator.const_visit(base).is_err());
    }

    #[test]
    fn test_nested_expression() {
        let mut base = Box::new(Node::new_base());

        let c1 = Box::new(Node::new_constant(NumericType::new(1.0)));
        let c2 = Box::new(Node::new_constant(NumericType::new(2.0)));
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

        let evaluator = SingleScenarioEvaluator::new().with_variables(3);
        evaluator.const_visit(base).unwrap();
        assert_eq!(
            evaluator.variables().pop().unwrap(),
            Value::Number(NumericType::new(3.0))
        );
    }

    #[test]
    fn test_equal() {
        let mut base = Box::new(Node::new_base());

        let c1 = Box::new(Node::new_constant(NumericType::new(1.0)));
        let c2 = Box::new(Node::new_constant(NumericType::new(1.0)));

        let mut equal = Box::new(Node::new_equal());
        equal.add_child(c1);
        equal.add_child(c2);

        base.add_child(equal);

        let evaluator = SingleScenarioEvaluator::new().with_variables(1);
        evaluator.const_visit(base).unwrap();
        assert_eq!(evaluator.boolean_stack().pop().unwrap(), true);
    }

    #[test]
    fn test_superior() {
        let mut base = Box::new(Node::new_base());

        let c1 = Box::new(Node::new_constant(NumericType::new(2.0)));
        let c2 = Box::new(Node::new_constant(NumericType::new(1.0)));

        let mut and = Box::new(Node::new_superior());
        and.add_child(c1);
        and.add_child(c2);

        base.add_child(and);

        let evaluator = SingleScenarioEvaluator::new();
        evaluator.const_visit(base).unwrap();
        assert_eq!(evaluator.boolean_stack().pop().unwrap(), true);
    }

    #[test]
    fn test_inferior() {
        let mut base = Box::new(Node::new_base());

        let c1 = Box::new(Node::new_constant(NumericType::new(1.0)));
        let c2 = Box::new(Node::new_constant(NumericType::new(2.0)));

        let mut and = Box::new(Node::new_inferior());
        and.add_child(c1);
        and.add_child(c2);

        base.add_child(and);

        let evaluator = SingleScenarioEvaluator::new();
        evaluator.const_visit(base).unwrap();

        assert_eq!(evaluator.boolean_stack().pop().unwrap(), true);
    }

    #[test]
    fn test_superior_or_equal() {
        let mut base = Box::new(Node::new_base());

        let c1 = Box::new(Node::new_constant(NumericType::new(2.0)));
        let c2 = Box::new(Node::new_constant(NumericType::new(1.0)));

        let mut and = Box::new(Node::new_superior_or_equal());
        and.add_child(c1);
        and.add_child(c2);

        base.add_child(and);

        let evaluator = SingleScenarioEvaluator::new();
        evaluator.const_visit(base).unwrap();
        assert_eq!(evaluator.boolean_stack().pop().unwrap(), true);
    }

    #[test]
    fn test_inferior_or_equal() {
        let mut base = Box::new(Node::new_base());

        let c1 = Box::new(Node::new_constant(NumericType::new(1.0)));
        let c2 = Box::new(Node::new_constant(NumericType::new(2.0)));

        let mut and = Box::new(Node::new_inferior_or_equal());
        and.add_child(c1);
        and.add_child(c2);

        base.add_child(and);

        let evaluator = SingleScenarioEvaluator::new();
        evaluator.const_visit(base).unwrap();
        assert_eq!(evaluator.boolean_stack().pop().unwrap(), true);
    }

    #[test]
    fn test_and() {
        let mut base = Box::new(Node::new_base());

        let c1 = Box::new(Node::new_constant(NumericType::new(1.0)));
        let c2 = Box::new(Node::new_constant(NumericType::new(1.0)));

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

        let evaluator = SingleScenarioEvaluator::new();
        evaluator.const_visit(base).unwrap();
        assert_eq!(evaluator.boolean_stack().pop().unwrap(), true);
    }

    #[test]
    fn test_or() {
        let mut base = Box::new(Node::new_base());

        let c1 = Box::new(Node::new_constant(NumericType::new(1.0)));
        let c2 = Box::new(Node::new_constant(NumericType::new(1.0)));

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

        let evaluator = SingleScenarioEvaluator::new();
        evaluator.const_visit(base).unwrap();
        assert_eq!(evaluator.boolean_stack().pop().unwrap(), true);
    }

    #[test]
    fn test_not() {
        let mut base = Box::new(Node::new_base());

        let c1 = Box::new(Node::new_constant(NumericType::new(1.0)));
        let c2 = Box::new(Node::new_constant(NumericType::new(1.0)));

        let mut equal = Box::new(Node::new_equal());
        equal.add_child(c1.clone());
        equal.add_child(c2.clone());

        let mut not = Box::new(Node::new_not());
        not.add_child(equal.clone());

        base.add_child(equal.clone());
        base.add_child(not.clone());

        let evaluator = SingleScenarioEvaluator::new();
        evaluator.const_visit(base).unwrap();
        assert_eq!(evaluator.boolean_stack().pop().unwrap(), false);
    }

    #[test]
    fn test_if() {
        let mut base = Box::new(Node::new_base());

        let x = Box::new(Node::new_variable_with_id("x".to_string(), 0));
        let c1 = Box::new(Node::new_constant(NumericType::new(1.0)));

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

        let evaluator = SingleScenarioEvaluator::new().with_variables(1);
        evaluator.const_visit(base).unwrap();
        assert_eq!(
            evaluator.variables().pop().unwrap(),
            Value::Number(NumericType::new(2.0))
        );
    }

    #[test]
    fn test_if_new_variable() {
        let base = Box::new(Node::Base(vec![
            Box::new(Node::Assign(vec![
                Box::new(Node::Variable(Vec::new(), "x".to_string(), 0.into())),
                Box::new(Node::Constant(NumericType::new(2.0))),
            ])),
            Box::new(Node::If(
                vec![
                    Box::new(Node::Equal(vec![
                        Box::new(Node::Variable(Vec::new(), "x".to_string(), 0.into())),
                        Box::new(Node::Constant(NumericType::new(1.0))),
                    ])),
                    Box::new(Node::Assign(vec![
                        Box::new(Node::Variable(Vec::new(), "z".to_string(), 1.into())),
                        Box::new(Node::Constant(NumericType::new(3.0))),
                    ])),
                    Box::new(Node::Assign(vec![
                        Box::new(Node::Variable(Vec::new(), "w".to_string(), 2.into())),
                        Box::new(Node::Constant(NumericType::new(4.0))),
                    ])),
                ],
                None,
            )),
        ]));

        let evaluator = SingleScenarioEvaluator::new().with_variables(3);
        evaluator.const_visit(base).unwrap();

        assert_eq!(
            evaluator.variables().get(0).unwrap(),
            &Value::Number(NumericType::new(2.0))
        );
        assert_eq!(evaluator.variables().get(1).unwrap(), &Value::Null);
        assert_eq!(evaluator.variables().get(2).unwrap(), &Value::Null);
    }
}

#[cfg(test)]
mod expr_evaluator_tests {
    use super::*;

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

        let evaluator = SingleScenarioEvaluator::new().with_variables(indexer.get_variables_size());
        evaluator.const_visit(nodes).unwrap();

        assert_eq!(
            *evaluator.variables().get(0).unwrap(),
            Value::Number(NumericType::new(1.0))
        );
        assert_eq!(
            *evaluator.variables().get(1).unwrap(),
            Value::Number(NumericType::new(2.0))
        );
        assert_eq!(
            *evaluator.variables().get(2).unwrap(),
            Value::Number(NumericType::new(3.0))
        );
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

        let evaluator = SingleScenarioEvaluator::new().with_variables(indexer.get_variables_size());
        evaluator.const_visit(nodes).unwrap();

        assert_eq!(
            *evaluator.variables().get(0).unwrap(),
            Value::Number(NumericType::new(2.0))
        );
        assert_eq!(
            *evaluator.variables().get(1).unwrap(),
            Value::Number(NumericType::new(2.0))
        );
        assert_eq!(
            *evaluator.variables().get(2).unwrap(),
            Value::Number(NumericType::new(4.0))
        );
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

        let evaluator = SingleScenarioEvaluator::new().with_variables(indexer.get_variables_size());
        evaluator.const_visit(nodes).unwrap();

        assert_eq!(
            *evaluator.variables().get(0).unwrap(),
            Value::Number(NumericType::new(2.0))
        );
        assert_eq!(
            *evaluator.variables().get(1).unwrap(),
            Value::Number(NumericType::new(2.0))
        );
        assert_eq!(
            *evaluator.variables().get(2).unwrap(),
            Value::Number(NumericType::new(4.0))
        );
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

        let evaluator = SingleScenarioEvaluator::new().with_variables(indexer.get_variables_size());
        evaluator.const_visit(nodes).unwrap();

        assert_eq!(
            *evaluator.variables().get(0).unwrap(),
            Value::Number(NumericType::new(2.0))
        );
        assert_eq!(
            *evaluator.variables().get(1).unwrap(),
            Value::Number(NumericType::new(2.0))
        );
        assert_eq!(
            *evaluator.variables().get(2).unwrap(),
            Value::Number(NumericType::new(5.0))
        );
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

        let evaluator = SingleScenarioEvaluator::new().with_variables(indexer.get_variables_size());
        evaluator.const_visit(nodes).unwrap();

        assert_eq!(
            *evaluator.variables().get(0).unwrap(),
            Value::Number(NumericType::new(2.0))
        );
        assert_eq!(
            *evaluator.variables().get(1).unwrap(),
            Value::Number(NumericType::new(2.0))
        );
        assert_eq!(
            *evaluator.variables().get(2).unwrap(),
            Value::Number(NumericType::new(4.0))
        );
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

        let evaluator = SingleScenarioEvaluator::new().with_variables(indexer.get_variables_size());
        evaluator.const_visit(nodes).unwrap();

        assert_eq!(
            *evaluator.variables().get(0).unwrap(),
            Value::Number(NumericType::new(2.0))
        );
        assert_eq!(
            *evaluator.variables().get(1).unwrap(),
            Value::Number(NumericType::new(2.0))
        );
        assert_eq!(
            *evaluator.variables().get(2).unwrap(),
            Value::Number(NumericType::new(3.0))
        );
        assert_eq!(
            *evaluator.variables().get(3).unwrap(),
            Value::Number(NumericType::new(4.0))
        );
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

        let evaluator = SingleScenarioEvaluator::new().with_variables(indexer.get_variables_size());
        evaluator.const_visit(nodes).unwrap();

        assert_eq!(
            *evaluator.variables().get(0).unwrap(),
            Value::Number(NumericType::new(2.0))
        );
        assert_eq!(
            *evaluator.variables().get(1).unwrap(),
            Value::Number(NumericType::new(2.0))
        );
        assert_eq!(
            *evaluator.variables().get(2).unwrap(),
            Value::Number(NumericType::new(5.0))
        );
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

        let evaluator = SingleScenarioEvaluator::new().with_variables(indexer.get_variables_size());
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

        let evaluator = SingleScenarioEvaluator::new().with_variables(indexer.get_variables_size());
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

        let evaluator = SingleScenarioEvaluator::new().with_variables(indexer.get_variables_size());
        evaluator.const_visit(nodes).unwrap();

        assert_eq!(*evaluator.variables().get(0).unwrap(), Value::Bool(true));
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

        assert_eq!(
            results.get("x"),
            Some(&Value::Number(NumericType::new(1.0)))
        );
        assert_eq!(
            results.get("y"),
            Some(&Value::Number(NumericType::new(2.0)))
        );
        assert_eq!(
            results.get("z"),
            Some(&Value::Number(NumericType::new(3.0)))
        );
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

        assert_eq!(
            results.get("x"),
            Some(&Value::Number(NumericType::new(1.0)))
        );
        assert_eq!(
            results.get("y"),
            Some(&Value::Number(NumericType::new(2.0)))
        );
        assert_eq!(
            results.get("z"),
            Some(&Value::Number(NumericType::new(3.0)))
        );
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

        let c1 = Node::new_constant(NumericType::new(1.0));

        unary_plus.add_child(Box::new(c1));
        base.add_child(unary_plus);

        let evaluator = SingleScenarioEvaluator::new();
        evaluator.const_visit(base).unwrap();

        assert_eq!(evaluator.digit_stack().pop().unwrap(), 1.0);
    }

    #[test]
    fn test_unary_minus_node() {
        // Test the UnaryMinus node to ensure it correctly negates the value.
        let mut base = Box::new(Node::new_base());
        let mut unary_minus = Box::new(Node::new_unary_minus());

        let c1 = Node::new_constant(NumericType::new(1.0));

        unary_minus.add_child(Box::new(c1));
        base.add_child(unary_minus);

        let evaluator = SingleScenarioEvaluator::new();
        evaluator.const_visit(base).unwrap();

        assert_eq!(evaluator.digit_stack().pop().unwrap(), -1.0);
    }

    #[test]
    fn test_min_node() {
        // Test the Min node to ensure it correctly finds the minimum value between two constants.
        let mut base = Box::new(Node::new_base());
        let mut min = Box::new(Node::new_min());

        let c1 = Node::new_constant(NumericType::new(1.0));
        let c2 = Node::new_constant(NumericType::new(2.0));

        min.add_child(Box::new(c1));
        min.add_child(Box::new(c2));
        base.add_child(min);

        let evaluator = SingleScenarioEvaluator::new();
        evaluator.const_visit(base).unwrap();

        assert_eq!(evaluator.digit_stack().pop().unwrap(), 1.0);
    }

    #[test]
    fn test_max_node() {
        // Test the Max node to ensure it correctly finds the maximum value between two constants.
        let mut base = Box::new(Node::new_base());
        let mut max = Box::new(Node::new_max());

        let c1 = Node::new_constant(NumericType::new(1.0));
        let c2 = Node::new_constant(NumericType::new(2.0));
        max.add_child(Box::new(c1));
        max.add_child(Box::new(c2));
        base.add_child(max);

        let evaluator = SingleScenarioEvaluator::new();
        evaluator.const_visit(base).unwrap();

        assert_eq!(evaluator.digit_stack().pop().unwrap(), 2.0);
    }

    #[test]
    fn test_fif_node() {
        // Test the Fif node to ensure it correctly computes the functional if.
        let mut base = Box::new(Node::new_base());
        let mut fif = Box::new(Node::new_fif());

        fif.add_child(Box::new(Node::new_constant(NumericType::new(0.0))));
        fif.add_child(Box::new(Node::new_constant(NumericType::new(1.0))));
        fif.add_child(Box::new(Node::new_constant(NumericType::new(0.0))));
        fif.add_child(Box::new(Node::new_constant(NumericType::new(1.0))));

        base.add_child(fif);

        let evaluator = SingleScenarioEvaluator::new();
        evaluator.const_visit(base).unwrap();

        let res = evaluator.digit_stack().pop().unwrap();
        assert!((res - 0.5).abs() < 1e-12);
    }

    #[test]
    fn test_pow_node() {
        // Test the Pow node to ensure it correctly calculates the power of one constant to another.
        let mut base = Box::new(Node::new_base());
        let mut pow = Box::new(Node::new_pow());

        let c1 = Node::new_constant(NumericType::new(2.0));
        let c2 = Node::new_constant(NumericType::new(3.0));

        pow.add_child(Box::new(c1));
        pow.add_child(Box::new(c2));
        base.add_child(pow);

        let evaluator = SingleScenarioEvaluator::new();
        evaluator.const_visit(base).unwrap();

        assert_eq!(evaluator.digit_stack().pop().unwrap(), 8.0);
    }

    #[test]
    fn test_ln_node() {
        // Test the Ln node to ensure it correctly calculates the natural logarithm of a constant.
        let mut base = Box::new(Node::new_base());
        let mut ln = Box::new(Node::new_ln());

        let c1 = Node::new_constant(NumericType::new(2.718281828459045));

        ln.add_child(Box::new(c1));
        base.add_child(ln);

        let evaluator = SingleScenarioEvaluator::new();
        evaluator.const_visit(base).unwrap();

        assert!((evaluator.digit_stack().pop().unwrap() - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_exp_node() {
        // Test the Exp node to ensure it correctly calculates the exponential of a constant.
        let mut base = Box::new(Node::new_base());
        let mut exp = Box::new(Node::new_exp());

        let c1 = Node::new_constant(NumericType::new(1.0));

        exp.add_child(Box::new(c1));
        base.add_child(exp);

        let evaluator = SingleScenarioEvaluator::new();
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

        let evaluator = SingleScenarioEvaluator::new();
        evaluator.const_visit(base).unwrap();

        assert!((evaluator.digit_stack().pop().unwrap() - (152.0 / 360.0)).abs() < f64::EPSILON);
    }

    #[test]
    fn test_pays_node_discount() {
        // Pays should apply the discount factor fetched from the scenario
        let mut base = Box::new(Node::new_base());
        let mut pays = Box::new(Node::new_pays());
        pays.add_child(Box::new(Node::new_constant(NumericType::new(100.0))));
        base.add_child(pays);

        let event_date = Date::new(2024, 1, 1);
        let scenario = vec![SimulationData::new(
            NumericType::new(1.0),
            vec![NumericType::new(0.5)],
            Vec::new(),
            Vec::new(),
            Vec::new(),
        )];

        let indexer = EventIndexer::new().with_event_date(event_date);
        let event = Event::new(event_date, base.clone());
        let events = EventStream::new().with_events(vec![event]);
        indexer.visit_events(&events).unwrap();

        let evaluator = SingleScenarioEvaluator::new()
            .with_scenario(&scenario)
            .with_variables(indexer.get_variables_size());
        let expr = events.events().first().unwrap().expr().clone();
        evaluator.const_visit(expr).unwrap();

        assert!((evaluator.digit_stack().pop().unwrap() - 50.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_pays_node_discount_and_fx() {
        // Pays should apply discount and fx conversion
        let mut base = Box::new(Node::new_base());
        let mut pays = Box::new(Node::new_pays());
        pays.add_child(Box::new(Node::new_constant(NumericType::new(100.0))));
        // set currency EUR
        if let Node::Pays(_, ref mut date, ref mut ccy, _, _) = *pays {
            *date = Some(Date::new(2024, 1, 1));
            *ccy = Some(Currency::EUR);
        }
        base.add_child(pays);

        let event_date = Date::new(2024, 1, 1);
        let scenario = vec![SimulationData::new(
            NumericType::new(2.0),
            vec![NumericType::new(0.5)],
            Vec::new(),
            vec![NumericType::new(0.8)],
            Vec::new(),
        )];

        let indexer = EventIndexer::new()
            .with_event_date(event_date)
            .with_local_currency(Currency::USD);
        let event = Event::new(event_date, base.clone());
        let events = EventStream::new().with_events(vec![event]);
        indexer.visit_events(&events).unwrap();

        let evaluator = SingleScenarioEvaluator::new()
            .with_scenario(&scenario)
            .with_variables(indexer.get_variables_size());
        let expr = events.events().first().unwrap().expr().clone();
        evaluator.const_visit(expr).unwrap();

        // 100 * 0.5 * 0.8 / 2 = 20
        assert!((evaluator.digit_stack().pop().unwrap() - 20.0).abs() < f64::EPSILON);
    }

    // #[test]
    // fn test_cvg_with_pays() {
    //     let mut base = Box::new(Node::new_base());
    //     let mut pays = Box::new(Node::new_pays());
    //     let mut cvg = Box::new(Node::new_cvg());
    //     cvg.add_child(Box::new(Node::String("2020-01-01".to_string())));
    //     cvg.add_child(Box::new(Node::String("2020-06-01".to_string())));
    //     cvg.add_child(Box::new(Node::String("Actual360".to_string())));
    //     pays.add_child(cvg);
    //     base.add_child(pays);

    //     let event_date = Date::from_str("2020-06-01", "%Y-%m-%d").unwrap();
    //     let scenario = vec![MarketData::new(
    //         0,
    //         event_date,
    //         None,
    //         None,
    //         None,
    //         NumericType::new(2.0),
    //     )];

    //     let indexer = EventIndexer::new();
    //     indexer.visit(&base).unwrap();

    //     let evaluator = SingleScenarioEvaluator::new().with_scenario(&scenario);
    //     evaluator.const_visit(base).unwrap();

    //     assert!(
    //         (evaluator.digit_stack().pop().unwrap() - (152.0 / 360.0) / 2.0).abs() < f64::EPSILON
    //     );
    // }

    // #[test]
    // fn test_pays_node_discount() {
    //     // Pays should discount the evaluated value by the scenario numerarie
    //     let mut base = Box::new(Node::new_base());
    //     let mut pays = Box::new(Node::new_pays());
    //     pays.add_child(Box::new(Node::new_constant(NumericType::new(100.0))));
    //     base.add_child(pays);

    //     let event_date = Date::new(2024, 1, 1);
    //     let scenario = vec![MarketData::new(
    //         0,
    //         event_date,
    //         None,
    //         None,
    //         None,
    //         NumericType::new(2.0),
    //     )];

    //     let indexer = EventIndexer::new();
    //     indexer.visit(&base).unwrap();

    //     let evaluator = SingleScenarioEvaluator::new().with_scenario(&scenario);
    //     evaluator.const_visit(base).unwrap();

    //     assert_eq!(evaluator.digit_stack().pop().unwrap(), 50.0);
    // }

    // #[test]
    // fn test_rate_index_eval() {
    //     let mut base = Box::new(Node::new_base());
    //     let rate = Node::new_rate_index(
    //         "0".to_string(),
    //         Date::new(2024, 1, 1),
    //         Date::new(2024, 2, 1),
    //     );
    //     base.add_child(Box::new(rate));

    //     let scenario = vec![MarketData::new(
    //         0,
    //         Date::new(2024, 1, 1),
    //         None,
    //         Some(NumericType::new(0.05)),
    //         None,
    //         NumericType::new(1.0),
    //     )];

    //     let indexer = EventIndexer::new();
    //     indexer.visit(&base).unwrap();

    //     let evaluator = SingleScenarioEvaluator::new().with_scenario(&scenario);
    //     evaluator.const_visit(base).unwrap();

    //     assert!((evaluator.digit_stack().pop().unwrap() - 0.05).abs() < f64::EPSILON);
    // }

    #[test]
    fn test_not_equal_node() {
        // Test the NotEqual node to ensure it correctly evaluates the inequality of two constants.
        let mut base = Box::new(Node::new_base());
        let mut not_equal = Box::new(Node::new_not_equal());

        let c1 = Node::new_constant(NumericType::new(1.0));
        let c2 = Node::new_constant(NumericType::new(2.0));

        not_equal.add_child(Box::new(c1));
        not_equal.add_child(Box::new(c2));
        base.add_child(not_equal);

        let evaluator = SingleScenarioEvaluator::new();
        evaluator.const_visit(base).unwrap();

        assert_eq!(evaluator.boolean_stack().pop().unwrap(), true);
    }

    #[test]
    fn test_add_assign_number() {
        // Test the AddAssign trait for Value::Number to ensure it correctly adds two numbers.
        let mut a = Value::Number(NumericType::new(3.0));
        let b = Value::Number(NumericType::new(1.0));
        a += b;
        assert_eq!(a, Value::Number(NumericType::new(4.0)));
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
        let mut a = Value::Number(NumericType::new(3.0));
        let b = Value::Number(NumericType::new(1.0));
        a -= b;
        assert_eq!(a, Value::Number(NumericType::new(2.0)));
    }

    #[test]
    fn test_add_number_and_string() {
        // Test the Add trait for Value to ensure it returns Value::Null when adding a number and a string.
        let a = Value::Number(NumericType::new(1.0));
        let b = Value::String("Hello".to_string());
        let result = a + b;
        assert_eq!(result, Value::Null);
    }

    #[test]
    fn test_sub_number_and_string() {
        // Test the Sub trait for Value to ensure it returns Value::Null when subtracting a string from a number.
        let a = Value::Number(NumericType::new(1.0));
        let b = Value::String("Hello".to_string());
        let result = a - b;
        assert_eq!(result, Value::Null);
    }

    #[test]
    fn test_mul_number_and_string() {
        // Test the Mul trait for Value to ensure it returns Value::Null when multiplying a number and a string.
        let a = Value::Number(NumericType::new(1.0));
        let b = Value::String("Hello".to_string());
        let result = a * b;
        assert_eq!(result, Value::Null);
    }

    #[test]
    fn test_div_number_and_string() {
        // Test the Div trait for Value to ensure it returns Value::Null when dividing a number by a string.
        let a = Value::Number(NumericType::new(1.0));
        let b = Value::String("Hello".to_string());
        let result = a / b;
        assert_eq!(result, Value::Null);
    }

    #[test]
    fn test_add_bool_and_number() {
        // Test the Add trait for Value to ensure it returns Value::Null when adding a boolean and a number.
        let a = Value::Bool(true);
        let b = Value::Number(NumericType::new(1.0));
        let result = a + b;
        assert_eq!(result, Value::Null);
    }

    #[test]
    fn test_sub_bool_and_number() {
        // Test the Sub trait for Value to ensure it returns Value::Null when subtracting a number from a boolean.
        let a = Value::Bool(true);
        let b = Value::Number(NumericType::new(1.0));
        let result = a - b;
        assert_eq!(result, Value::Null);
    }

    #[test]
    fn test_mul_bool_and_number() {
        // Test the Mul trait for Value to ensure it returns Value::Null when multiplying a boolean and a number.
        let a = Value::Bool(true);
        let b = Value::Number(NumericType::new(1.0));
        let result = a * b;
        assert_eq!(result, Value::Null);
    }

    #[test]
    fn test_div_bool_and_number() {
        // Test the Div trait for Value to ensure it returns Value::Null when dividing a boolean by a number.
        let a = Value::Bool(true);
        let b = Value::Number(NumericType::new(1.0));
        let result = a / b;
        assert_eq!(result, Value::Null);
    }

    #[test]
    fn test_add_null_and_number() {
        // Test the Add trait for Value to ensure it returns Value::Null when adding a null and a number.
        let a = Value::Null;
        let b = Value::Number(NumericType::new(1.0));
        let result = a + b;
        assert_eq!(result, Value::Null);
    }

    #[test]
    fn test_sub_null_and_number() {
        // Test the Sub trait for Value to ensure it returns Value::Null when subtracting a number from a null.
        let a = Value::Null;
        let b = Value::Number(NumericType::new(1.0));
        let result = a - b;
        assert_eq!(result, Value::Null);
    }

    #[test]
    fn test_mul_null_and_number() {
        // Test the Mul trait for Value to ensure it returns Value::Null when multiplying a null and a number.
        let a = Value::Null;
        let b = Value::Number(NumericType::new(1.0));
        let result = a * b;
        assert_eq!(result, Value::Null);
    }

    #[test]
    fn test_div_null_and_number() {
        // Test the Div trait for Value to ensure it returns Value::Null when dividing a null by a number.
        let a = Value::Null;
        let b = Value::Number(NumericType::new(1.0));
        let result = a / b;
        assert_eq!(result, Value::Null);
    }

    #[test]
    fn test_event_stream_evaluator_no_scenarios() {
        // Test the EventStreamEvaluator to ensure it returns an error when no scenarios are set.
        let evaluator: EventStreamEvaluator<'_> = EventStreamEvaluator::new(1);
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
        // Test the SingleScenarioEvaluator to ensure it correctly resizes the variables.
        let evaluator = SingleScenarioEvaluator::new().with_variables(3);
        assert_eq!(evaluator.variables().len(), 3);
        assert_eq!(
            evaluator.variables(),
            vec![Value::Null, Value::Null, Value::Null]
        );
    }

    // #[test]
    // fn test_expr_evaluator_digit_stack() {
    //     // Test the SingleScenarioEvaluator to ensure it correctly returns the digit stack.
    //     let evaluator = SingleScenarioEvaluator::new();
    //     evaluator
    //         .digit_stack
    //         .lock()
    //         .unwrap()
    //         .push(NumericType::new(1.0));
    //     assert_eq!(evaluator.digit_stack(), vec![1.0]);
    // }

    #[test]
    fn test_expr_evaluator_boolean_stack() {
        // Test the SingleScenarioEvaluator to ensure it correctly returns the boolean stack.
        let evaluator = SingleScenarioEvaluator::new();
        evaluator.boolean_stack.borrow_mut().push(true);
        assert_eq!(evaluator.boolean_stack(), vec![true]);
    }

    #[test]
    fn test_expr_evaluator_is_lhs_variable() {
        // Test the SingleScenarioEvaluator to ensure it correctly sets and gets the is_lhs_variable flag.
        let evaluator = SingleScenarioEvaluator::new();
        *evaluator.is_lhs_variable.borrow_mut() = true;
        assert_eq!(*evaluator.is_lhs_variable.borrow_mut(), true);
    }

    #[test]
    fn test_expr_evaluator_lhs_variable() {
        // Test the SingleScenarioEvaluator to ensure it correctly sets and gets the lhs_variable.
        let evaluator = SingleScenarioEvaluator::new();
        let node = Box::new(Node::new_constant(NumericType::new(1.0)));
        *evaluator.lhs_variable.borrow_mut() = Some(node.clone());
        assert_eq!(*evaluator.lhs_variable.borrow_mut(), Some(node));
    }

    #[test]
    fn test_for_each_range_loop() {
        let script = "total = 0; for i in range(1,3) { total = total + i; }";
        let expr = ExprTree::try_from(script).unwrap();
        let indexer = EventIndexer::new();
        indexer.visit(&expr).unwrap();
        let evaluator = SingleScenarioEvaluator::new().with_variables(indexer.get_variables_size());
        evaluator.const_visit(expr).unwrap();
        let idx = indexer.get_variable_index("total").unwrap();
        if let Value::Number(v) = evaluator.variables().get(idx).unwrap() {
            assert_eq!(*v, NumericType::new(6.0));
        } else {
            panic!("variable not found");
        }
    }

    #[test]
    fn test_for_each_list_loop() {
        let script = "sum = 0; for x in [1,2,3] { sum = sum + x; }";
        let expr = ExprTree::try_from(script).unwrap();
        let indexer = EventIndexer::new();
        indexer.visit(&expr).unwrap();
        let evaluator = SingleScenarioEvaluator::new().with_variables(indexer.get_variables_size());
        evaluator.const_visit(expr).unwrap();
        let idx = indexer.get_variable_index("sum").unwrap();
        if let Value::Number(v) = evaluator.variables().get(idx).unwrap() {
            assert_eq!(*v, NumericType::new(6.0));
        } else {
            panic!("variable not found");
        }
    }

    #[test]
    fn test_list_assignment() {
        let script = "a = [1,2];";
        let expr = ExprTree::try_from(script).unwrap();
        let indexer = EventIndexer::new();
        indexer.visit(&expr).unwrap();
        let evaluator = SingleScenarioEvaluator::new().with_variables(indexer.get_variables_size());
        evaluator.const_visit(expr).unwrap();
        let idx = indexer.get_variable_index("a").unwrap();
        if let Value::Array(arr) = evaluator.variables().get(idx).unwrap() {
            assert_eq!(arr.len(), 2);
        } else {
            panic!("variable not found");
        }
    }

    #[test]
    fn test_list_with_variable_values() {
        let script = "x = 1; a = [x, 2];";
        let expr = ExprTree::try_from(script).unwrap();
        let indexer = EventIndexer::new();
        indexer.visit(&expr).unwrap();
        let evaluator = SingleScenarioEvaluator::new().with_variables(indexer.get_variables_size());
        evaluator.const_visit(expr).unwrap();
        let idx = indexer.get_variable_index("a").unwrap();
        if let Value::Array(arr) = evaluator.variables().get(idx).unwrap() {
            assert_eq!(arr.len(), 2);
            match arr[0] {
                Value::Number(n) => assert_eq!(n, NumericType::new(1.0)),
                _ => panic!("unexpected value"),
            }
        } else {
            panic!("variable not found");
        }
    }

    #[test]
    fn test_array_indexing() {
        let script = "arr = [1,2,3]; x = arr[1];";
        let expr = ExprTree::try_from(script).unwrap();
        let indexer = EventIndexer::new();
        indexer.visit(&expr).unwrap();
        let evaluator = SingleScenarioEvaluator::new().with_variables(indexer.get_variables_size());
        evaluator.const_visit(expr).unwrap();
        let idx = indexer.get_variable_index("x").unwrap();
        if let Value::Number(v) = evaluator.variables().get(idx).unwrap() {
            assert_eq!(*v, NumericType::new(2.0));
        } else {
            panic!("variable not found");
        }
    }

    #[test]
    fn test_array_indexing_out_of_bounds() {
        let script = "arr = [1,2,3]; x = arr[5];";
        let expr = ExprTree::try_from(script).unwrap();
        let indexer = EventIndexer::new();
        indexer.visit(&expr).unwrap();
        let evaluator = SingleScenarioEvaluator::new().with_variables(indexer.get_variables_size());
        let result = evaluator.const_visit(expr);
        assert!(result.is_err());
    }

    #[test]
    fn test_for_each_variable_loop() {
        let script = "arr = [1,2,3]; sum = 0; for v in arr { sum = sum + v; }";
        let expr = ExprTree::try_from(script).unwrap();
        let indexer = EventIndexer::new();
        indexer.visit(&expr).unwrap();
        let evaluator = SingleScenarioEvaluator::new().with_variables(indexer.get_variables_size());
        evaluator.const_visit(expr).unwrap();
        let idx = indexer.get_variable_index("sum").unwrap();
        if let Value::Number(v) = evaluator.variables().get(idx).unwrap() {
            assert_eq!(*v, NumericType::new(6.0));
        } else {
            panic!("variable not found");
        }
    }

    #[test]
    fn test_compound_assignments() {
        let script = "x = 1; x += 2; x -= 1; x *= 3; x /= 2;";
        let expr = ExprTree::try_from(script).unwrap();
        let indexer = EventIndexer::new();
        indexer.visit(&expr).unwrap();
        let evaluator = SingleScenarioEvaluator::new().with_variables(indexer.get_variables_size());
        evaluator.const_visit(expr).unwrap();
        let idx = indexer.get_variable_index("x").unwrap();
        if let Value::Number(v) = evaluator.variables().get(idx).unwrap() {
            assert_eq!(*v, NumericType::new(3.0));
        } else {
            panic!("variable not found");
        }
    }

    #[test]
    fn test_append_and_statistics() {
        let script = "arr = [1,2]; arr.append(3); mean_val = arr.mean(); std_val = arr.std();";
        let expr = ExprTree::try_from(script).unwrap();
        let indexer = EventIndexer::new();
        indexer.visit(&expr).unwrap();
        let evaluator = SingleScenarioEvaluator::new().with_variables(indexer.get_variables_size());
        evaluator.const_visit(expr).unwrap();
        let arr_idx = indexer.get_variable_index("arr").unwrap();
        if let Value::Array(arr) = evaluator.variables().get(arr_idx).unwrap() {
            assert_eq!(arr.len(), 3);
        } else {
            panic!("arr not found");
        }
        let mean_idx = indexer.get_variable_index("mean_val").unwrap();
        if let Value::Number(v) = evaluator.variables().get(mean_idx).unwrap() {
            assert!((v.value() - 2.0).abs() < 1e-8);
        } else {
            panic!("mean not found");
        }
        let std_idx = indexer.get_variable_index("std_val").unwrap();
        if let Value::Number(v) = evaluator.variables().get(std_idx).unwrap() {
            assert!((v.value() - 0.81649658).abs() < 1e-6);
        } else {
            panic!("std not found");
        }
    }

    #[test]
    fn test_mean_on_literal_list() {
        let script = "avg = [1,2,3].mean();";
        let expr = ExprTree::try_from(script).unwrap();
        let indexer = EventIndexer::new();
        indexer.visit(&expr).unwrap();
        let evaluator = SingleScenarioEvaluator::new().with_variables(indexer.get_variables_size());
        evaluator.const_visit(expr).unwrap();
        let idx = indexer.get_variable_index("avg").unwrap();
        if let Value::Number(v) = evaluator.variables().get(idx).unwrap() {
            assert!((v.value() - 2.0).abs() < 1e-8);
        } else {
            panic!("avg not found");
        }
    }

    #[test]
    fn test_expr_evaluator_with_scenario_none() {
        // Test the SingleScenarioEvaluator to ensure it correctly handles None scenario.
        let evaluator = SingleScenarioEvaluator::new();
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
