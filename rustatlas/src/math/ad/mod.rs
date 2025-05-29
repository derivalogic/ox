use std::cell::RefCell;
use std::cmp::Ordering;
use std::ops::{Add, Div, Mul, Neg, Sub};

use crate::utils::num::Real;

const ID_NONE: usize = usize::MAX;

thread_local! {
    static TAPE: RefCell<Vec<Node>> = RefCell::new(Vec::with_capacity(128));
}

#[derive(Clone)]
enum Op {
    /* leaf */ Input,
    /* binary */ Add,
    Sub,
    Mul,
    Div,
    /* rhs const */ AddConst(f64),
    MulConst(f64),
    /* lhs const */ ConstSub(f64),
    ConstDiv(f64),
    /* unary  */ Neg,
    Ln,
    Exp,
    Sin,
    Cos,
    Sqrt,
    Abs,
}

#[derive(Clone)]
struct Node {
    value: f64,
    op: Op,
    lhs: usize,
    rhs: usize,
}

#[inline]
fn push(n: Node) -> usize {
    TAPE.with(|t| {
        let mut t = t.borrow_mut();
        t.push(n);
        t.len() - 1
    })
}
#[inline]
fn node(id: usize) -> Node {
    TAPE.with(|t| t.borrow()[id].clone())
}
pub fn reset_tape() {
    TAPE.with(|t| t.borrow_mut().clear())
}

/* =======================================================================
 * 3.  Var handle
 * ==================================================================== */

#[derive(Clone, Copy)]
pub struct Var {
    id: usize,
    value: f64,
}

impl Var {
    #[inline]
    pub fn new(v: f64) -> Self {
        let id = push(Node {
            value: v,
            op: Op::Input,
            lhs: ID_NONE,
            rhs: ID_NONE,
        });
        Var { id, value: v }
    }
    #[inline]
    pub fn id(self) -> usize {
        self.id
    }
    #[inline]
    pub fn value(self) -> f64 {
        self.value
    }

    #[inline(always)]
    fn unary(self, op: Op, f: impl FnOnce(f64) -> f64) -> Self {
        let val = f(self.value);
        Var {
            id: push(Node {
                value: val,
                op,
                lhs: self.id,
                rhs: ID_NONE,
            }),
            value: val,
        }
    }
    #[inline(always)]
    fn binary(self, rhs: Self, op: Op, f: impl FnOnce(f64, f64) -> f64) -> Self {
        let val = f(self.value, rhs.value);
        Var {
            id: push(Node {
                value: val,
                op,
                lhs: self.id,
                rhs: rhs.id,
            }),
            value: val,
        }
    }

    /* elementary */
    #[inline]
    pub fn ln(self) -> Self {
        self.unary(Op::Ln, f64::ln)
    }
    #[inline]
    pub fn exp(self) -> Self {
        self.unary(Op::Exp, f64::exp)
    }
    #[inline]
    pub fn sin(self) -> Self {
        self.unary(Op::Sin, f64::sin)
    }
    #[inline]
    pub fn cos(self) -> Self {
        self.unary(Op::Cos, f64::cos)
    }
    #[inline]
    pub fn sqrt(self) -> Self {
        self.unary(Op::Sqrt, f64::sqrt)
    }
    #[inline]
    pub fn abs(self) -> Self {
        self.unary(Op::Abs, f64::abs)
    }
    pub fn powf(self, rhs: Self) -> Self {
        (self.ln() * rhs).exp()
    }
}

/* =======================================================================
 * 4.  Arithmetic impls – with folding
 * ==================================================================== */

impl Add for Var {
    type Output = Self;
    #[inline]
    fn add(self, rhs: Self) -> Self {
        self.binary(rhs, Op::Add, |a, b| a + b)
    }
}
impl Add<f64> for Var {
    type Output = Self;
    #[inline]
    fn add(self, c: f64) -> Self {
        if c == 0.0 {
            return self;
        }
        let v = self.value + c;
        Var {
            id: push(Node {
                value: v,
                op: Op::AddConst(c),
                lhs: self.id,
                rhs: ID_NONE,
            }),
            value: v,
        }
    }
}
impl Mul for Var {
    type Output = Self;
    #[inline]
    fn mul(self, rhs: Self) -> Self {
        self.binary(rhs, Op::Mul, |a, b| a * b)
    }
}
impl Mul<f64> for Var {
    type Output = Self;
    #[inline]
    fn mul(self, k: f64) -> Self {
        if k == 1.0 {
            return self;
        }
        if k == 0.0 {
            return Var::new(0.0);
        }
        let v = self.value * k;
        Var {
            id: push(Node {
                value: v,
                op: Op::MulConst(k),
                lhs: self.id,
                rhs: ID_NONE,
            }),
            value: v,
        }
    }
}
impl Sub for Var {
    type Output = Self;
    #[inline]
    fn sub(self, rhs: Self) -> Self {
        self.binary(rhs, Op::Sub, |a, b| a - b)
    }
}
impl Sub<f64> for Var {
    type Output = Self;
    #[inline]
    fn sub(self, c: f64) -> Self {
        self + (-c)
    }
} // x-c
impl Div for Var {
    type Output = Self;
    #[inline]
    fn div(self, rhs: Self) -> Self {
        self.binary(rhs, Op::Div, |a, b| a / b)
    }
}
impl Div<f64> for Var {
    type Output = Self;
    #[inline]
    fn div(self, c: f64) -> Self {
        self * (1.0 / c)
    }
} // x/c
impl Neg for Var {
    type Output = Self;
    fn neg(self) -> Self {
        let v = -self.value;
        Var {
            id: push(Node {
                value: v,
                op: Op::Neg,
                lhs: self.id,
                rhs: ID_NONE,
            }),
            value: v,
        }
    }
}

/* constant-on-the-left impls (local type `Var` ⇒ OK with orphan rule) */
impl Add<Var> for f64 {
    type Output = Var;
    #[inline]
    fn add(self, r: Var) -> Var {
        r + self
    }
}
impl Sub<Var> for f64 {
    type Output = Var;
    #[inline]
    fn sub(self, r: Var) -> Var {
        let v = self - r.value;
        Var {
            id: push(Node {
                value: v,
                op: Op::ConstSub(self),
                lhs: r.id,
                rhs: ID_NONE,
            }),
            value: v,
        }
    }
}
impl Mul<Var> for f64 {
    type Output = Var;
    #[inline]
    fn mul(self, r: Var) -> Var {
        r * self
    }
}
impl Div<Var> for f64 {
    type Output = Var;
    #[inline]
    fn div(self, r: Var) -> Var {
        let v = self / r.value;
        Var {
            id: push(Node {
                value: v,
                op: Op::ConstDiv(self),
                lhs: r.id,
                rhs: ID_NONE,
            }),
            value: v,
        }
    }
}

/* comparisons */
impl PartialEq for Var {
    fn eq(&self, o: &Self) -> bool {
        self.value.eq(&o.value)
    }
}
impl PartialOrd for Var {
    fn partial_cmp(&self, o: &Self) -> Option<Ordering> {
        self.value.partial_cmp(&o.value)
    }
}
/* conversions */
impl From<f64> for Var {
    fn from(v: f64) -> Self {
        Var::new(v)
    }
}
impl From<f32> for Var {
    fn from(v: f32) -> Self {
        Var::new(v as f64)
    }
}
impl From<i32> for Var {
    fn from(v: i32) -> Self {
        Var::new(v as f64)
    }
}

/* =======================================================================
 * 5.  Gradient (reverse sweep) – unchanged apart from new const-ops
 * ==================================================================== */

pub fn backward(result: &Var) -> Vec<f64> {
    TAPE.with(|cell| {
        let tape = cell.borrow();
        let mut g = vec![0.0; tape.len()];
        g[result.id] = 1.0;
        for i in (0..=result.id).rev() {
            match &tape[i].op {
                Op::Input => {}
                Op::Add => {
                    g[tape[i].lhs] += g[i];
                    g[tape[i].rhs] += g[i];
                }
                Op::Sub => {
                    g[tape[i].lhs] += g[i];
                    g[tape[i].rhs] -= g[i];
                }
                Op::Mul => {
                    let (lv, rv) = (tape[tape[i].lhs].value, tape[tape[i].rhs].value);
                    g[tape[i].lhs] += g[i] * rv;
                    g[tape[i].rhs] += g[i] * lv;
                }
                Op::Div => {
                    let (lv, rv) = (tape[tape[i].lhs].value, tape[tape[i].rhs].value);
                    g[tape[i].lhs] += g[i] / rv;
                    g[tape[i].rhs] -= g[i] * lv / (rv * rv);
                }
                Op::AddConst(_) => {
                    g[tape[i].lhs] += g[i];
                }
                Op::MulConst(k) => {
                    g[tape[i].lhs] += g[i] * k;
                }
                Op::ConstSub(_) => {
                    g[tape[i].lhs] -= g[i];
                }
                Op::ConstDiv(c) => {
                    let xv = tape[tape[i].lhs].value;
                    g[tape[i].lhs] -= g[i] * c / (xv * xv);
                }
                Op::Neg => {
                    g[tape[i].lhs] -= g[i];
                }
                Op::Ln => {
                    let lv = tape[tape[i].lhs].value;
                    g[tape[i].lhs] += g[i] / lv;
                }
                Op::Exp => {
                    let v = tape[i].value;
                    g[tape[i].lhs] += g[i] * v;
                }
                Op::Sin => {
                    let lv = tape[tape[i].lhs].value;
                    g[tape[i].lhs] += g[i] * lv.cos();
                }
                Op::Cos => {
                    let lv = tape[tape[i].lhs].value;
                    g[tape[i].lhs] -= g[i] * lv.sin();
                }
                Op::Sqrt => {
                    let lv = tape[tape[i].lhs].value;
                    g[tape[i].lhs] += g[i] * 0.5 / lv.sqrt();
                }
                Op::Abs => {
                    let lv = tape[tape[i].lhs].value;
                    let s = if lv > 0.0 {
                        1.0
                    } else if lv < 0.0 {
                        -1.0
                    } else {
                        0.0
                    };
                    g[tape[i].lhs] += g[i] * s;
                }
            }
        }
        g
    })
}

/* =======================================================================
 * 6.  Real impl for `Var`
 * ==================================================================== */

impl Real for Var {
    #[inline]
    fn ln(self) -> Self {
        self.ln()
    }
    #[inline]
    fn exp(self) -> Self {
        self.exp()
    }
    #[inline]
    fn powf(self, rhs: Self) -> Self {
        self.powf(rhs)
    }
    #[inline]
    fn sqrt(self) -> Self {
        self.sqrt()
    }
    #[inline]
    fn sin(self) -> Self {
        self.sin()
    }
    #[inline]
    fn cos(self) -> Self {
        self.cos()
    }
    #[inline]
    fn abs(self) -> Self {
        self.abs()
    }
    #[inline]
    fn min(self, other: Self) -> Self {
        if self < other {
            self
        } else {
            other
        }
    }
    #[inline]
    fn max(self, other: Self) -> Self {
        if self > other {
            self
        } else {
            other
        }
    }
}

/* =======================================================================
 * 7.  Tests – demonstrate both styles
 * ==================================================================== */

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn option_a_constant_on_right() {
        reset_tape();
        fn payoff<T: Real>(x: T) -> T {
            /* constants on RHS ⇒ compiles for every T: Real */
            x * 2.0 + 5.0 - x / 4.0
        }
        let v = Var::new(3.0);
        let y = payoff(v);
        let g = backward(&y);
        let expected = 2.0 - 1.0 / 4.0;
        assert!((g[v.id()] - expected).abs() < 1e-12);
    }

    #[test]
    fn option_b_helpers_constant_left() {
        reset_tape();
        fn payoff<T: Real>(x: T) -> T {
            Real::sub_from_const(10.0, x)   // 10 - x
              + Real::mul_to_const(3.0, x) // 3 * x
        }
        let v = Var::new(4.0);
        let y = payoff(v);
        let g = backward(&y);
        assert!((g[v.id()] - (-1.0 + 3.0)).abs() < 1e-12);
    }
}
