use rustatlas::prelude::*;
use std::{cell::RefCell, collections::{BTreeMap, HashMap}};

use crate::prelude::*;
use crate::utils::errors::{Result, ScriptingError};

use super::evaluator::{Value};

/// Visitor that collects undiscounted cashflows per currency for a single scenario.
pub struct SingleScenarioCashflowCollector<'a> {
    variables: RefCell<Vec<Value>>,
    digit_stack: RefCell<Vec<NumericType>>,
    boolean_stack: RefCell<Vec<bool>>,
    string_stack: RefCell<Vec<String>>,
    array_stack: RefCell<Vec<Vec<Value>>>,
    is_lhs_variable: RefCell<bool>,
    lhs_variable: RefCell<Option<Box<Node>>>,
    scenario: Option<&'a Scenario>,
    current_event: RefCell<usize>,
    current_event_date: RefCell<Option<Date>>,
    local_currency: Currency,
    cashflows: RefCell<HashMap<Currency, BTreeMap<Date, NumericType>>>,
}

impl<'a> SingleScenarioCashflowCollector<'a> {
    pub fn new(local_currency: Currency) -> Self {
        Self {
            variables: RefCell::new(Vec::new()),
            digit_stack: RefCell::new(Vec::new()),
            boolean_stack: RefCell::new(Vec::new()),
            string_stack: RefCell::new(Vec::new()),
            array_stack: RefCell::new(Vec::new()),
            is_lhs_variable: RefCell::new(false),
            lhs_variable: RefCell::new(None),
            scenario: None,
            current_event: RefCell::new(0),
            current_event_date: RefCell::new(None),
            local_currency,
            cashflows: RefCell::new(HashMap::new()),
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

    pub fn set_variable(&self, idx: usize, val: Value) {
        let mut vars = self.variables.borrow_mut();
        if idx >= vars.len() {
            vars.resize(idx + 1, Value::Null);
        }
        vars[idx] = val;
    }

    pub fn set_current_event(&self, event: usize, date: Date) {
        *self.current_event.borrow_mut() = event;
        *self.current_event_date.borrow_mut() = Some(date);
    }

    pub fn cashflows(&self) -> HashMap<Currency, BTreeMap<Date, NumericType>> {
        self.cashflows.borrow().clone()
    }
}

impl<'a> NodeConstVisitor for SingleScenarioCashflowCollector<'a> {
    type Output = Result<()>;
    fn const_visit(&self, node: Box<Node>) -> Self::Output {
        let eval: Result<()> = match node.as_ref() {
            Node::Base(children) => {
                children.iter().try_for_each(|child| self.const_visit(child.clone()))?;
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
                            )));
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
                                    )));
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
            Node::Pays(children, pay_date, currency, df_index, fx_index) => {
                children.iter().try_for_each(|child| self.const_visit(child.clone()))?;

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
                let df_id = df_index
                    .get()
                    .ok_or(ScriptingError::EvaluationError(
                        "Pays not indexed".to_string(),
                    ))?;
                let df = market_data.get_df(*df_id)?;
                let numerarie = market_data.numerarie();

                // accumulate undiscounted cashflow
                let pay_date = pay_date.unwrap_or(
                    self
                        .current_event_date
                        .borrow()
                        .ok_or(ScriptingError::EvaluationError(
                            "Event date not set".to_string(),
                        ))?,
                );
                let ccy = currency.unwrap_or(self.local_currency);
                {
                    let mut map = self.cashflows.borrow_mut();
                    let entry = map.entry(ccy).or_insert_with(BTreeMap::new);
                    let amt = entry.entry(pay_date).or_insert(NumericType::new(0.0));
                    *amt = (*amt + current_value).into();
                }

                let value: NumericType = if let Some(_) = currency {
                    let fx_id = fx_index
                        .get()
                        .ok_or(ScriptingError::EvaluationError(
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

impl<'a> SingleScenarioCashflowCollector<'a> {
    pub fn visit_events(&self, events: &EventStream) -> Result<HashMap<Currency, BTreeMap<Date, NumericType>>> {
        events.events().iter().enumerate().try_for_each(|(i, ev)| {
            self.set_current_event(i, ev.event_date());
            self.const_visit(ev.expr().clone())
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
        Self { n_vars, scenarios, local_currency }
    }

    pub fn visit_events(&self, events: &EventStream) -> Result<HashMap<Currency, BTreeMap<Date, NumericType>>> {
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

