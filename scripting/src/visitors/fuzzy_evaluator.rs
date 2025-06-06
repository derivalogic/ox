use rustatlas::prelude::FloatExt;
use rustatlas::prelude::{NumericType, ToNumeric, max, min};
use crate::prelude::*;
use crate::utils::math::f_if;

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


    fn eval_dt(&self, node: &ExprTree) -> NumericType {
        match node.as_ref() {
            Node::Superior(children) => {
                let left = self.eval(&children[0]);
                let right = self.eval(&children[1]);
                f_if((left - right).into(), NumericType::one(), NumericType::zero(), self.eps)
            }
            Node::Inferior(children) => {
                let left = self.eval(&children[0]);
                let right = self.eval(&children[1]);
                f_if((right - left).into(), NumericType::one(), NumericType::zero(), self.eps)
            }
            Node::SuperiorOrEqual(children) => {
                let left = self.eval(&children[0]);
                let right = self.eval(&children[1]);
                f_if((left - right).into(), NumericType::one(), NumericType::zero(), self.eps)
            }
            Node::InferiorOrEqual(children) => {
                let left = self.eval(&children[0]);
                let right = self.eval(&children[1]);
                f_if((right - left).into(), NumericType::one(), NumericType::zero(), self.eps)
            }
            Node::Equal(children) => {
                let left = self.eval(&children[0]);
                let right = self.eval(&children[1]);
                let diff = left - right;
                f_if((-diff.abs()).into(), NumericType::one(), NumericType::zero(), self.eps)
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
            Node::FIf(children) => {
                let x = self.eval(&children[0]);
                let a = self.eval(&children[1]);
                let b = self.eval(&children[2]);
                let eps = self.eval(&children[3]);
                f_if(x, a, b, eps)
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

