// use std::cell::RefCell;

// use crate::prelude::*;
// use crate::visitors::evaluator::{SingleScenarioEvaluator, Value};
// use rustatlas::prelude::*;

// /// Evaluator implementing a simple fuzzy logic mode using
// /// the `fIf` smoothing kernel described in `docs/AGENTS.md`.
// ///
// /// The evaluator behaves like `SingleScenarioEvaluator` but
// /// logical operations return values in `[0,1]` and assignments
// /// inside `if` blocks are weighted by these probabilities.
// pub struct FuzzyEvaluator<'a> {
//     base: SingleScenarioEvaluator<'a>,
//     dt_stack: RefCell<Vec<NumericType>>, // condition truth values in [0,1]
//     weight_stack: RefCell<Vec<NumericType>>, // multiplicative weights
//     eps: f64,
// }

// impl<'a> FuzzyEvaluator<'a> {
//     /// Create a new fuzzy evaluator with default epsilon = 1.
//     pub fn new() -> Self {
//         Self {
//             base: SingleScenarioEvaluator::new(),
//             dt_stack: RefCell::new(Vec::new()),
//             weight_stack: RefCell::new(vec![NumericType::one()]),
//             eps: 1e-12,
//         }
//     }

//     /// Set market scenario for market-data dependent nodes.
//     pub fn with_scenario(mut self, scenario: &'a Scenario) -> Self {
//         self.base = self.base.with_scenario(scenario);
//         self
//     }

//     /// Pre-allocate variable storage.
//     pub fn with_variables(mut self, n: usize) -> Self {
//         self.base = self.base.with_variables(n);
//         self
//     }

//     /// Current variables after evaluation.
//     pub fn variables(&self) -> Vec<Value> {
//         self.base.variables()
//     }

//     /// Access numeric stack (mainly for tests).
//     pub fn digit_stack(&self) -> Vec<NumericType> {
//         self.base.digit_stack()
//     }

//     fn push_weight(&self, w: NumericType) {
//         let current = *self.weight_stack.borrow().last().unwrap();
//         self.weight_stack.borrow_mut().push((current * w).into());
//     }

//     fn pop_weight(&self) {
//         self.weight_stack.borrow_mut().pop();
//     }

//     fn current_weight(&self) -> NumericType {
//         *self.weight_stack.borrow().last().unwrap()
//     }

//     fn fif(&self, x: NumericType, a: NumericType, b: NumericType) -> NumericType {
//         let half = self.eps * 0.5;
//         let inner = (x + half)
//             .min(NumericType::from(self.eps))
//             .max(NumericType::zero());
//         let res = b.clone() + ((a - b) / self.eps) * inner;
//         res.into()
//     }
// }

// impl<'a> NodeConstVisitor for FuzzyEvaluator<'a> {
//     type Output = Result<()>;

//     fn const_visit(&self, node: Box<Node>) -> Self::Output {
//         match node.as_ref() {
//             Node::True => {
//                 self.dt_stack.borrow_mut().push(NumericType::one());
//                 Ok(())
//             }
//             Node::False => {
//                 self.dt_stack.borrow_mut().push(NumericType::zero());
//                 Ok(())
//             }
//             Node::Superior(children) => {
//                 for c in children {
//                     c.const_accept(self);
//                 }
//                 let right = self.base.digit_stack.borrow_mut().pop().unwrap();
//                 let left = self.base.digit_stack.borrow_mut().pop().unwrap();
//                 let dt = self.fif(
//                     (left - right).into(),
//                     NumericType::one(),
//                     NumericType::zero(),
//                 );

//                 self.dt_stack.borrow_mut().push(dt);
//                 Ok(())
//             }
//             Node::Inferior(children) => {
//                 for c in children {
//                     c.const_accept(self);
//                 }
//                 let right = self.base.digit_stack.borrow_mut().pop().unwrap();
//                 let left = self.base.digit_stack.borrow_mut().pop().unwrap();
//                 let dt = self.fif(
//                     (right - left).into(),
//                     NumericType::one(),
//                     NumericType::zero(),
//                 );
//                 self.dt_stack.borrow_mut().push(dt);
//                 Ok(())
//             }
//             Node::SuperiorOrEqual(children) => {
//                 for c in children {
//                     c.const_accept(self);
//                 }
//                 let right = self.base.digit_stack.borrow_mut().pop().unwrap();
//                 let left = self.base.digit_stack.borrow_mut().pop().unwrap();
//                 let dt = self.fif(
//                     (left - right).into(),
//                     NumericType::one(),
//                     NumericType::zero(),
//                 );
//                 self.dt_stack.borrow_mut().push(dt);
//                 Ok(())
//             }
//             Node::InferiorOrEqual(children) => {
//                 for c in children {
//                     c.const_accept(self);
//                 }
//                 let right = self.base.digit_stack.borrow_mut().pop().unwrap();
//                 let left = self.base.digit_stack.borrow_mut().pop().unwrap();
//                 let dt = self.fif(
//                     (right - left).into(),
//                     NumericType::one(),
//                     NumericType::zero(),
//                 );
//                 self.dt_stack.borrow_mut().push(dt);
//                 Ok(())
//             }
//             Node::Equal(children) => {
//                 for c in children {
//                     c.const_accept(self);
//                 }
//                 let right = self.base.digit_stack.borrow_mut().pop().unwrap();
//                 let left = self.base.digit_stack.borrow_mut().pop().unwrap();
//                 let diff = (right - left).abs();
//                 let dt = if diff < f64::EPSILON {
//                     NumericType::one()
//                 } else {
//                     NumericType::zero()
//                 };
//                 self.dt_stack.borrow_mut().push(dt);
//                 Ok(())
//             }
//             Node::NotEqual(children) => {
//                 for c in children {
//                     c.const_accept(self);
//                 }
//                 let right = self.base.digit_stack.borrow_mut().pop().unwrap();
//                 let left = self.base.digit_stack.borrow_mut().pop().unwrap();
//                 let diff = (right - left).abs();
//                 let dt = if diff >= f64::EPSILON {
//                     NumericType::one()
//                 } else {
//                     NumericType::zero()
//                 };
//                 self.dt_stack.borrow_mut().push(dt);
//                 Ok(())
//             }
//             Node::And(children) => {
//                 for c in children {
//                     c.const_accept(self);
//                 }
//                 let b = self.dt_stack.borrow_mut().pop().unwrap();
//                 let mut binding = self.dt_stack.borrow_mut();
//                 let a_ref = binding.last_mut().unwrap();
//                 *a_ref = (*a_ref * b).into();
//                 Ok(())
//             }
//             Node::Or(children) => {
//                 for c in children {
//                     c.const_accept(self);
//                 }
//                 let b = self.dt_stack.borrow_mut().pop().unwrap();
//                 let mut binding = self.dt_stack.borrow_mut();
//                 let a_ref = binding.last_mut().unwrap();
//                 *a_ref = (*a_ref + b - *a_ref * b).into();
//                 Ok(())
//             }
//             Node::Not(children) => {
//                 for c in children {
//                     c.const_accept(self);
//                 }
//                 let mut binding = self.dt_stack.borrow_mut();
//                 let top = binding.last_mut().unwrap();
//                 *top = (NumericType::one() - *top).into();
//                 Ok(())
//             }
//             Node::If(children, first_else, ..) => {
//                 // evaluate condition
//                 children[0].const_accept(self);
//                 let dt = self.dt_stack.borrow_mut().pop().unwrap();
//                 let last_condition = first_else.unwrap_or(children.len());

//                 // then branch
//                 self.push_weight(dt);
//                 for c in children.iter().skip(1).take(last_condition - 1) {
//                     c.const_accept(self);
//                 }
//                 self.pop_weight();

//                 // else branch
//                 if let Some(idx) = first_else {
//                     self.push_weight((-dt + 1.0).into());
//                     for c in children.iter().skip(*idx) {
//                         c.const_accept(self);
//                     }
//                     self.pop_weight();
//                 }
//                 Ok(())
//             }
//             Node::Assign(children) => {
//                 *self.base.is_lhs_variable.borrow_mut() = true;
//                 self.const_visit(children[0].clone())?;
//                 *self.base.is_lhs_variable.borrow_mut() = false;
//                 self.const_visit(children[1].clone())?;
//                 let v = self.base.lhs_variable.borrow_mut().clone().unwrap();
//                 let variable = v.as_ref();
//                 match variable {
//                     Node::Variable(_, name, index) => match index.get() {
//                         None => {
//                             return Err(ScriptingError::EvaluationError(format!(
//                                 "Variable {} not indexed",
//                                 name
//                             )))
//                         }
//                         Some(id) => {
//                             let mut variables = self.base.variables.borrow_mut();
//                             let weight = self.current_weight();
//                             if !self.base.boolean_stack.borrow().is_empty() {
//                                 let value = self.base.boolean_stack.borrow_mut().pop().unwrap();
//                                 if weight.value() >= 1.0 - f64::EPSILON {
//                                     variables[*id] = Value::Bool(value);
//                                 }
//                             } else if !self.base.string_stack.borrow().is_empty() {
//                                 let value = self.base.string_stack.borrow_mut().pop().unwrap();
//                                 if weight.value() >= 1.0 - f64::EPSILON {
//                                     variables[*id] = Value::String(value);
//                                 }
//                             } else if !self.base.array_stack.borrow().is_empty() {
//                                 let value = self.base.array_stack.borrow_mut().pop().unwrap();
//                                 if weight.value() >= 1.0 - f64::EPSILON {
//                                     variables[*id] = Value::Array(value);
//                                 }
//                             } else {
//                                 let value = self.base.digit_stack.borrow_mut().pop().unwrap();
//                                 if weight.value() >= 1.0 - f64::EPSILON {
//                                     variables[*id] = Value::Number(value);
//                                 } else {
//                                     let existing = variables
//                                         .get(*id)
//                                         .cloned()
//                                         .unwrap_or(Value::Number(NumericType::zero()));
//                                     if let Value::Number(old) = existing {
//                                         let new_val =
//                                             old * (NumericType::one() - weight) + value * weight;
//                                         variables[*id] = Value::Number(new_val.into());
//                                     } else {
//                                         variables[*id] = Value::Number((value * weight).into());
//                                     }
//                                 }
//                             }
//                             Ok(())
//                         }
//                     },
//                     _ => Err(ScriptingError::EvaluationError(
//                         "Invalid variable assignment".to_string(),
//                     )),
//                 }
//             }
//             _ => self.base.const_visit(node),
//         }
//     }
// }
