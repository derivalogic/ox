// src/visitors/evaluator/fuzzy_evaluator.rs
use std::cell::{Cell, RefCell};

use crate::prelude::*;
use crate::visitors::evaluator::{SingleScenarioEvaluator, Value};
use rustatlas::prelude::*;

const EPS: f64 = 1.0e-12;
const ONE_MINUS_EPS: f64 = 0.999_999_999_999;

/// Evaluator implementing the “simple fuzzy logic” mode described in
/// `docs/AGENTS.md` (Antoine Savine, *Modern Computational Finance*).
pub struct FuzzyEvaluator<'a> {
    base: SingleScenarioEvaluator<'a>,

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

    pub fn new() -> Self {
        Self {
            base: SingleScenarioEvaluator::new(),
            dt_stack: RefCell::new(Vec::new()),
            eps: EPS,
            var_store0: RefCell::new(Vec::new()),
            var_store1: RefCell::new(Vec::new()),
            nested_if_lvl: Cell::new(0),
        }
    }

    pub fn with_eps(mut self, eps: f64) -> Self {
        self.eps = eps;
        self
    }

    pub fn with_scenario(mut self, scenario: &'a Scenario) -> Self {
        self.base = self.base.with_scenario(scenario);
        self
    }

    /* ─────────────────────── public accessors ─────────────────────── */

    pub fn variables(&self) -> Vec<Value> {
        self.base.variables()
    }

    pub fn with_variables(mut self, n: usize) -> Self {
        self.base = self.base.with_variables(n);
        self
    }

    /* ──────────────────────── fIf primitives ──────────────────────── */

    fn fif(&self, x: NumericType, a: NumericType, b: NumericType, eps: f64) -> NumericType {
        let half = eps * 0.5;
        let inner = (x + half)
            .min(NumericType::from(eps))
            .max(NumericType::zero());
        (b + ((a - b) / eps) * inner).into()
    }

    /// Call-spread centred on 0, width `eps`.
    fn c_spr(&self, x: NumericType, eps: f64) -> NumericType {
        let half = eps * 0.5;
        ((x + half)
            .min(NumericType::from(eps))
            .max(NumericType::zero())
            / eps)
            .into()
    }

    /// Call-spread on explicit bounds `[lb, rb]`.
    fn c_spr_bounds(&self, x: NumericType, lb: f64, rb: f64) -> NumericType {
        ((x - NumericType::from(lb))
            .min(NumericType::from(rb - lb))
            .max(NumericType::zero())
            / (rb - lb))
            .into()
    }

    /// Butterfly centred on 0, width `eps`.
    fn bfly(&self, x: NumericType, eps: f64) -> NumericType {
        let half = eps * 0.5;
        let inner = NumericType::from(half) - x.abs();
        (inner.max(NumericType::zero()) / half).into()
    }

    /// Butterfly with explicit bounds `lb < 0 < rb`.
    fn bfly_bounds(&self, x: NumericType, lb: f64, rb: f64) -> NumericType {
        if x.value() < lb || x.value() > rb {
            NumericType::zero()
        } else if x.value() < 0.0 {
            (NumericType::one() - x / lb).into()
        } else {
            (NumericType::one() - x / rb).into()
        }
    }

    /* ────────────────────────── utilities ─────────────────────────── */

    /// Grows `var_store0`/`var_store1` so they contain a store for the
    /// current `nested_if_lvl`.  Called **after** the level is incremented.
    fn ensure_level(&self) {
        let lvl = self.nested_if_lvl.get(); // 1-based inside an `if`
        let n_vars = self.base.variables.borrow().len();

        let mut s0 = self.var_store0.borrow_mut();
        if s0.len() < lvl {
            s0.push(vec![NumericType::zero(); n_vars]);
        }
        let mut s1 = self.var_store1.borrow_mut();
        if s1.len() < lvl {
            s1.push(vec![NumericType::zero(); n_vars]);
        }
    }
}

impl<'a> NodeConstVisitor for FuzzyEvaluator<'a> {
    type Output = Result<()>;

    fn const_visit(&self, node: &Node) -> Self::Output {
        match node {
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
                let expr = self.base.digit_stack.borrow_mut().pop().unwrap();

                // node-specific ε overrides the default when present
                // let eps = data.eps.unwrap_or(self.eps);
                let eps = self.eps;

                let dt = if data.discrete {
                    self.bfly_bounds(expr, data.lb, data.rb)
                } else {
                    self.bfly(expr, eps)
                };
                self.dt_stack.borrow_mut().push(dt);
                Ok(())
            }

            Node::Superior(data) | Node::SuperiorOrEqual(data) => {
                self.const_visit(&data.children[0])?;
                let expr = self.base.digit_stack.borrow_mut().pop().unwrap();

                // let eps = data.eps.unwrap_or(self.eps);
                let eps = self.eps;

                let dt = if data.discrete {
                    self.c_spr_bounds(expr, data.lb, data.rb)
                } else {
                    self.c_spr(expr, eps)
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
                self.dt_stack.borrow_mut().push((b1 * b2).into());
                Ok(())
            }
            Node::Or(data) => {
                self.const_visit(&data.children[0])?;
                self.const_visit(&data.children[1])?;
                let b2 = self.dt_stack.borrow_mut().pop().unwrap();
                let b1 = self.dt_stack.borrow_mut().pop().unwrap();
                self.dt_stack
                    .borrow_mut()
                    .push((b1 + b2 - (b1 * b2)).into());
                Ok(())
            }
            Node::Not(data) => {
                self.const_visit(&data.children[0])?;
                let b = self.dt_stack.borrow_mut().pop().unwrap();
                self.dt_stack
                    .borrow_mut()
                    .push((NumericType::one() - b).into());
                Ok(())
            }

            /* ─────────────── if / else ─────────────── */
            Node::If(data) => {
                // keep 1-based depth like the C++ code
                self.nested_if_lvl.set(self.nested_if_lvl.get() + 1);
                self.ensure_level();

                /* ── evaluate condition ── */
                let last_true = data.first_else.unwrap_or(data.children.len());
                self.const_visit(&data.children[0])?;
                let dt = self.dt_stack.borrow_mut().pop().unwrap();

                /* ── dt ≈ true ── */
                if dt.value() > ONE_MINUS_EPS {
                    for c in data.children.iter().skip(1).take(last_true - 1) {
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
                    {
                        let mut store0 = self.var_store0.borrow_mut();
                        let backup = &mut store0[self.nested_if_lvl.get() - 1];
                        data.affected_vars.iter().for_each(|&idx| {
                            backup[idx] = match self.base.variables.borrow()[idx] {
                                Value::Number(n) => n,
                                _ => panic!("expected numeric var"),
                            }
                        });
                    }

                    /* evaluate “then”-branch */
                    for c in data.children.iter().skip(1).take(last_true - 1) {
                        self.const_visit(c)?;
                    }

                    /* record “then” result and restore backup */
                    {
                        let lvl = self.nested_if_lvl.get() - 1;
                        let mut s1 = self.var_store1.borrow_mut();
                        let mut vars = self.base.variables.borrow_mut();

                        let store0 = &self.var_store0.borrow()[lvl];
                        let store1 = &mut s1[lvl];

                        data.affected_vars.iter().for_each(|&idx| {
                            /* record */
                            store1[idx] = match vars[idx] {
                                Value::Number(n) => n,
                                _ => panic!("expected numeric var"),
                            };
                            /* restore */
                            vars[idx] = Value::Number(store0[idx]);
                        });
                    }

                    /* evaluate “else”-branch (if any) */
                    if let Some(start) = data.first_else {
                        for c in data.children.iter().skip(start) {
                            self.const_visit(c)?;
                        }
                    }

                    /* final fuzzy blend */
                    {
                        let lvl = self.nested_if_lvl.get() - 1;
                        let store1 = &self.var_store1.borrow()[lvl];
                        let mut vars = self.base.variables.borrow_mut();

                        data.affected_vars.iter().for_each(|&idx| {
                            let v_true = store1[idx];
                            let v_false = match vars[idx] {
                                Value::Number(n) => n,
                                _ => panic!("expected numeric var"),
                            };
                            vars[idx] = Value::Number(
                                (dt * v_true + (NumericType::one() - dt) * v_false).into(),
                            );
                        });
                    }
                }

                /* leave this `if` */
                self.nested_if_lvl.set(self.nested_if_lvl.get() - 1);
                Ok(())
            }

            /* ─────────────── fall-through ─────────────── */
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

        let evaluator = FuzzyEvaluator::new().with_variables(indexer.get_variables_size());
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

        let evaluator = FuzzyEvaluator::new().with_variables(indexer.get_variables_size());

        evaluator.const_visit(&nodes).unwrap();

        assert_eq!(
            evaluator.variables(),
            vec![Value::Number(NumericType::new(2.0))]
        );
    }

    #[test]
    fn test_fuzzy_case() {
        Tape::start_recording();

        let script1 = "x = 0; y=0; if x > 0 { y = 1; } else { y = 0; }".to_string();
        let tokens = Lexer::new(script1).tokenize().unwrap();
        let mut script1_nodes = Parser::new(tokens).parse().unwrap();

        let script2 = "x = 0; y = fif(x,1,0,1);".to_string();
        let tokens2 = Lexer::new(script2).tokenize().unwrap();
        let mut script2_nodes = Parser::new(tokens2).parse().unwrap();

        let indexer = VarIndexer::new();
        indexer.visit(&mut script1_nodes).unwrap();

        let if_processor = IfProcessor::new();
        if_processor.visit(&mut script1_nodes).unwrap();
        let doman_processor = DomainProcessor::new(indexer.get_variables_size());
        doman_processor.visit(&mut script1_nodes).unwrap();

        let fuzzy_evaluator = FuzzyEvaluator::new()
            .with_eps(1.0)
            .with_variables(indexer.get_variables_size());

        fuzzy_evaluator.const_visit(&script1_nodes).unwrap();

        let eval_vars = fuzzy_evaluator.variables();
        match eval_vars.get(1).unwrap() {
            Value::Number(n) => {
                n.backward().unwrap();
            }
            _ => panic!("Expected y to be a number"),
        }

        let result = match eval_vars.get(0).unwrap() {
            Value::Number(n) => n.adjoint().unwrap(),
            _ => panic!("Expected x to be a number"),
        };

        indexer.clear();
        indexer.visit(&mut script2_nodes).unwrap();
        let evaluator = SingleScenarioEvaluator::new().with_variables(indexer.get_variables_size());
        evaluator.const_visit(&script2_nodes).unwrap();

        let eval_vars2 = evaluator.variables();
        match eval_vars2.get(1).unwrap() {
            Value::Number(n) => {
                n.backward().unwrap();
            }
            _ => panic!("Expected x to be a number"),
        }
        let result2 = match eval_vars2.get(0).unwrap() {
            Value::Number(n) => n.adjoint().unwrap(),
            _ => panic!("Expected fif result to be a number"),
        };
        assert!((result - result2).abs() < 1e-6, "Results do not match");
        Tape::stop_recording();
    }
}
