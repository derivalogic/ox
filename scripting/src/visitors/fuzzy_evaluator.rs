use std::cell::{Cell, RefCell};

use crate::prelude::*;
use crate::visitors::evaluator::{SingleScenarioEvaluator, Value};
use rustatlas::prelude::*;

/// Evaluator implementing a simple fuzzy logic mode using the
/// `fIf` smoothing kernel described in `docs/AGENTS.md`.
///
/// The evaluator behaves like `SingleScenarioEvaluator` but logical
/// operations return values in `[0,1]` and assignments inside `if`
/// blocks are weighted by these probabilities.
pub struct FuzzyEvaluator<'a> {
    base: SingleScenarioEvaluator<'a>,
    dt_stack: RefCell<Vec<NumericType>>, // condition truth values in [0,1]
    eps: f64,
    var_store0: RefCell<Vec<Vec<Value>>>,
    var_store1: RefCell<Vec<Vec<Value>>>,
    nested_if_lvl: Cell<usize>,
    max_nested_ifs: Cell<usize>,
}

impl<'a> FuzzyEvaluator<'a> {
    /// Create a new fuzzy evaluator with default epsilon = 1e-12.
    pub fn new() -> Self {
        Self {
            base: SingleScenarioEvaluator::new(),
            dt_stack: RefCell::new(Vec::new()),
            eps: 1e-12,
            var_store0: RefCell::new(Vec::new()),
            var_store1: RefCell::new(Vec::new()),
            nested_if_lvl: Cell::new(0),
            max_nested_ifs: Cell::new(0),
        }
    }

    /// Set market scenario for market-data dependent nodes.
    pub fn with_scenario(mut self, scenario: &'a Scenario) -> Self {
        self.base = self.base.with_scenario(scenario);
        self
    }

    /// Pre-allocate variable storage.
    pub fn with_variables(mut self, n: usize) -> Self {
        self.base = self.base.with_variables(n);

        let depth = self.max_nested_ifs.get();
        {
            let mut s0 = self.var_store0.borrow_mut();
            s0.resize_with(depth, || vec![Value::Null; n]);
            for vec in s0.iter_mut() {
                vec.resize(n, Value::Null);
            }
        }
        {
            let mut s1 = self.var_store1.borrow_mut();
            s1.resize_with(depth, || vec![Value::Null; n]);
            for vec in s1.iter_mut() {
                vec.resize(n, Value::Null);
            }
        }
        self
    }

    /// Configure maximum nested `if` depth for variable storage.
    pub fn with_max_nested_ifs(mut self, depth: usize) -> Self {
        self.max_nested_ifs.set(depth);
        let n = self.base.variables.borrow().len();
        {
            let mut s0 = self.var_store0.borrow_mut();
            s0.resize_with(depth, || vec![Value::Null; n]);
            for vec in s0.iter_mut() {
                vec.resize(n, Value::Null);
            }
        }
        {
            let mut s1 = self.var_store1.borrow_mut();
            s1.resize_with(depth, || vec![Value::Null; n]);
            for vec in s1.iter_mut() {
                vec.resize(n, Value::Null);
            }
        }
        self
    }

    /// Current variables after evaluation.
    pub fn variables(&self) -> Vec<Value> {
        self.base.variables()
    }

    /// Access numeric stack (mainly for tests).
    pub fn digit_stack(&self) -> Vec<NumericType> {
        self.base.digit_stack()
    }

    fn fif(&self, x: NumericType, a: NumericType, b: NumericType) -> NumericType {
        let half = self.eps * 0.5;
        let inner = (x + half)
            .min(NumericType::from(self.eps))
            .max(NumericType::zero());
        let res = b.clone() + ((a - b) / self.eps) * inner;
        res.into()
    }
}

impl<'a> NodeConstVisitor for FuzzyEvaluator<'a> {
    type Output = Result<()>;

    fn const_visit(&self, node: &Node) -> Self::Output {
        match node {
            Node::True => {
                self.dt_stack.borrow_mut().push(NumericType::one());
                Ok(())
            }
            Node::False => {
                self.dt_stack.borrow_mut().push(NumericType::zero());
                Ok(())
            }
            Node::Superior(data) => {
                data.children.iter().try_for_each(|c| self.const_visit(c))?;
                let right = self.base.digit_stack.borrow_mut().pop().unwrap();
                let left = self.base.digit_stack.borrow_mut().pop().unwrap();
                let dt = self.fif((left - right).into(), NumericType::one(), NumericType::zero());
                self.dt_stack.borrow_mut().push(dt);
                Ok(())
            }
            Node::Inferior(data) => {
                data.children.iter().try_for_each(|c| self.const_visit(c))?;
                let right = self.base.digit_stack.borrow_mut().pop().unwrap();
                let left = self.base.digit_stack.borrow_mut().pop().unwrap();
                let dt = self.fif((right - left).into(), NumericType::one(), NumericType::zero());
                self.dt_stack.borrow_mut().push(dt);
                Ok(())
            }
            Node::SuperiorOrEqual(data) => {
                data.children.iter().try_for_each(|c| self.const_visit(c))?;
                let right = self.base.digit_stack.borrow_mut().pop().unwrap();
                let left = self.base.digit_stack.borrow_mut().pop().unwrap();
                let dt = self.fif((left - right).into(), NumericType::one(), NumericType::zero());
                self.dt_stack.borrow_mut().push(dt);
                Ok(())
            }
            Node::InferiorOrEqual(data) => {
                data.children.iter().try_for_each(|c| self.const_visit(c))?;
                let right = self.base.digit_stack.borrow_mut().pop().unwrap();
                let left = self.base.digit_stack.borrow_mut().pop().unwrap();
                let dt = self.fif((right - left).into(), NumericType::one(), NumericType::zero());
                self.dt_stack.borrow_mut().push(dt);
                Ok(())
            }
            Node::Equal(data) => {
                data.children.iter().try_for_each(|c| self.const_visit(c))?;
                let right = self.base.digit_stack.borrow_mut().pop().unwrap();
                let left = self.base.digit_stack.borrow_mut().pop().unwrap();
                let diff = (right - left).abs();
                let dt = if diff < f64::EPSILON {
                    NumericType::one()
                } else {
                    NumericType::zero()
                };
                self.dt_stack.borrow_mut().push(dt);
                Ok(())
            }
            Node::NotEqual(data) => {
                data.children.iter().try_for_each(|c| self.const_visit(c))?;
                let right = self.base.digit_stack.borrow_mut().pop().unwrap();
                let left = self.base.digit_stack.borrow_mut().pop().unwrap();
                let diff = (right - left).abs();
                let dt = if diff >= f64::EPSILON {
                    NumericType::one()
                } else {
                    NumericType::zero()
                };
                self.dt_stack.borrow_mut().push(dt);
                Ok(())
            }
            Node::And(data) => {
                data.children.iter().try_for_each(|c| self.const_visit(c))?;
                let b = self.dt_stack.borrow_mut().pop().unwrap();
                let mut binding = self.dt_stack.borrow_mut();
                let a_ref = binding.last_mut().unwrap();
                *a_ref = (*a_ref * b).into();
                Ok(())
            }
            Node::Or(data) => {
                data.children.iter().try_for_each(|c| self.const_visit(c))?;
                let b = self.dt_stack.borrow_mut().pop().unwrap();
                let mut binding = self.dt_stack.borrow_mut();
                let a_ref = binding.last_mut().unwrap();
                *a_ref = (*a_ref + b - *a_ref * b).into();
                Ok(())
            }
            Node::Not(data) => {
                data.children.iter().try_for_each(|c| self.const_visit(c))?;
                let mut binding = self.dt_stack.borrow_mut();
                let top = binding.last_mut().unwrap();
                *top = (NumericType::one() - *top).into();
                Ok(())
            }
            Node::If(data) => {
                // evaluate condition
                self.const_visit(&data.children[0])?;
                let dt = self.dt_stack.borrow_mut().pop().unwrap();
                let last = data.first_else.unwrap_or(data.children.len());

                let lvl = self.nested_if_lvl.get();
                self.nested_if_lvl.set(lvl + 1);

                if dt.value() >= 1.0 - self.eps {
                    for c in data.children.iter().skip(1).take(last - 1) {
                        self.const_visit(c)?;
                    }
                } else if dt.value() <= self.eps {
                    if let Some(start) = data.first_else {
                        for c in data.children.iter().skip(start) {
                            self.const_visit(c)?;
                        }
                    }
                } else {
                    {
                        let vars = self.base.variables.borrow();
                        let mut store = self.var_store0.borrow_mut();
                        if lvl >= store.len() {
                            let n = vars.len();
                            store.resize_with(lvl + 1, || vec![Value::Null; n]);
                        }
                        for &idx in &data.affected_vars {
                            if idx >= store[lvl].len() {
                                store[lvl].resize(idx + 1, Value::Null);
                            }
                            store[lvl][idx] = vars[idx].clone();
                        }
                    }

                    for c in data.children.iter().skip(1).take(last - 1) {
                        self.const_visit(c)?;
                    }

                    {
                        let mut vars = self.base.variables.borrow_mut();
                        let s0 = self.var_store0.borrow();
                        let mut s1 = self.var_store1.borrow_mut();
                        if lvl >= s1.len() {
                            let n = vars.len();
                            s1.resize_with(lvl + 1, || vec![Value::Null; n]);
                        }
                        for &idx in &data.affected_vars {
                            if idx >= s1[lvl].len() {
                                s1[lvl].resize(idx + 1, Value::Null);
                            }
                            s1[lvl][idx] = vars[idx].clone();
                            vars[idx] = s0[lvl][idx].clone();
                        }
                    }

                    if let Some(start) = data.first_else {
                        for c in data.children.iter().skip(start) {
                            self.const_visit(c)?;
                        }
                    }

                    {
                        let mut vars = self.base.variables.borrow_mut();
                        let s1 = self.var_store1.borrow();
                        for &idx in &data.affected_vars {
                            let true_v = s1[lvl][idx].clone();
                            let false_v = vars[idx].clone();
                            vars[idx] = match (true_v, false_v) {
                                (Value::Number(a), Value::Number(b)) => {
                                    let new_val = a * dt + b * (NumericType::one() - dt);
                                    Value::Number(new_val.into())
                                }
                                (t, f) => {
                                    if dt.value() >= 0.5 {
                                        t
                                    } else {
                                        f
                                    }
                                }
                            };
                        }
                    }
                }

                self.nested_if_lvl.set(lvl);
                Ok(())
            }
            Node::Assign(_) => self.base.const_visit(node),
            _ => self.base.const_visit(node),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::visitors::ifprocessor::IfProcessor;

    #[test]
    fn test_basic_assignment() {
        let script = "x = 1; y = x + 2;".to_string();
        let tokens = Lexer::new(script).tokenize().unwrap();
        let mut nodes = Parser::new(tokens).parse().unwrap();

        let indexer = VarIndexer::new();
        indexer.visit(&mut nodes).unwrap();

        let processor = IfProcessor::new();
        processor.visit(&mut nodes).unwrap();

        let evaluator = FuzzyEvaluator::new()
            .with_variables(indexer.get_variables_size())
            .with_max_nested_ifs(processor.max_nested_ifs());
        evaluator.const_visit(&nodes).unwrap();

        assert_eq!(
            evaluator.variables(),
            vec![
                Value::Number(NumericType::new(1.0)),
                Value::Number(NumericType::new(3.0)),
            ]
        );
    }

    #[test]
    fn test_simple_if_condition() {
        let script = "x = 1; if x > 0 { x = 2; }".to_string();
        let tokens = Lexer::new(script).tokenize().unwrap();
        let mut nodes = Parser::new(tokens).parse().unwrap();

        let indexer = VarIndexer::new();
        indexer.visit(&mut nodes).unwrap();

        let processor = IfProcessor::new();
        processor.visit(&mut nodes).unwrap();

        let evaluator = FuzzyEvaluator::new()
            .with_variables(indexer.get_variables_size())
            .with_max_nested_ifs(processor.max_nested_ifs());
        evaluator.const_visit(&nodes).unwrap();

        assert_eq!(
            evaluator.variables(),
            vec![Value::Number(NumericType::new(2.0))]
        );
    }

}
