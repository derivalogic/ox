use std::cell::RefCell;
use std::cmp::Ordering;
use std::fmt::{Debug, Display};
use std::ops::{Add, Div, Mul, Neg, Sub};

use crate::utils::num::Real;

const ID_NONE: usize = usize::MAX;

thread_local! {
    static TAPE: RefCell<Vec<Node>> = RefCell::new(Vec::with_capacity(128));
    /// Mark position used by [`mark_tape`] and [`rewind_to_mark`]
    static MARK: RefCell<usize> = RefCell::new(0);
}

/// Node stored on the tape.
///
/// `n_args` indicates how many arguments this operation has (0, 1 or 2).
/// For unary operations `rhs` and `der_rhs` are unused.
#[derive(Clone)]
struct Node {
    value: f64,
    lhs: usize,
    rhs: usize,
    der_lhs: f64,
    der_rhs: f64,
    n_args: u8,
}

/// Tape segment recorded on a worker thread.
pub struct ThreadTape {
    /// Recorded nodes in execution order
    nodes: Vec<Node>,
}

#[inline]
fn push(n: Node) -> usize {
    TAPE.with(|t| {
        let mut t = t.borrow_mut();
        t.push(n);
        t.len() - 1
    })
}
// #[inline]
// fn node(id: usize) -> Node {
//     TAPE.with(|t| t.borrow()[id].clone())
// }
pub fn reset_tape() {
    TAPE.with(|t| t.borrow_mut().clear())
}

/// Remember the current end of the tape for later rewinding.
pub fn mark_tape() {
    TAPE.with(|t| MARK.with(|m| *m.borrow_mut() = t.borrow().len()))
}

/// Truncate the tape back to the last mark.
pub fn rewind_to_mark() {
    TAPE.with(|t| MARK.with(|m| t.borrow_mut().truncate(*m.borrow())))
}

/// Number of nodes currently stored on the tape.
pub fn tape_len() -> usize {
    TAPE.with(|t| t.borrow().len())
}

/// Extract and clear the current thread's tape, returning the captured segment.
pub fn take_thread_tape() -> ThreadTape {
    TAPE.with(|t| ThreadTape {
        nodes: std::mem::take(&mut *t.borrow_mut()),
    })
}

/// Append a thread tape onto the main tape, shifting node indices.
///
/// Returns the index offset that was applied to the merged nodes.
pub fn merge_thread_tape(mut tape: ThreadTape) -> usize {
    TAPE.with(|t| {
        let mut main = t.borrow_mut();
        let offset = main.len();
        for node in &mut tape.nodes {
            if node.lhs != ID_NONE {
                node.lhs += offset;
            }
            if node.rhs != ID_NONE {
                node.rhs += offset;
            }
        }
        main.extend(tape.nodes);
        offset
    })
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
            lhs: ID_NONE,
            rhs: ID_NONE,
            der_lhs: 0.0,
            der_rhs: 0.0,
            n_args: 0,
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

    /// Return a copy of the variable whose identifier is increased by `offset`.
    #[inline]
    pub fn shifted(self, offset: usize) -> Self {
        Var {
            id: self.id + offset,
            value: self.value,
        }
    }

    #[inline(always)]
    fn unary(
        self,
        f: impl FnOnce(f64) -> f64,
        df: impl FnOnce(f64, f64) -> f64,
    ) -> Self {
        let val = f(self.value);
        let der = df(self.value, val);
        Var {
            id: push(Node {
                value: val,
                lhs: self.id,
                rhs: ID_NONE,
                der_lhs: der,
                der_rhs: 0.0,
                n_args: 1,
            }),
            value: val,
        }
    }
    #[inline(always)]
    fn binary(
        self,
        rhs: Self,
        der_lhs: f64,
        der_rhs: f64,
        f: impl FnOnce(f64, f64) -> f64,
    ) -> Self {
        let val = f(self.value, rhs.value);
        Var {
            id: push(Node {
                value: val,
                lhs: self.id,
                rhs: rhs.id,
                der_lhs,
                der_rhs,
                n_args: 2,
            }),
            value: val,
        }
    }

    /* elementary */
    #[inline]
    pub fn ln(self) -> Self {
        self.unary(f64::ln, |x, _| 1.0 / x)
    }
    #[inline]
    pub fn exp(self) -> Self {
        self.unary(f64::exp, |_, v| v)
    }
    #[inline]
    pub fn sin(self) -> Self {
        self.unary(f64::sin, |x, _| x.cos())
    }
    #[inline]
    pub fn cos(self) -> Self {
        self.unary(f64::cos, |x, _| -x.sin())
    }
    #[inline]
    pub fn sqrt(self) -> Self {
        self.unary(f64::sqrt, |_, v| 0.5 / v)
    }
    #[inline]
    pub fn abs(self) -> Self {
        self.unary(f64::abs, |x, _| if x > 0.0 { 1.0 } else if x < 0.0 { -1.0 } else { 0.0 })
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
        self.binary(rhs, 1.0, 1.0, |a, b| a + b)
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
                lhs: self.id,
                rhs: ID_NONE,
                der_lhs: 1.0,
                der_rhs: 0.0,
                n_args: 1,
            }),
            value: v,
        }
    }
}
impl Mul for Var {
    type Output = Self;
    #[inline]
    fn mul(self, rhs: Self) -> Self {
        self.binary(rhs, rhs.value, self.value, |a, b| a * b)
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
                lhs: self.id,
                rhs: ID_NONE,
                der_lhs: k,
                der_rhs: 0.0,
                n_args: 1,
            }),
            value: v,
        }
    }
}
impl Sub for Var {
    type Output = Self;
    #[inline]
    fn sub(self, rhs: Self) -> Self {
        self.binary(rhs, 1.0, -1.0, |a, b| a - b)
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
        let inv = 1.0 / rhs.value;
        self.binary(rhs, inv, -self.value * inv * inv, |a, b| a / b)
    }
}
impl Div<f64> for Var {
    type Output = Self;
    #[inline]
    fn div(self, c: f64) -> Self {
        if c == 1.0 {
            self
        } else {
            let v = self.value / c;
            Var {
                id: push(Node {
                    value: v,
                    lhs: self.id,
                    rhs: ID_NONE,
                    der_lhs: 1.0 / c,
                    der_rhs: 0.0,
                    n_args: 1,
                }),
                value: v,
            }
        }
    }
} // x/c
impl Neg for Var {
    type Output = Self;
    fn neg(self) -> Self {
        let v = -self.value;
        Var {
            id: push(Node {
                value: v,
                lhs: self.id,
                rhs: ID_NONE,
                der_lhs: -1.0,
                der_rhs: 0.0,
                n_args: 1,
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
                lhs: r.id,
                rhs: ID_NONE,
                der_lhs: -1.0,
                der_rhs: 0.0,
                n_args: 1,
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
                lhs: r.id,
                rhs: ID_NONE,
                der_lhs: -self / (r.value * r.value),
                der_rhs: 0.0,
                n_args: 1,
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

impl From<Var> for f64 {
    fn from(v: Var) -> Self {
        v.value()
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
            let node = &tape[i];
            match node.n_args {
                0 => {}
                1 => {
                    g[node.lhs] += g[i] * node.der_lhs;
                }
                2 => {
                    g[node.lhs] += g[i] * node.der_lhs;
                    g[node.rhs] += g[i] * node.der_rhs;
                }
                _ => unreachable!(),
            }
        }
        g
    })
}

/* =======================================================================
 * 6.  Real impl for `Var`
 * ==================================================================== */

impl Debug for Var {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Var(id={}, value={})", self.id, self.value)
    }
}

impl Display for Var {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.value)
    }
}

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

    #[test]
    fn merge_thread_tape_parallel() {
        use rayon::prelude::*;

        let inputs = vec![1.0, 2.0];

        // run two parallel computations each on its own tape
        let parts: Vec<(Var, Var, ThreadTape)> = inputs
            .into_par_iter()
            .map(|x| {
                reset_tape();
                let xv = Var::new(x);
                let y = xv * xv;
                let tape = take_thread_tape();
                (xv, y, tape)
            })
            .collect();

        reset_tape();
        let mut total = Var::new(0.0);
        let mut xs = Vec::new();

        for (x, y, tape) in parts {
            let offset = merge_thread_tape(tape);
            let x = x.shifted(offset);
            let y = y.shifted(offset);
            xs.push(x);
            total = total + y;
        }

        let g = backward(&total);
        assert!((g[xs[0].id()] - 2.0 * 1.0).abs() < 1e-12);
        assert!((g[xs[1].id()] - 2.0 * 2.0).abs() < 1e-12);
    }
}
