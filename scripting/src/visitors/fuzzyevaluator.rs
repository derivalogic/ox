use std::cell::{Cell, RefCell};

use crate::prelude::*;
use crate::visitors::evaluator::Value;
use rustatlas::prelude::*;

const EPS: f64 = 1.0e-12;
const ONE_MINUS_EPS: f64 = 1.0 - EPS;

pub struct FuzzyEvaluator<'a> {
    variables: RefCell<Vec<Value>>,
    digit_stack: RefCell<Vec<NumericType>>,
    boolean_stack: RefCell<Vec<bool>>,
    string_stack: RefCell<Vec<String>>,
    array_stack: RefCell<Vec<Vec<Value>>>,
    is_lhs_variable: RefCell<bool>,
    lhs_variable: RefCell<Option<Node>>,
    scenario: Option<&'a Scenario>,
    current_event: RefCell<usize>,

    /// Stack of truth degrees (`dt`) produced while evaluating conditions.
    dt_stack: RefCell<Vec<NumericType>>,

    /// Default smoothing width (ε) when a node does not override it.
    eps: f64,

    /// Temporary variable stores per *nested-if* level.
    /// `[level][var_index]`
    var_store0: RefCell<Vec<Vec<NumericType>>>,
    var_store1: RefCell<Vec<Vec<NumericType>>>,

    /// Current *nested-if* depth (0 = outside any `if`).
    nested_if_lvl: Cell<usize>,
}

impl<'a> FuzzyEvaluator<'a> {
    /* ───────────────────────── constructors ───────────────────────── */

    pub fn new(n_vars: usize, max_nested_ifs: usize) -> Self {
        let mut var_store0 = Vec::with_capacity(max_nested_ifs);
        let mut var_store1 = Vec::with_capacity(max_nested_ifs);
        for _ in 0..max_nested_ifs {
            var_store0.push(vec![NumericType::zero(); n_vars]);
            var_store1.push(vec![NumericType::zero(); n_vars]);
        }
        Self {
            variables: RefCell::new(vec![Value::Null; n_vars]),
            digit_stack: RefCell::new(Vec::new()),
            boolean_stack: RefCell::new(Vec::new()),
            string_stack: RefCell::new(Vec::new()),
            array_stack: RefCell::new(Vec::new()),
            is_lhs_variable: RefCell::new(false),
            lhs_variable: RefCell::new(None),
            scenario: None,
            current_event: RefCell::new(0),
            dt_stack: RefCell::new(Vec::new()),
            eps: EPS,
            var_store0: RefCell::new(var_store0),
            var_store1: RefCell::new(var_store1),
            nested_if_lvl: Cell::new(0),
        }
    }

    pub fn with_eps(mut self, eps: f64) -> Self {
        self.eps = eps;
        self
    }

    pub fn with_scenario(mut self, scenario: &'a Scenario) -> Self {
        self.scenario = Some(scenario);
        self
    }

    /* ─────────────────────── public accessors ─────────────────────── */

    pub fn variables(&self) -> Vec<Value> {
        self.variables.borrow().clone()
    }

    pub fn with_variables(self, n: usize) -> Self {
        self.variables.borrow_mut().resize(n, Value::Null);
        self
    }

    pub fn with_current_event(self, event: usize) -> Self {
        *self.current_event.borrow_mut() = event;
        self
    }

    pub fn current_market_data(&self) -> Result<&SimulationData> {
        let scenario = self
            .scenario
            .ok_or(ScriptingError::EvaluationError("No scenario set".into()))?;
        scenario
            .get(*self.current_event.borrow())
            .ok_or(ScriptingError::EvaluationError("Event not found".into()))
    }

    pub fn current_event(&self) -> usize {
        *self.current_event.borrow()
    }

    pub fn set_current_event(&self, event: usize) {
        *self.current_event.borrow_mut() = event;
    }

    pub fn set_variable(&self, idx: usize, val: Value) {
        let mut vars = self.variables.borrow_mut();
        if idx >= vars.len() {
            vars.resize(idx + 1, Value::Null);
        }
        vars[idx] = val;
    }

    pub fn digit_stack(&self) -> Vec<NumericType> {
        self.digit_stack.borrow().clone()
    }

    pub fn boolean_stack(&self) -> Vec<bool> {
        self.boolean_stack.borrow().clone()
    }

    /// Call-spread centred on 0, width `eps`.
    fn c_spr(&self, x: NumericType, eps: f64) -> NumericType {
        let half = eps * 0.5;
        if x < -half {
            NumericType::zero()
        } else if x > half {
            NumericType::one()
        } else {
            ((x + half) / eps).into()
        }
    }

    /// Call-spread on explicit bounds `[lb, rb]`.
    fn c_spr_bounds(&self, x: NumericType, lb: f64, rb: f64) -> NumericType {
        if x < lb {
            NumericType::zero()
        } else if x > rb {
            NumericType::one()
        } else {
            ((x - lb) / (rb - lb)).into()
        }
    }

    /// Butterfly centred on 0, width `eps`.
    fn bfly(&self, x: NumericType, eps: f64) -> NumericType {
        let half = eps * 0.5;
        if x < -half || x > half {
            NumericType::zero()
        } else {
            ((-x.abs() + half) / half).into()
        }
    }

    /// Butterfly with explicit bounds `lb < 0 < rb`.
    fn bfly_bounds(&self, x: NumericType, lb: f64, rb: f64) -> NumericType {
        if x < lb || x > rb {
            NumericType::zero()
        } else if x < 0.0 {
            (NumericType::one() - x / lb).into()
        } else {
            (NumericType::one() - x / rb).into()
        }
    }
}

impl<'a> NodeConstVisitor for FuzzyEvaluator<'a> {
    type Output = Result<()>;

    fn const_visit(&self, node: &Node) -> Self::Output {
        match node {
            /* ─────────────── base / variables ─────────────── */
            Node::Base(data) => {
                for child in &data.children {
                    self.const_visit(child)?;
                }
                Ok(())
            }
            Node::Variable(data) => {
                let name = &data.name;
                if *self.is_lhs_variable.borrow() {
                    *self.lhs_variable.borrow_mut() = Some(node.clone());
                    Ok(())
                } else {
                    match data.id {
                        None => Err(ScriptingError::EvaluationError(format!(
                            "Variable {} not indexed",
                            name
                        ))),
                        Some(id) => {
                            let vars = self.variables.borrow();
                            let value = vars.get(id).unwrap();
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
            Node::Spot(data) => {
                let id = data
                    .id
                    .ok_or(ScriptingError::EvaluationError("Spot not indexed".into()))?;
                let market_data = self
                    .scenario
                    .ok_or(ScriptingError::EvaluationError("No scenario set".into()))?
                    .get(*self.current_event.borrow())
                    .ok_or(ScriptingError::EvaluationError("Spot not found".into()))?;
                self.digit_stack.borrow_mut().push(market_data.get_fx(id)?);
                Ok(())
            }
            Node::Df(data) => {
                let id = data
                    .id
                    .ok_or(ScriptingError::EvaluationError("Df not indexed".into()))?;
                let market_data = self
                    .scenario
                    .ok_or(ScriptingError::EvaluationError("No scenario set".into()))?
                    .get(*self.current_event.borrow())
                    .ok_or(ScriptingError::EvaluationError("Df not found".into()))?;
                self.digit_stack.borrow_mut().push(market_data.get_df(id)?);
                Ok(())
            }
            Node::RateIndex(data) => {
                let id = data.id.ok_or(ScriptingError::EvaluationError(
                    "RateIndex not indexed".into(),
                ))?;
                let market_data = self
                    .scenario
                    .ok_or(ScriptingError::EvaluationError("No scenario set".into()))?
                    .get(*self.current_event.borrow())
                    .ok_or(ScriptingError::EvaluationError(
                        "RateIndex not found".into(),
                    ))?;
                self.digit_stack.borrow_mut().push(market_data.get_fwd(id)?);
                Ok(())
            }
            Node::Pays(data) => {
                for child in &data.children {
                    self.const_visit(child)?;
                }
                let market_data = self
                    .scenario
                    .ok_or(ScriptingError::EvaluationError("No scenario set".into()))?
                    .get(*self.current_event.borrow())
                    .ok_or(ScriptingError::EvaluationError("Event not found".into()))?
                    .clone();
                let current_value = self.digit_stack.borrow_mut().pop().unwrap();
                let df_id = data
                    .df_id
                    .ok_or(ScriptingError::EvaluationError("Pays not indexed".into()))?;
                let df = market_data.get_df(df_id)?;
                let numerarie = market_data.numerarie();
                let value: NumericType = if data.currency.is_some() {
                    let fx_id = data.spot_id.ok_or(ScriptingError::EvaluationError(
                        "Pays FX not indexed".into(),
                    ))?;
                    let fx = market_data.get_fx(fx_id)?;
                    ((current_value * df * fx) / numerarie).into()
                } else {
                    ((current_value * df) / numerarie).into()
                };
                self.digit_stack.borrow_mut().push(value);
                Ok(())
            }
            Node::Constant(data) => {
                self.digit_stack.borrow_mut().push(data.const_value.into());
                Ok(())
            }
            Node::String(value) => {
                self.string_stack.borrow_mut().push(value.clone());
                Ok(())
            }

            /* ─────────────── math ops ─────────────── */
            Node::Add(data) => {
                for child in &data.children {
                    self.const_visit(child)?;
                }
                let right = self.digit_stack.borrow_mut().pop().unwrap();
                let left = self.digit_stack.borrow_mut().pop().unwrap();
                self.digit_stack.borrow_mut().push((left + right).into());
                Ok(())
            }
            Node::Subtract(data) => {
                for child in &data.children {
                    self.const_visit(child)?;
                }
                let right = self.digit_stack.borrow_mut().pop().unwrap();
                let left = self.digit_stack.borrow_mut().pop().unwrap();
                self.digit_stack.borrow_mut().push((left - right).into());
                Ok(())
            }
            Node::Multiply(data) => {
                for child in &data.children {
                    self.const_visit(child)?;
                }
                let right = self.digit_stack.borrow_mut().pop().unwrap();
                let left = self.digit_stack.borrow_mut().pop().unwrap();
                self.digit_stack.borrow_mut().push((left * right).into());
                Ok(())
            }
            Node::Divide(data) => {
                for child in &data.children {
                    self.const_visit(child)?;
                }
                let right = self.digit_stack.borrow_mut().pop().unwrap();
                let left = self.digit_stack.borrow_mut().pop().unwrap();
                self.digit_stack.borrow_mut().push((left / right).into());
                Ok(())
            }
            Node::Assign(data) => {
                *self.is_lhs_variable.borrow_mut() = true;
                self.const_visit(&data.children[0])?;
                *self.is_lhs_variable.borrow_mut() = false;
                self.const_visit(&data.children[1])?;

                let variable = self.lhs_variable.borrow_mut().clone().unwrap();
                if let Node::Variable(var_data) = variable {
                    let id = var_data.id.ok_or(ScriptingError::EvaluationError(format!(
                        "Variable {} not indexed",
                        var_data.name
                    )))?;
                    let mut vars = self.variables.borrow_mut();
                    if !self.boolean_stack.borrow().is_empty() {
                        vars[id] = Value::Bool(self.boolean_stack.borrow_mut().pop().unwrap());
                    } else if !self.string_stack.borrow().is_empty() {
                        vars[id] = Value::String(self.string_stack.borrow_mut().pop().unwrap());
                    } else if !self.array_stack.borrow().is_empty() {
                        vars[id] = Value::Array(self.array_stack.borrow_mut().pop().unwrap());
                    } else {
                        vars[id] = Value::Number(self.digit_stack.borrow_mut().pop().unwrap());
                    }
                    Ok(())
                } else {
                    Err(ScriptingError::EvaluationError(
                        "Invalid variable assignment".into(),
                    ))
                }
            }
            Node::NotEqual(data) => {
                for child in &data.children {
                    self.const_visit(child)?;
                }
                let right = self.digit_stack.borrow_mut().pop().unwrap();
                let left = self.digit_stack.borrow_mut().pop().unwrap();
                self.boolean_stack
                    .borrow_mut()
                    .push((right - left).abs() >= f64::EPSILON);
                Ok(())
            }

            /* ─────────────── literals ─────────────── */
            Node::True => {
                self.dt_stack.borrow_mut().push(NumericType::one());
                Ok(())
            }
            Node::False => {
                self.dt_stack.borrow_mut().push(NumericType::zero());
                Ok(())
            }

            /* ─────────────── comparison ─────────────── */
            Node::Equal(data) => {
                self.const_visit(&data.children[0])?;
                let expr = self.digit_stack.borrow_mut().pop().unwrap();

                let dt = if data.discrete {
                    self.bfly_bounds(expr, data.lb, data.rb)
                } else {
                    self.bfly(expr, self.eps)
                };
                self.dt_stack.borrow_mut().push(dt);
                Ok(())
            }

            Node::Superior(data) | Node::SuperiorOrEqual(data) => {
                self.const_visit(&data.children[0])?;
                let expr = self.digit_stack.borrow_mut().pop().unwrap();

                let dt = if data.discrete {
                    self.c_spr_bounds(expr, data.lb, data.rb)
                } else {
                    self.c_spr(expr, self.eps)
                };
                self.dt_stack.borrow_mut().push(dt);
                Ok(())
            }

            /* ─────────────── combinators ─────────────── */
            Node::And(data) => {
                self.const_visit(&data.children[0])?;
                self.const_visit(&data.children[1])?;
                let b2 = self.dt_stack.borrow_mut().pop().unwrap();
                let b1 = self.dt_stack.borrow_mut().pop().unwrap();
                let res: NumericType = (b1 * b2).into();
                self.dt_stack.borrow_mut().push(res);
                Ok(())
            }
            Node::Or(data) => {
                self.const_visit(&data.children[0])?;
                self.const_visit(&data.children[1])?;
                let b2 = self.dt_stack.borrow_mut().pop().unwrap();
                let b1 = self.dt_stack.borrow_mut().pop().unwrap();
                let dt: NumericType = (b1 + b2 - (b1 * b2)).into();
                self.dt_stack.borrow_mut().push(dt);
                Ok(())
            }
            Node::Not(data) => {
                self.const_visit(&data.children[0])?;
                let b = self.dt_stack.borrow_mut().pop().unwrap();
                let dt: NumericType = (NumericType::one() - b).into();
                self.dt_stack.borrow_mut().push(dt);
                Ok(())
            }

            /* ─────────────── if / else ─────────────── */
            Node::If(data) => {
                // keep 1-based depth like the C++ code
                self.nested_if_lvl.set(self.nested_if_lvl.get() + 1);
                let last_true = data.first_else.unwrap_or(data.children.len()) - 1;

                /* ── evaluate condition ── */
                self.const_visit(&data.children[0])?;
                let dt = self.dt_stack.borrow_mut().pop().unwrap();

                /* ── dt ≈ true ── */
                if dt.value() > ONE_MINUS_EPS {
                    for c in data.children.iter().skip(1).take(last_true) {
                        self.const_visit(c)?;
                    }
                }
                /* ── dt ≈ false ── */
                else if dt.value() < EPS {
                    if let Some(start) = data.first_else {
                        for c in data.children.iter().skip(start) {
                            self.const_visit(c)?;
                        }
                    }
                }
                /* ── fuzzy branch ── */
                else {
                    /* backup current values */

                    let store0 = &mut self.var_store0.borrow_mut()[self.nested_if_lvl.get() - 1];

                    data.affected_vars.iter().for_each(|&idx| {
                        store0[idx] = match self.variables.borrow()[idx] {
                            Value::Number(n) => n,
                            _ => panic!("expected numeric var"),
                        }
                    });

                    /* evaluate “then”-branch */
                    for c in data.children.iter().skip(1).take(last_true) {
                        self.const_visit(c)?;
                    }

                    /* record “then” result and restore backup */

                    let store1 = &mut self.var_store1.borrow_mut()[self.nested_if_lvl.get() - 1];
                    data.affected_vars.iter().for_each(|&idx| {
                        let v = match self.variables.borrow()[idx] {
                            Value::Number(n) => n,
                            _ => panic!("expected numeric var"),
                        };
                        store1[idx] = v;
                        self.variables.borrow_mut()[idx] = Value::Number(store0[idx]);
                    });

                    /* evaluate “else”-branch (if any) */
                    if let Some(start) = data.first_else {
                        for c in data.children.iter().skip(start) {
                            self.const_visit(c)?;
                        }
                    }

                    /* final fuzzy blend */

                    data.affected_vars.iter().for_each(|&idx| {
                        let v_true = store1[idx];
                        let v_false = match self.variables.borrow()[idx] {
                            Value::Number(n) => n,
                            _ => panic!("expected numeric var"),
                        };
                        let v = Value::Number((dt * v_true + (-dt + 1.0) * v_false).into());
                        self.variables.borrow_mut()[idx] = v;
                    });
                }

                /* leave this `if` */
                self.nested_if_lvl.set(self.nested_if_lvl.get() - 1);
                Ok(())
            }

            /* ─────────────── unhandled ─────────────── */
            _ => Err(ScriptingError::EvaluationError(
                "Node not implemented".into(),
            )),
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

        let evaluator =
            FuzzyEvaluator::new(indexer.get_variables_size(), processor.max_nested_ifs());
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

        let evaluator =
            FuzzyEvaluator::new(indexer.get_variables_size(), processor.max_nested_ifs());

        evaluator.const_visit(&nodes).unwrap();

        assert_eq!(
            evaluator.variables(),
            vec![Value::Number(NumericType::new(2.0))]
        );
    }

    #[test]
    fn test_simple_if_condition2() {
        let script = "x = 0; if x-1 > 0 { x = 2; }".to_string();
        let tokens = Lexer::new(script).tokenize().unwrap();
        let mut nodes = Parser::new(tokens).parse().unwrap();

        let indexer = VarIndexer::new();
        indexer.visit(&mut nodes).unwrap();

        let processor = IfProcessor::new();
        processor.visit(&mut nodes).unwrap();

        let evaluator =
            FuzzyEvaluator::new(indexer.get_variables_size(), processor.max_nested_ifs());

        evaluator.const_visit(&nodes).unwrap();

        assert_eq!(
            evaluator.variables(),
            vec![Value::Number(NumericType::new(0.0))]
        );
    }

    #[test]
    fn test_fuzzy_case() {
        Tape::start_recording();

        let script1 = "x = 0.0; y = 0; if x > 0 { y = 1; }".to_string();
        let tokens = Lexer::new(script1).tokenize().unwrap();
        let mut script1_nodes = Parser::new(tokens).parse().unwrap();

        let script2 = "x = 0; y = fif(x,1,0,0.0001);".to_string();
        let tokens2 = Lexer::new(script2).tokenize().unwrap();
        let mut script2_nodes = Parser::new(tokens2).parse().unwrap();
        let indexer = VarIndexer::new();
        indexer.visit(&mut script1_nodes).unwrap();

        let if_processor = IfProcessor::new();
        if_processor.visit(&mut script1_nodes).unwrap();
        let domain_processor = DomainProcessor::new(indexer.get_variables_size());
        domain_processor.visit(&mut script1_nodes).unwrap();

        let fuzzy_evaluator =
            FuzzyEvaluator::new(indexer.get_variables_size(), if_processor.max_nested_ifs())
                .with_eps(0.0001);

        fuzzy_evaluator.const_visit(&script1_nodes).unwrap();

        let eval_vars = fuzzy_evaluator.variables();
        match eval_vars.get(1).unwrap() {
            Value::Number(n) => {
                println!("y = {:?}", n);
                n.backward().unwrap();
            }
            _ => panic!("Expected y to be a number"),
        }

        let result = match eval_vars.get(0).unwrap() {
            Value::Number(n) => {
                println!("x = {:?}", n);
                n.adjoint().unwrap()
            }
            _ => panic!("Expected x to be a number"),
        };

        indexer.clear();
        indexer.visit(&mut script2_nodes).unwrap();
        let evaluator = SingleScenarioEvaluator::new().with_variables(indexer.get_variables_size());
        evaluator.const_visit(&script2_nodes).unwrap();

        let eval_vars2 = evaluator.variables();

        match eval_vars2.get(0).unwrap() {
            Value::Number(n) => {
                n.backward().unwrap();
            }
            _ => panic!("Expected x to be a number"),
        }
        let result2 = match eval_vars2.get(0).unwrap() {
            Value::Number(n) => n.adjoint().unwrap(),
            _ => panic!("Expected fif result to be a number"),
        };
        assert!(
            (result  - result2).abs() < 1e-6,
            "Results do not match"
        );

        Tape::stop_recording();
    }
}
