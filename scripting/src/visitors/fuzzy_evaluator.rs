use rustatlas::prelude::FloatExt;
use rustatlas::prelude::{NumericType, ToNumeric, max, min};
use crate::prelude::*;

/// Smooth conditional evaluation using fuzzy logic.
/// The smoothing parameter `eps` controls the transition width around zero.
pub struct FuzzyEvaluator {
    eps: NumericType,
}

impl FuzzyEvaluator {
    /// Create a new evaluator with the given smoothing parameter.
    pub fn new(eps: NumericType) -> Self {
        Self { eps }
    }

    /// Continuous, piecewise linear replacement of `if x > 0`.
    fn f_if(&self, x: NumericType, a: NumericType, b: NumericType) -> NumericType {
        let half = self.eps / NumericType::new(2.0);
        let t = min(max(x + half, NumericType::zero()), self.eps);
        (b + (a - b) * t / self.eps).into()
    }

    fn eval_dt(&self, node: &ExprTree) -> NumericType {
        match node.as_ref() {
            Node::Superior(children) => {
                let left = self.eval(&children[0]);
                let right = self.eval(&children[1]);
                self.f_if((left - right).into(), NumericType::one(), NumericType::zero())
            }
            Node::Inferior(children) => {
                let left = self.eval(&children[0]);
                let right = self.eval(&children[1]);
                self.f_if((right - left).into(), NumericType::one(), NumericType::zero())
            }
            Node::SuperiorOrEqual(children) => {
                let left = self.eval(&children[0]);
                let right = self.eval(&children[1]);
                self.f_if((left - right).into(), NumericType::one(), NumericType::zero())
            }
            Node::InferiorOrEqual(children) => {
                let left = self.eval(&children[0]);
                let right = self.eval(&children[1]);
                self.f_if((right - left).into(), NumericType::one(), NumericType::zero())
            }
            Node::Equal(children) => {
                let left = self.eval(&children[0]);
                let right = self.eval(&children[1]);
                let diff = left - right;
                self.f_if((-diff.abs()).into(), NumericType::one(), NumericType::zero())
            }
            Node::Not(children) => {
                let dt = self.eval_dt(&children[0]);
                (NumericType::one() - dt).into()
            }
            Node::And(children) => {
                let a = self.eval_dt(&children[0]);
                let b = self.eval_dt(&children[1]);
                (a * b).into()
            }
            Node::Or(children) => {
                let a = self.eval_dt(&children[0]);
                let b = self.eval_dt(&children[1]);
                (a + b - a * b).into()
            }
            Node::True => NumericType::one(),
            Node::False => NumericType::zero(),
            _ => NumericType::zero(),
        }
    }

    /// Evaluate an expression under fuzzy logic.
    pub fn eval(&self, node: &ExprTree) -> NumericType {
        match node.as_ref() {
            Node::Constant(v) => *v,
            Node::Add(children) => (self.eval(&children[0]) + self.eval(&children[1])).into(),
            Node::Subtract(children) => (self.eval(&children[0]) - self.eval(&children[1])).into(),
            Node::Multiply(children) => (self.eval(&children[0]) * self.eval(&children[1])).into(),
            Node::Divide(children) => (self.eval(&children[0]) / self.eval(&children[1])).into(),
            Node::Min(children) => {
                let a = self.eval(&children[0]);
                let b = self.eval(&children[1]);
                min(a, b).into()
            }
            Node::Max(children) => {
                let a = self.eval(&children[0]);
                let b = self.eval(&children[1]);
                max(a, b).into()
            }
            Node::If(children, first_else) => {
                let dt = self.eval_dt(&children[0]);
                let mut then_val = NumericType::zero();
                let mut else_val = NumericType::zero();
                let split = first_else.unwrap_or(children.len());
                if split > 1 {
                    then_val = self.eval(&children[1]);
                }
                if split < children.len() {
                    else_val = self.eval(&children[split]);
                }
                (dt * then_val + (NumericType::one() - dt) * else_val).into()
            }
            _ => NumericType::zero(),
        }
    }
}

