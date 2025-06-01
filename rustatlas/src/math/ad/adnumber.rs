//! aad.rs  ―  Expression-template reverse-mode AD in pure Rust
//! Public API:  ADNumber  +  free fns  exp, log, sqrt, fabs,
//! normal_dens, normal_cdf, pow, max, min  +  flatten/propagation helpers.

#![allow(clippy::needless_return)]

use crate::prelude::*;
use std::ops::*;
/* ═════════════════════════  REVERSE-MODE TAPE  ═══════════════════════ */

/* ═══════════════════════  EXPRESSION TRAIT  ═════════════════════════ */

pub trait Expr: Clone {
    fn value(&self) -> f64;
    fn push_adj(&self, parent: &mut Node, adj: f64);
}

/* ═══════════════════════  LEAF: ADNumber  ═════════════════════════════ */

#[derive(Clone)]
pub struct ADNumber {
    val: f64,
    idx: usize, // position on the tape
}

impl ADNumber {
    pub fn new(v: f64) -> Self {
        let idx = TAPE.with(|t| t.borrow_mut().new_leaf());
        Self { val: v, idx }
    }

    /* ---- accessors ---- */
    #[inline]
    pub fn value(&self) -> f64 {
        self.val
    }
    #[inline]
    pub fn adjoint(&self) -> f64 {
        TAPE.with(|t| t.borrow().nodes[self.idx].adj)
    }

    /* ---- tape helpers ---- */
    pub fn reset_adjoints() {
        TAPE.with(|t| {
            for n in &mut t.borrow_mut().nodes {
                n.adj = 0.0;
            }
        });
    }

    pub fn put_on_tape(&mut self) {
        self.idx = TAPE.with(|t| t.borrow_mut().new_leaf());
    }

    pub fn propagate_to_start(&self) {
        TAPE.with(|t| t.borrow_mut().nodes[self.idx].adj = 1.0);
        propagate_range(self.idx, 0);
    }

    pub fn propagate_to_mark(&self) {
        let stop = TAPE.with(|t| t.borrow().mark);
        TAPE.with(|t| t.borrow_mut().nodes[self.idx].adj = 1.0);
        propagate_range(self.idx, stop);
    }

    pub fn propagate_mark_to_start() {
        let (from, to) = TAPE.with(|t| {
            let t = t.borrow();
            (t.mark.saturating_sub(1), 0usize)
        });
        propagate_range(from, to);
    }
}

impl Expr for ADNumber {
    fn value(&self) -> f64 {
        self.val
    }

    fn push_adj(&self, parent: &mut Node, adj: f64) {
        parent.childs.push(self.idx);
        parent.derivs.push(adj);
    }
}

/* ═══════════════════════  CONSTANT LEAF  ════════════════════════════ */

#[derive(Clone, Copy)]
pub struct Const(pub f64);

impl From<f64> for Const {
    fn from(v: f64) -> Self {
        Const(v)
    }
}

impl Expr for Const {
    fn value(&self) -> f64 {
        self.0
    }
    fn push_adj(&self, _parent: &mut Node, _adj: f64) {}
}

/* ═════════════════════  OPERATOR “TYPE CLASSES”  ════════════════════ */

pub trait BinOp {
    fn eval(l: f64, r: f64) -> f64;
    fn d_left(l: f64, r: f64) -> f64;
    fn d_right(l: f64, r: f64) -> f64;
}

pub struct AddOp;
impl BinOp for AddOp {
    fn eval(l: f64, r: f64) -> f64 {
        l + r
    }
    fn d_left(_l: f64, _r: f64) -> f64 {
        1.0
    }
    fn d_right(_l: f64, _r: f64) -> f64 {
        1.0
    }
}

pub struct SubOp;
impl BinOp for SubOp {
    fn eval(l: f64, r: f64) -> f64 {
        l - r
    }
    fn d_left(_l: f64, _r: f64) -> f64 {
        1.0
    }
    fn d_right(_l: f64, _r: f64) -> f64 {
        -1.0
    }
}

pub struct MulOp;
impl BinOp for MulOp {
    fn eval(l: f64, r: f64) -> f64 {
        l * r
    }
    fn d_left(_l: f64, r: f64) -> f64 {
        r
    }
    fn d_right(l: f64, _r: f64) -> f64 {
        l
    }
}

pub struct DivOp;
impl BinOp for DivOp {
    fn eval(l: f64, r: f64) -> f64 {
        l / r
    }
    fn d_left(_l: f64, r: f64) -> f64 {
        1.0 / r
    }
    fn d_right(l: f64, r: f64) -> f64 {
        -l / (r * r)
    }
}

pub struct PowOp;
impl BinOp for PowOp {
    fn eval(l: f64, r: f64) -> f64 {
        l.powf(r)
    }
    fn d_left(l: f64, r: f64) -> f64 {
        r * l.powf(r - 1.0)
    }
    fn d_right(l: f64, r: f64) -> f64 {
        l.powf(r) * l.ln()
    }
}

pub struct MaxOp;
impl BinOp for MaxOp {
    fn eval(l: f64, r: f64) -> f64 {
        l.max(r)
    }
    fn d_left(l: f64, r: f64) -> f64 {
        if l > r {
            1.0
        } else {
            0.0
        }
    }
    fn d_right(l: f64, r: f64) -> f64 {
        if r > l {
            1.0
        } else {
            0.0
        }
    }
}

pub struct MinOp;

impl BinOp for MinOp {
    fn eval(l: f64, r: f64) -> f64 {
        l.min(r)
    }
    fn d_left(l: f64, r: f64) -> f64 {
        if l < r {
            1.0
        } else {
            0.0
        }
    }
    fn d_right(l: f64, r: f64) -> f64 {
        if r < l {
            1.0
        } else {
            0.0
        }
    }
}

/* ═══════════════════════  CLONE OPERATORS  ══════════════════════════ */

impl Clone for AddOp {
    fn clone(&self) -> Self {
        AddOp
    }
}

impl Clone for SubOp {
    fn clone(&self) -> Self {
        SubOp
    }
}

impl Clone for MulOp {
    fn clone(&self) -> Self {
        MulOp
    }
}

impl Clone for DivOp {
    fn clone(&self) -> Self {
        DivOp
    }
}

impl Clone for PowOp {
    fn clone(&self) -> Self {
        PowOp
    }
}

impl Clone for MinOp {
    fn clone(&self) -> Self {
        MinOp
    }
}

/* ════════════════════  BINARY EXPRESSION NODE  ══════════════════════ */

#[derive(Clone)]
pub struct BinExpr<L, R, O> {
    l: L,
    r: R,
    val: f64,
    _ph: std::marker::PhantomData<O>,
}

impl<L: Expr, R: Expr, O: BinOp> BinExpr<L, R, O> {
    fn new(l: L, r: R) -> Self {
        let val = O::eval(l.value(), r.value());
        Self {
            l,
            r,
            val,
            _ph: std::marker::PhantomData,
        }
    }
}

impl<L: Expr, R: Expr, O: BinOp + Clone> Expr for BinExpr<L, R, O> {
    fn value(&self) -> f64 {
        self.val
    }

    fn push_adj(&self, parent: &mut Node, adj: f64) {
        self.l
            .push_adj(parent, adj * O::d_left(self.l.value(), self.r.value()));
        self.r
            .push_adj(parent, adj * O::d_right(self.l.value(), self.r.value()));
    }
}

/* ═══════════════════════  UNARY OPERATORS  ═════════════════════════ */

pub trait UnOp {
    fn eval(x: f64) -> f64;
    fn deriv(x: f64, v: f64) -> f64;
}

macro_rules! un_op {
    ($name:ident, $eval:expr, $d:expr) => {
        pub struct $name;
        impl UnOp for $name {
            fn eval(x: f64) -> f64 {
                $eval(x)
            }
            fn deriv(x: f64, v: f64) -> f64 {
                $d(x, v)
            }
        }
    };
}

un_op!(ExpOp, f64::exp, |_x, v| v);
un_op!(LogOp, f64::ln, |x, _v| 1.0 / x);
un_op!(SqrtOp, f64::sqrt, |_x, v| 0.5 / v);
un_op!(FabsOp, f64::abs, |x, _v| if x >= 0.0 { 1.0 } else { -1.0 });
un_op!(SinOp, f64::sin, |x, v| v * f64::cos(x));
un_op!(CosOp, f64::cos, |x, v| -v * f64::sin(x));

#[derive(Clone)]
pub struct UnExpr<A, O> {
    a: A,
    val: f64,
    _ph: std::marker::PhantomData<O>,
}

impl<A: Expr, O: UnOp> UnExpr<A, O> {
    fn new(a: A) -> Self {
        let val = O::eval(a.value());
        Self {
            a,
            val,
            _ph: std::marker::PhantomData,
        }
    }
}

impl<A: Expr, O: UnOp + Clone> Expr for UnExpr<A, O> {
    fn value(&self) -> f64 {
        self.val
    }

    fn push_adj(&self, parent: &mut Node, adj: f64) {
        self.a
            .push_adj(parent, adj * O::deriv(self.a.value(), self.val));
    }
}

/* ══════════════  OPERATOR OVERLOADS (coherence-safe)  ═══════════════ */

macro_rules! impl_bin_ops_local {
    ($Self:ty) => {
        /* Add */
        impl<Rhs> Add<Rhs> for $Self
        where
            Rhs: Expr + Clone,
            Self: Expr + Clone,
        {
            type Output = BinExpr<Self, Rhs, AddOp>;
            fn add(self, rhs: Rhs) -> Self::Output {
                BinExpr::new(self, rhs)
            }
        }
        impl Add<f64> for $Self
        where
            Self: Expr + Clone,
        {
            type Output = BinExpr<Self, Const, AddOp>;
            fn add(self, rhs: f64) -> Self::Output {
                BinExpr::new(self, Const(rhs))
            }
        }
        /* Sub */
        impl<Rhs> Sub<Rhs> for $Self
        where
            Rhs: Expr + Clone,
            Self: Expr + Clone,
        {
            type Output = BinExpr<Self, Rhs, SubOp>;
            fn sub(self, rhs: Rhs) -> Self::Output {
                BinExpr::new(self, rhs)
            }
        }
        impl Sub<f64> for $Self
        where
            Self: Expr + Clone,
        {
            type Output = BinExpr<Self, Const, SubOp>;
            fn sub(self, rhs: f64) -> Self::Output {
                BinExpr::new(self, Const(rhs))
            }
        }
        /* Mul */
        impl<Rhs> Mul<Rhs> for $Self
        where
            Rhs: Expr + Clone,
            Self: Expr + Clone,
        {
            type Output = BinExpr<Self, Rhs, MulOp>;
            fn mul(self, rhs: Rhs) -> Self::Output {
                BinExpr::new(self, rhs)
            }
        }
        impl Mul<f64> for $Self
        where
            Self: Expr + Clone,
        {
            type Output = BinExpr<Self, Const, MulOp>;
            fn mul(self, rhs: f64) -> Self::Output {
                BinExpr::new(self, Const(rhs))
            }
        }
        /* Div */
        impl<Rhs> Div<Rhs> for $Self
        where
            Rhs: Expr + Clone,
            Self: Expr + Clone,
        {
            type Output = BinExpr<Self, Rhs, DivOp>;
            fn div(self, rhs: Rhs) -> Self::Output {
                BinExpr::new(self, rhs)
            }
        }
        impl Div<f64> for $Self
        where
            Self: Expr + Clone,
        {
            type Output = BinExpr<Self, Const, DivOp>;
            fn div(self, rhs: f64) -> Self::Output {
                BinExpr::new(self, Const(rhs))
            }
        }
        /* Neg */
        impl Neg for $Self
        where
            Self: Expr + Clone,
        {
            type Output = BinExpr<Const, Self, SubOp>;
            fn neg(self) -> Self::Output {
                BinExpr::new(Const(0.0), self)
            }
        }
    };
}

/* apply the macro to every local expression kind */
impl_bin_ops_local!(ADNumber);
impl_bin_ops_local!(Const);

/* Manual implementations for generic types to avoid macro expansion issues */
impl<L, R, O, Rhs> Add<Rhs> for BinExpr<L, R, O>
where
    Rhs: Expr + Clone,
    Self: Expr + Clone,
{
    type Output = BinExpr<Self, Rhs, AddOp>;
    fn add(self, rhs: Rhs) -> Self::Output {
        BinExpr::new(self, rhs)
    }
}
impl<L, R, O> Add<f64> for BinExpr<L, R, O>
where
    Self: Expr + Clone,
{
    type Output = BinExpr<Self, Const, AddOp>;
    fn add(self, rhs: f64) -> Self::Output {
        BinExpr::new(self, Const(rhs))
    }
}
impl<L, R, O, Rhs> Sub<Rhs> for BinExpr<L, R, O>
where
    Rhs: Expr + Clone,
    Self: Expr + Clone,
{
    type Output = BinExpr<Self, Rhs, SubOp>;
    fn sub(self, rhs: Rhs) -> Self::Output {
        BinExpr::new(self, rhs)
    }
}
impl<L, R, O> Sub<f64> for BinExpr<L, R, O>
where
    Self: Expr + Clone,
{
    type Output = BinExpr<Self, Const, SubOp>;
    fn sub(self, rhs: f64) -> Self::Output {
        BinExpr::new(self, Const(rhs))
    }
}
impl<L, R, O, Rhs> Mul<Rhs> for BinExpr<L, R, O>
where
    Rhs: Expr + Clone,
    Self: Expr + Clone,
{
    type Output = BinExpr<Self, Rhs, MulOp>;
    fn mul(self, rhs: Rhs) -> Self::Output {
        BinExpr::new(self, rhs)
    }
}
impl<L, R, O> Mul<f64> for BinExpr<L, R, O>
where
    Self: Expr + Clone,
{
    type Output = BinExpr<Self, Const, MulOp>;
    fn mul(self, rhs: f64) -> Self::Output {
        BinExpr::new(self, Const(rhs))
    }
}
impl<L, R, O, Rhs> Div<Rhs> for BinExpr<L, R, O>
where
    Rhs: Expr + Clone,
    Self: Expr + Clone,
{
    type Output = BinExpr<Self, Rhs, DivOp>;
    fn div(self, rhs: Rhs) -> Self::Output {
        BinExpr::new(self, rhs)
    }
}
impl<L, R, O> Div<f64> for BinExpr<L, R, O>
where
    Self: Expr + Clone,
{
    type Output = BinExpr<Self, Const, DivOp>;
    fn div(self, rhs: f64) -> Self::Output {
        BinExpr::new(self, Const(rhs))
    }
}
impl<L, R, O> Neg for BinExpr<L, R, O>
where
    Self: Expr + Clone,
{
    type Output = BinExpr<Const, Self, SubOp>;
    fn neg(self) -> Self::Output {
        BinExpr::new(Const(0.0), self)
    }
}

impl<A, O, Rhs> Add<Rhs> for UnExpr<A, O>
where
    Rhs: Expr + Clone,
    Self: Expr + Clone,
{
    type Output = BinExpr<Self, Rhs, AddOp>;
    fn add(self, rhs: Rhs) -> Self::Output {
        BinExpr::new(self, rhs)
    }
}
impl<A, O> Add<f64> for UnExpr<A, O>
where
    Self: Expr + Clone,
{
    type Output = BinExpr<Self, Const, AddOp>;
    fn add(self, rhs: f64) -> Self::Output {
        BinExpr::new(self, Const(rhs))
    }
}
impl<A, O, Rhs> Sub<Rhs> for UnExpr<A, O>
where
    Rhs: Expr + Clone,
    Self: Expr + Clone,
{
    type Output = BinExpr<Self, Rhs, SubOp>;
    fn sub(self, rhs: Rhs) -> Self::Output {
        BinExpr::new(self, rhs)
    }
}
impl<A, O> Sub<f64> for UnExpr<A, O>
where
    Self: Expr + Clone,
{
    type Output = BinExpr<Self, Const, SubOp>;
    fn sub(self, rhs: f64) -> Self::Output {
        BinExpr::new(self, Const(rhs))
    }
}
impl<A, O, Rhs> Mul<Rhs> for UnExpr<A, O>
where
    Rhs: Expr + Clone,
    Self: Expr + Clone,
{
    type Output = BinExpr<Self, Rhs, MulOp>;
    fn mul(self, rhs: Rhs) -> Self::Output {
        BinExpr::new(self, rhs)
    }
}
impl<A, O> Mul<f64> for UnExpr<A, O>
where
    Self: Expr + Clone,
{
    type Output = BinExpr<Self, Const, MulOp>;
    fn mul(self, rhs: f64) -> Self::Output {
        BinExpr::new(self, Const(rhs))
    }
}
impl<A, O, Rhs> Div<Rhs> for UnExpr<A, O>
where
    Rhs: Expr + Clone,
    Self: Expr + Clone,
{
    type Output = BinExpr<Self, Rhs, DivOp>;
    fn div(self, rhs: Rhs) -> Self::Output {
        BinExpr::new(self, rhs)
    }
}
impl<A, O> Div<f64> for UnExpr<A, O>
where
    Self: Expr + Clone,
{
    type Output = BinExpr<Self, Const, DivOp>;
    fn div(self, rhs: f64) -> Self::Output {
        BinExpr::new(self, Const(rhs))
    }
}
impl<A, O> Neg for UnExpr<A, O>
where
    Self: Expr + Clone,
{
    type Output = BinExpr<Const, Self, SubOp>;
    fn neg(self) -> Self::Output {
        BinExpr::new(Const(0.0), self)
    }
}

/* ----  assignment variants only for ADNumber  ------------------------- */

macro_rules! impl_assign {
    ($Trait:ident, $func:ident, $Op:ident, $sym:tt) => {
        impl<E> $Trait<E> for ADNumber
        where
            E: Expr + Clone,
        {
            fn $func(&mut self, rhs: E) {
                *self = flatten(&(self.clone() $sym rhs));
            }
        }
        impl $Trait<f64> for ADNumber {
            fn $func(&mut self, rhs: f64) {
                *self = flatten(&(self.clone() $sym Const(rhs)));
            }
        }
    };
}

impl_assign!(AddAssign, add_assign, AddOp, +);
impl_assign!(SubAssign, sub_assign, SubOp, -);
impl_assign!(MulAssign, mul_assign, MulOp, *);
impl_assign!(DivAssign, div_assign, DivOp, /);

/* ════════════════  PUBLIC MATH HELPERS (free fns)  ══════════════════ */

#[inline]
pub fn exp<A: Expr + Clone>(a: A) -> UnExpr<A, ExpOp> {
    UnExpr::new(a)
}
#[inline]
pub fn log<A: Expr + Clone>(a: A) -> UnExpr<A, LogOp> {
    UnExpr::new(a)
}
#[inline]
pub fn sqrt<A: Expr + Clone>(a: A) -> UnExpr<A, SqrtOp> {
    UnExpr::new(a)
}
#[inline]
pub fn fabs<A: Expr + Clone>(a: A) -> UnExpr<A, FabsOp> {
    UnExpr::new(a)
}
// #[inline]
// pub fn normal_dens<A: Expr + Clone>(a: A) -> UnExpr<A, NormDensOp> {
//     UnExpr::new(a)
// }
// #[inline]
// pub fn normal_cdf<A: Expr + Clone>(a: A) -> UnExpr<A, NormCdfOp> {
//     UnExpr::new(a)
// }

#[inline]
pub fn pow<L: Expr + Clone, R: Expr + Clone>(l: L, r: R) -> BinExpr<L, R, PowOp> {
    BinExpr::new(l, r)
}
#[inline]
pub fn max<L: Expr + Clone, R: Expr + Clone>(l: L, r: R) -> BinExpr<L, R, MaxOp> {
    BinExpr::new(l, r)
}
#[inline]
pub fn min<L: Expr + Clone, R: Expr + Clone>(l: L, r: R) -> BinExpr<L, R, MinOp> {
    BinExpr::new(l, r)
}

/* ════════════════  FLATTEN AN EXPRESSION TO A ADNumber  ═══════════════ */

/// “Flatten” an arbitrary expression into a concrete `ADNumber` node on the tape
fn flatten<E: Expr + Clone>(e: &E) -> ADNumber {
    let mut node = Node::default();
    e.push_adj(&mut node, 1.0);
    let idx = TAPE.with(|t| t.borrow_mut().record(node));
    ADNumber {
        val: e.value(),
        idx,
    }
}

impl<L, R, O> From<BinExpr<L, R, O>> for ADNumber
where
    L: Expr + Clone,
    R: Expr + Clone,
    O: BinOp + Clone,
{
    fn from(expr: BinExpr<L, R, O>) -> Self {
        // `flatten` does the real work
        flatten(&expr)
    }
}

// ── Unary expressions ────────────────────────────────────────────────
impl<A, O> From<UnExpr<A, O>> for ADNumber
where
    A: Expr + Clone,
    O: UnOp + Clone,
{
    fn from(expr: UnExpr<A, O>) -> Self {
        flatten(&expr)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::math::ad::tape::TAPE;

    #[test]
    fn test_flatten() {
        let a = ADNumber::new(3.0);
        let b = ADNumber::new(4.0);
        let expr = a + b;
        let result: ADNumber = expr.into();
        assert_eq!(result.value(), 7.0);
        assert_eq!(result.adjoint(), 0.0); // adjoint should be zero before propagation
    }

    #[test]
    fn test_propagation() {
        let a = ADNumber::new(3.0);
        let b = ADNumber::new(4.0);
        let expr = a + b;
        let result = flatten(&expr);
        result.propagate_to_start();
        assert_eq!(result.adjoint(), 1.0); // adjoint should be 1 after propagation
    }

    #[test]
    fn test_aritmetics() {
        let a = ADNumber::new(3.0);
        let b = ADNumber::new(4.0);
        let c = a * b;
        let result = flatten(&c);
        assert_eq!(result.value(), 12.0);
        result.propagate_to_start();
        assert_eq!(result.adjoint(), 1.0); // adjoint should be 1 after propagation
        let tape = TAPE.with(|t| t.borrow().nodes.clone());
        assert_eq!(tape.len(), 3); // should have 3 nodes: a, b, and c
        assert_eq!(tape[0].adj, 1.0); // a's adjoint
        assert_eq!(tape[1].adj, 1.0); // b's adjoint
        assert_eq!(tape[2].adj, 1.0); // c's adjoint
    }
}
