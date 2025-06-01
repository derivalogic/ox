//! aad.rs  ―  Expression-template reverse-mode AD in pure Rust
//! Public API:  ADNumber  +  helpers  exp, log, sqrt, fabs, sin, cos,
//! pow, max, min, abs  +  flatten / propagation helpers.
//
//! This file is standalone except for the tape machinery:
//! `Node`, `TAPE`, `propagate_range` live in the surrounding crate
//! exactly as in the original version.

use serde::{Deserialize, Serialize};

use crate::prelude::*;
use core::fmt;
use std::{cmp::Ordering, ops::*};

/* ───────────────────────── Feature aliases ───────────────────────── */

#[cfg(feature = "adnumber")]
pub type NumericType = ADNumber;
#[cfg(feature = "f64")]
pub type NumericType = f64;

/* ═══════════════════  Value ⇆ Wrapper convenience  ═════════════════ */

pub trait ToNumeric<T> {
    fn new(v: T) -> Self;
    fn value(&self) -> T;
    fn one() -> Self;
    fn zero() -> Self;
}

/* ---- primitive  ↔  f64 ------------------------------------------- */

impl ToNumeric<f64> for f64 {
    #[inline]
    fn new(v: f64) -> Self {
        v
    }
    #[inline]
    fn value(&self) -> f64 {
        *self
    }

    #[inline]
    fn one() -> Self {
        1.0
    }
    #[inline]
    fn zero() -> Self {
        0.0
    }
}

impl fmt::Debug for ADNumber {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ADNumber({})", self.val)
    }
}

impl fmt::Display for ADNumber {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ADNumber({})", self.val)
    }
}

/* ---- ADNumber  ↔  f64 -------------------------------------------- */

impl ToNumeric<f64> for ADNumber {
    #[inline]
    fn new(v: f64) -> Self {
        let idx = TAPE.with(|t| t.borrow_mut().new_leaf());
        Self { val: v, idx }
    }
    #[inline]
    fn value(&self) -> f64 {
        self.val
    }
    #[inline]
    fn one() -> Self {
        ADNumber::new(1.0)
    }
    #[inline]
    fn zero() -> Self {
        ADNumber::new(0.0)
    }
}

/* ═════════════════════  TRAIT FOR ALL NODES  ═══════════════════════ */

pub trait Expr: Clone {
    fn inner_value(&self) -> f64;
    fn push_adj(&self, parent: &mut TapeNode, adj: f64);
}

/* ════════════════════════  LEAF: ADNumber  ═════════════════════════ */

#[derive(Clone)]
pub struct ADNumber {
    val: f64,
    idx: usize, // position on the tape
}

/* ---- constructors & tape helpers --------------------------------- */

impl ADNumber {
    pub fn new(v: f64) -> Self {
        let idx = TAPE.with(|t| t.borrow_mut().new_leaf());
        Self { val: v, idx }
    }

    #[inline]
    pub fn value(&self) -> f64 {
        self.val
    }
    #[inline]
    pub fn adjoint(&self) -> f64 {
        TAPE.with(|t| t.borrow().nodes[self.idx].adj)
    }

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

/* ---- integrate with Expr ----------------------------------------- */

impl Expr for ADNumber {
    #[inline]
    fn inner_value(&self) -> f64 {
        self.val
    }
    fn push_adj(&self, parent: &mut TapeNode, adj: f64) {
        parent.childs.push(self.idx);
        parent.derivs.push(adj);
    }
}

/* ---- debug / serde / ordering ------------------------------------ */

impl PartialEq for ADNumber {
    fn eq(&self, o: &Self) -> bool {
        self.val == o.val
    }
}
impl PartialOrd for ADNumber {
    fn partial_cmp(&self, o: &Self) -> Option<Ordering> {
        self.val.partial_cmp(&o.val)
    }
}

impl Copy for ADNumber {}

impl Serialize for ADNumber {
    fn serialize<S>(&self, s: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        s.serialize_f64(self.val)
    }
}
impl<'de> Deserialize<'de> for ADNumber {
    fn deserialize<D>(d: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let v = f64::deserialize(d)?;
        Ok(ADNumber::new(v))
    }
}

/* ═════════════════════  CONSTANT LEAF (Const)  ═════════════════════ */

#[derive(Clone, Copy)]
pub struct Const(pub f64);

impl From<f64> for Const {
    #[inline]
    fn from(v: f64) -> Self {
        Const(v)
    }
}
impl From<f32> for Const {
    #[inline]
    fn from(v: f32) -> Self {
        Const(v as f64)
    }
}
impl From<i32> for Const {
    #[inline]
    fn from(v: i32) -> Self {
        Const(v as f64)
    }
}
impl From<u32> for Const {
    #[inline]
    fn from(v: u32) -> Self {
        Const(v as f64)
    }
}
impl From<i64> for Const {
    #[inline]
    fn from(v: i64) -> Self {
        Const(v as f64)
    }
}
impl From<u64> for Const {
    #[inline]
    fn from(v: u64) -> Self {
        Const(v as f64)
    }
}
impl From<Const> for f64 {
    #[inline]
    fn from(c: Const) -> Self {
        c.0
    }
}

impl Expr for Const {
    #[inline]
    fn inner_value(&self) -> f64 {
        self.0
    }
    #[inline]
    fn push_adj(&self, _: &mut TapeNode, _: f64) {}
}

/* ═════════════════════  BIN-OP “TYPE CLASS”  ═══════════════════════ */

pub trait BinOp {
    fn eval(l: f64, r: f64) -> f64;
    fn d_left(l: f64, r: f64) -> f64;
    fn d_right(l: f64, r: f64) -> f64;
}

/* ---- concrete binary ops ----------------------------------------- */

#[derive(Clone, Copy, Debug)]
pub struct AddOp;
impl BinOp for AddOp {
    #[inline]
    fn eval(l: f64, r: f64) -> f64 {
        l + r
    }
    #[inline]
    fn d_left(_: f64, _: f64) -> f64 {
        1.0
    }
    #[inline]
    fn d_right(_: f64, _: f64) -> f64 {
        1.0
    }
}
#[derive(Clone, Copy, Debug)]
pub struct SubOp;
impl BinOp for SubOp {
    #[inline]
    fn eval(l: f64, r: f64) -> f64 {
        l - r
    }
    #[inline]
    fn d_left(_: f64, _: f64) -> f64 {
        1.0
    }
    #[inline]
    fn d_right(_: f64, _: f64) -> f64 {
        -1.0
    }
}
#[derive(Clone, Copy, Debug)]
pub struct MulOp;
impl BinOp for MulOp {
    #[inline]
    fn eval(l: f64, r: f64) -> f64 {
        l * r
    }
    #[inline]
    fn d_left(_: f64, r: f64) -> f64 {
        r
    }
    #[inline]
    fn d_right(l: f64, _: f64) -> f64 {
        l
    }
}
#[derive(Clone, Copy, Debug)]
pub struct DivOp;
impl BinOp for DivOp {
    #[inline]
    fn eval(l: f64, r: f64) -> f64 {
        l / r
    }
    #[inline]
    fn d_left(_: f64, r: f64) -> f64 {
        1.0 / r
    }
    #[inline]
    fn d_right(l: f64, r: f64) -> f64 {
        -l / (r * r)
    }
}
#[derive(Clone, Copy, Debug)]
pub struct PowOp;
impl BinOp for PowOp {
    #[inline]
    fn eval(l: f64, r: f64) -> f64 {
        l.powf(r)
    }
    #[inline]
    fn d_left(l: f64, r: f64) -> f64 {
        r * l.powf(r - 1.0)
    }
    #[inline]
    fn d_right(l: f64, r: f64) -> f64 {
        l.powf(r) * l.ln()
    }
}

#[derive(Clone, Copy, Debug)]
pub struct MaxOp;
impl BinOp for MaxOp {
    #[inline]
    fn eval(l: f64, r: f64) -> f64 {
        l.max(r)
    }
    #[inline]
    fn d_left(l: f64, r: f64) -> f64 {
        if l > r {
            1.0
        } else {
            0.0
        }
    }
    #[inline]
    fn d_right(l: f64, r: f64) -> f64 {
        if r > l {
            1.0
        } else {
            0.0
        }
    }
}
#[derive(Clone, Copy, Debug)]
pub struct MinOp;
impl BinOp for MinOp {
    #[inline]
    fn eval(l: f64, r: f64) -> f64 {
        l.min(r)
    }
    #[inline]
    fn d_left(l: f64, r: f64) -> f64 {
        if l < r {
            1.0
        } else {
            0.0
        }
    }
    #[inline]
    fn d_right(l: f64, r: f64) -> f64 {
        if r < l {
            1.0
        } else {
            0.0
        }
    }
}

/* ═══════════════════  NODE: BinExpr (binary)  ══════════════════════ */

#[derive(Clone)]
pub struct BinExpr<L, R, O> {
    l: L,
    r: R,
    val: f64,
    _ph: std::marker::PhantomData<O>,
}
impl<L: Expr, R: Expr, O: BinOp> BinExpr<L, R, O> {
    #[inline]
    fn new(l: L, r: R) -> Self {
        let val = O::eval(l.inner_value(), r.inner_value());
        Self {
            l,
            r,
            val,
            _ph: std::marker::PhantomData,
        }
    }
}
impl<L: Expr, R: Expr, O: BinOp + Clone> Expr for BinExpr<L, R, O> {
    #[inline]
    fn inner_value(&self) -> f64 {
        self.val
    }
    fn push_adj(&self, parent: &mut TapeNode, adj: f64) {
        self.l.push_adj(
            parent,
            adj * O::d_left(self.l.inner_value(), self.r.inner_value()),
        );
        self.r.push_adj(
            parent,
            adj * O::d_right(self.l.inner_value(), self.r.inner_value()),
        );
    }
}

/* ═════════════════════  UNARY OPERATORS  ═══════════════════════════ */

pub trait UnOp {
    fn eval(x: f64) -> f64;
    fn deriv(x: f64, v: f64) -> f64;
}

/* every unary-operator struct gets Clone+Copy+Debug via derive */
macro_rules! un_op {
    ($name:ident,$eval:expr,$d:expr) => {
        #[derive(Clone, Copy, Debug)]
        pub struct $name;
        impl UnOp for $name {
            #[inline]
            fn eval(x: f64) -> f64 {
                $eval(x)
            }
            #[inline]
            fn deriv(x: f64, v: f64) -> f64 {
                $d(x, v)
            }
        }
    };
}

/* instantiate concrete unary ops */
un_op!(ExpOp, f64::exp, |_x, v| v);
un_op!(LogOp, f64::ln, |x, _| 1.0 / x);
un_op!(SqrtOp, f64::sqrt, |_x, v| 0.5 / v);
un_op!(FabsOp, f64::abs, |x, _| if x >= 0.0 { 1.0 } else { -1.0 });
un_op!(SinOp, f64::sin, |x, _v| f64::cos(x));
un_op!(CosOp, f64::cos, |x, _v| -f64::sin(x));
un_op!(AbsOp, f64::abs, |x, _v| if x >= 0.0 { 1.0 } else { -1.0 });

/* ---- node: UnExpr (unary) ---------------------------------------- */

#[derive(Clone)]
pub struct UnExpr<A, O> {
    a: A,
    val: f64,
    _ph: std::marker::PhantomData<O>,
}
impl<A: Expr, O: UnOp> UnExpr<A, O> {
    #[inline]
    fn new(a: A) -> Self {
        let val = O::eval(a.inner_value());
        Self {
            a,
            val,
            _ph: std::marker::PhantomData,
        }
    }
}
impl<A: Expr, O: UnOp + Clone> Expr for UnExpr<A, O> {
    #[inline]
    fn inner_value(&self) -> f64 {
        self.val
    }
    fn push_adj(&self, parent: &mut TapeNode, adj: f64) {
        self.a
            .push_adj(parent, adj * O::deriv(self.a.inner_value(), self.val));
    }
}

/* ═══════════════  ARITHMETIC OPERATOR OVERLOADS  ═══════════════════ */

macro_rules! impl_bin_ops_local {
    ($Self:ty) => {
        /* Add ------------------------------------------------------- */
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

        /* Sub ------------------------------------------------------- */
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

        /* Mul ------------------------------------------------------- */
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

        /* Div ------------------------------------------------------- */
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

        /* Neg ------------------------------------------------------- */
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

/* apply to both leaf types */
impl_bin_ops_local!(ADNumber);
impl_bin_ops_local!(Const);

/* ---- generic overloads for BinExpr -------------------------------- */

macro_rules! impl_bin_ops_expr {
    ($Expr:ident) => {
        /* Add ------------------------------------------------------- */
        impl<L, R, O, Rhs> Add<Rhs> for $Expr<L, R, O>
        where
            Rhs: Expr + Clone,
            Self: Expr + Clone,
        {
            type Output = BinExpr<Self, Rhs, AddOp>;
            fn add(self, rhs: Rhs) -> Self::Output {
                BinExpr::new(self, rhs)
            }
        }
        impl<L, R, O> Add<f64> for $Expr<L, R, O>
        where
            Self: Expr + Clone,
        {
            type Output = BinExpr<Self, Const, AddOp>;
            fn add(self, rhs: f64) -> Self::Output {
                BinExpr::new(self, Const(rhs))
            }
        }

        /* Sub ------------------------------------------------------- */
        impl<L, R, O, Rhs> Sub<Rhs> for $Expr<L, R, O>
        where
            Rhs: Expr + Clone,
            Self: Expr + Clone,
        {
            type Output = BinExpr<Self, Rhs, SubOp>;
            fn sub(self, rhs: Rhs) -> Self::Output {
                BinExpr::new(self, rhs)
            }
        }
        impl<L, R, O> Sub<f64> for $Expr<L, R, O>
        where
            Self: Expr + Clone,
        {
            type Output = BinExpr<Self, Const, SubOp>;
            fn sub(self, rhs: f64) -> Self::Output {
                BinExpr::new(self, Const(rhs))
            }
        }

        /* Mul ------------------------------------------------------- */
        impl<L, R, O, Rhs> Mul<Rhs> for $Expr<L, R, O>
        where
            Rhs: Expr + Clone,
            Self: Expr + Clone,
        {
            type Output = BinExpr<Self, Rhs, MulOp>;
            fn mul(self, rhs: Rhs) -> Self::Output {
                BinExpr::new(self, rhs)
            }
        }
        impl<L, R, O> Mul<f64> for $Expr<L, R, O>
        where
            Self: Expr + Clone,
        {
            type Output = BinExpr<Self, Const, MulOp>;
            fn mul(self, rhs: f64) -> Self::Output {
                BinExpr::new(self, Const(rhs))
            }
        }

        /* Div ------------------------------------------------------- */
        impl<L, R, O, Rhs> Div<Rhs> for $Expr<L, R, O>
        where
            Rhs: Expr + Clone,
            Self: Expr + Clone,
        {
            type Output = BinExpr<Self, Rhs, DivOp>;
            fn div(self, rhs: Rhs) -> Self::Output {
                BinExpr::new(self, rhs)
            }
        }
        impl<L, R, O> Div<f64> for $Expr<L, R, O>
        where
            Self: Expr + Clone,
        {
            type Output = BinExpr<Self, Const, DivOp>;
            fn div(self, rhs: f64) -> Self::Output {
                BinExpr::new(self, Const(rhs))
            }
        }

        /* Neg ------------------------------------------------------- */
        impl<L, R, O> Neg for $Expr<L, R, O>
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

impl_bin_ops_expr!(BinExpr);

/* ---- ADNumber assignment variants -------------------------------- */

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

/* ═════════════  COMPARISON TRAITS (value based)  ═══════════════════ */

/* UnExpr ≷ UnExpr & f64 */
impl<A, O> PartialEq for UnExpr<A, O>
where
    A: Expr,
    O: UnOp + Clone,
{
    fn eq(&self, rhs: &Self) -> bool {
        self.inner_value() == rhs.inner_value()
    }
}
impl<A, O> PartialOrd for UnExpr<A, O>
where
    A: Expr,
    O: UnOp + Clone,
{
    fn partial_cmp(&self, rhs: &Self) -> Option<Ordering> {
        self.inner_value().partial_cmp(&rhs.inner_value())
    }
}
impl<A, O> PartialEq<f64> for UnExpr<A, O>
where
    A: Expr,
    O: UnOp + Clone,
{
    fn eq(&self, rhs: &f64) -> bool {
        self.inner_value() == *rhs
    }
}
impl<A, O> PartialOrd<f64> for UnExpr<A, O>
where
    A: Expr,
    O: UnOp + Clone,
{
    fn partial_cmp(&self, rhs: &f64) -> Option<Ordering> {
        self.inner_value().partial_cmp(rhs)
    }
}

/* BinExpr ≷ BinExpr & f64 */
impl<L, R, O> PartialEq for BinExpr<L, R, O>
where
    L: Expr,
    R: Expr,
    O: BinOp + Clone,
{
    fn eq(&self, rhs: &Self) -> bool {
        self.inner_value() == rhs.inner_value()
    }
}
impl<L, R, O> PartialOrd for BinExpr<L, R, O>
where
    L: Expr,
    R: Expr,
    O: BinOp + Clone,
{
    fn partial_cmp(&self, rhs: &Self) -> Option<Ordering> {
        self.inner_value().partial_cmp(&rhs.inner_value())
    }
}
impl<L, R, O> PartialEq<f64> for BinExpr<L, R, O>
where
    L: Expr,
    R: Expr,
    O: BinOp + Clone,
{
    fn eq(&self, rhs: &f64) -> bool {
        self.inner_value() == *rhs
    }
}
impl<L, R, O> PartialOrd<f64> for BinExpr<L, R, O>
where
    L: Expr,
    R: Expr,
    O: BinOp + Clone,
{
    fn partial_cmp(&self, rhs: &f64) -> Option<Ordering> {
        self.inner_value().partial_cmp(rhs)
    }
}

/* ═══════════════  PUBLIC MATH HELPERS (free fns)  ═════════════════ */

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
#[inline]
pub fn sin<A: Expr + Clone>(a: A) -> UnExpr<A, SinOp> {
    UnExpr::new(a)
}
#[inline]
pub fn cos<A: Expr + Clone>(a: A) -> UnExpr<A, CosOp> {
    UnExpr::new(a)
}
#[inline]
pub fn abs<A: Expr + Clone>(a: A) -> UnExpr<A, AbsOp> {
    UnExpr::new(a)
}

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

/* ═══════  FLATTEN ANY EXPRESSION INTO A TAPE NODE (ADNumber)  ═════ */

fn flatten<E: Expr + Clone>(e: &E) -> ADNumber {
    let mut node = TapeNode::default();
    e.push_adj(&mut node, 1.0);
    let idx = TAPE.with(|t| t.borrow_mut().record(node));
    ADNumber {
        val: e.inner_value(),
        idx,
    }
}

/* ---- From conversions into ADNumber ------------------------------ */

impl<L, R, O> From<BinExpr<L, R, O>> for ADNumber
where
    L: Expr + Clone,
    R: Expr + Clone,
    O: BinOp + Clone,
{
    fn from(e: BinExpr<L, R, O>) -> Self {
        flatten(&e)
    }
}
impl<A, O> From<UnExpr<A, O>> for ADNumber
where
    A: Expr + Clone,
    O: UnOp + Clone,
{
    fn from(e: UnExpr<A, O>) -> Self {
        flatten(&e)
    }
}
impl From<f64> for ADNumber {
    fn from(v: f64) -> Self {
        ADNumber::new(v)
    }
}
impl From<f32> for ADNumber {
    fn from(v: f32) -> Self {
        ADNumber::new(v as f64)
    }
}
impl From<i32> for ADNumber {
    fn from(v: i32) -> Self {
        ADNumber::new(v as f64)
    }
}

/* ═════════════  OPTIONAL METHOD-STYLE EXTENSIONS  ═════════════════ */

pub trait FloatExt: Expr + Clone + Sized {
    #[inline]
    fn exp(self) -> UnExpr<Self, ExpOp> {
        UnExpr::new(self)
    }
    #[inline]
    fn ln(self) -> UnExpr<Self, LogOp> {
        UnExpr::new(self)
    }
    #[inline]
    fn sin(self) -> UnExpr<Self, SinOp> {
        UnExpr::new(self)
    }
    #[inline]
    fn cos(self) -> UnExpr<Self, CosOp> {
        UnExpr::new(self)
    }
    #[inline]
    fn abs(self) -> UnExpr<Self, AbsOp> {
        UnExpr::new(self)
    }

    #[inline]
    fn powf(self, p: f64) -> BinExpr<Self, Const, PowOp> {
        BinExpr::new(self, Const(p))
    }

    #[inline]
    fn sqrt(self) -> UnExpr<Self, SqrtOp> {
        UnExpr::new(self)
    }

    /// Raise `self` to an **expression** power ( x.pow_expr(y) )
    #[inline]
    fn pow_expr<R: Expr + Clone>(self, p: R) -> BinExpr<Self, R, PowOp> {
        BinExpr::new(self, p)
    }

    #[inline]
    fn min<R: Expr + Clone>(self, r: R) -> BinExpr<Self, R, MinOp> {
        BinExpr::new(self, r)
    }

    #[inline]
    fn max<R: Expr + Clone>(self, r: R) -> BinExpr<Self, R, MaxOp> {
        BinExpr::new(self, r)
    }
}
impl<T: Expr + Clone> FloatExt for T {}

/* ─────────────────  ARITHMETIC FOR UnExpr  ─────────────────────── */
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

/* Sub */
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

/* Mul */
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

/* Div */
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

/* Neg */
impl<A, O> Neg for UnExpr<A, O>
where
    Self: Expr + Clone,
{
    type Output = BinExpr<Const, Self, SubOp>;
    fn neg(self) -> Self::Output {
        BinExpr::new(Const(0.0), self)
    }
}

impl PartialEq<f64> for ADNumber {
    #[inline]
    fn eq(&self, rhs: &f64) -> bool {
        self.value() == *rhs
    }
}

impl PartialOrd<f64> for ADNumber {
    #[inline]
    fn partial_cmp(&self, rhs: &f64) -> Option<Ordering> {
        self.value().partial_cmp(rhs)
    }
}

/* ═════════════════════════  TESTS  ════════════════════════════════ */

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compare_and_flatten() {
        let x = ADNumber::new(5.0);
        let y = abs(x - 2.0);
        assert!(y > 2.0); // value-based comparison
        let z: ADNumber = (y + 1.0).into();
        assert_eq!(z.value(), 4.0);
    }

    #[test]
    fn backprop_basic() {
        let a = ADNumber::new(3.0);
        let b = ADNumber::new(4.0);
        let expr = (a * b).sin();
        let out: ADNumber = expr.into();
        out.propagate_to_start();
        assert_eq!(out.adjoint(), 1.0);
    }

    #[test]
    fn backprop_with_const() {
        let a = ADNumber::new(3.0);
        let b = Const(4.0);
        let expr = (a * b).sin();
        let out: ADNumber = expr.into();
        out.propagate_to_start();
        assert_eq!(out.adjoint(), 1.0);
    }

    #[test]
    fn tape_reset() {
        let a = ADNumber::new(3.0);
        let b = ADNumber::new(4.0);
        let expr = (a * b).sin();
        let out: ADNumber = expr.into();
        out.propagate_to_start();
        assert_eq!(out.adjoint(), 1.0);

        ADNumber::reset_adjoints(); // reset adjoints
        assert_eq!(out.adjoint(), 0.0); // should be zero now
    }

    #[test]
    fn tape_propagate_mark() {
        let a = ADNumber::new(3.0);
        let b = ADNumber::new(4.0);
        let expr = (a * b).sin();
        let out: ADNumber = expr.into();
        out.propagate_to_mark(); // propagate to the current mark
        assert_eq!(out.adjoint(), 1.0); // should be 1.0
    }

    #[test]
    fn tape_propagate_mark_to_start() {
        let a = ADNumber::new(3.0);
        let b = ADNumber::new(4.0);
        let expr = (a * b).sin();
        let out: ADNumber = expr.into();
        out.propagate_to_mark(); // propagate to the current mark
        assert_eq!(out.adjoint(), 1.0); // should be 1.0

        ADNumber::propagate_mark_to_start(); // propagate from mark to start
        assert_eq!(out.adjoint(), 1.0); // should still be 1.0
    }

    #[test]
    fn check_exp_derivate() {
        let x = ADNumber::new(2.0);
        let expr = exp(x);
        let out: ADNumber = expr.into();
        out.propagate_to_start();
        assert_eq!(x.adjoint(), f64::exp(2.0)); // derivative of exp(x) wrt x
        assert_eq!(out.adjoint(), 1.0);
    }

    #[test]
    fn check_log_derivative() {
        let x = ADNumber::new(2.0);
        let expr = log(x);
        let out: ADNumber = expr.into();
        out.propagate_to_start();
        assert_eq!(x.adjoint(), 1.0 / 2.0); // derivative of log(x) wrt x
        assert_eq!(out.adjoint(), 1.0);
    }
    #[test]
    fn check_sqrt_derivative() {
        let x = ADNumber::new(4.0);
        let expr = sqrt(x);
        let out: ADNumber = expr.into();
        out.propagate_to_start();
        assert_eq!(x.adjoint(), 0.5 / 2.0); // derivative of sqrt(x) wrt x
        assert_eq!(out.adjoint(), 1.0);
    }
    #[test]
    fn check_sin_derivative() {
        let x = ADNumber::new(0.0);
        let expr = sin(x);
        let out: ADNumber = expr.into();
        out.propagate_to_start();
        assert_eq!(x.adjoint(), 1.0); // derivative of sin(x) wrt x
        assert_eq!(out.adjoint(), 1.0);
    }
    #[test]
    fn check_cos_derivative() {
        let x = ADNumber::new(0.0);
        let expr = cos(x);
        let out: ADNumber = expr.into();
        out.propagate_to_start();
        assert_eq!(x.adjoint(), 0.0); // derivative of cos(x) wrt x
        assert_eq!(out.adjoint(), 1.0);
    }
    #[test]
    fn check_abs_derivative() {
        let x = ADNumber::new(-3.0);
        let expr = abs(x);
        let out: ADNumber = expr.into();
        out.propagate_to_start();
        assert_eq!(x.adjoint(), -1.0); // derivative of abs(x) wrt x
        assert_eq!(out.adjoint(), 1.0);
    }
    #[test]
    fn check_pow_derivative() {
        let x = ADNumber::new(2.0);
        let expr = pow(x, Const(3.0));
        let out: ADNumber = expr.into();
        out.propagate_to_start();
        assert_eq!(x.adjoint(), 3.0 * 2.0f64.powf(2.0)); // derivative of x^3 wrt x at x=2
        assert_eq!(out.adjoint(), 1.0);
    }
    #[test]
    fn check_max_derivative() {
        let x = ADNumber::new(2.0);
        let y = ADNumber::new(3.0);
        let expr = max(x, y);
        let out: ADNumber = expr.into();
        out.propagate_to_start();
        assert_eq!(x.adjoint(), 0.0); // derivative wrt x
        assert_eq!(y.adjoint(), 1.0); // derivative wrt y
        assert_eq!(out.adjoint(), 1.0);
    }
    #[test]
    fn check_min_derivative() {
        let x = ADNumber::new(2.0);
        let y = ADNumber::new(3.0);
        let expr = min(x, y);
        let out: ADNumber = expr.into();
        out.propagate_to_start();
        assert_eq!(x.adjoint(), 1.0); // derivative wrt x
        assert_eq!(y.adjoint(), 0.0); // derivative wrt y
        assert_eq!(out.adjoint(), 1.0);
    }
    #[test]
    fn check_flattening() {
        let x = ADNumber::new(5.0);
        let y = ADNumber::new(3.0);
        let expr = (x + y) * 2.0;
        let out: ADNumber = expr.into();
        assert_eq!(out.value(), 16.0); // (5 + 3) * 2 = 16
        out.propagate_to_start();
        assert_eq!(out.adjoint(), 1.0); // should be 1.0 after propagation
    }

    #[test]
    fn check_add_derivative() {
        let x = ADNumber::new(2.0);
        let y = ADNumber::new(3.0);
        let expr = x + y;
        let out: ADNumber = expr.into();
        out.propagate_to_start();
        assert_eq!(x.adjoint(), 1.0);
        assert_eq!(y.adjoint(), 1.0);
        assert_eq!(out.adjoint(), 1.0);
    }

    #[test]
    fn check_sub_derivative() {
        let x = ADNumber::new(5.0);
        let y = ADNumber::new(2.0);
        let expr = x - y;
        let out: ADNumber = expr.into();
        out.propagate_to_start();
        assert_eq!(x.adjoint(), 1.0);
        assert_eq!(y.adjoint(), -1.0);
        assert_eq!(out.adjoint(), 1.0);
    }

    #[test]
    fn check_mul_derivative() {
        let x = ADNumber::new(4.0);
        let y = ADNumber::new(2.0);
        let expr = x * y;
        let out: ADNumber = expr.into();
        out.propagate_to_start();
        assert_eq!(x.adjoint(), 2.0);
        assert_eq!(y.adjoint(), 4.0);
        assert_eq!(out.adjoint(), 1.0);
    }

    #[test]
    fn check_div_derivative() {
        let x = ADNumber::new(6.0);
        let y = ADNumber::new(3.0);
        let expr = x / y;
        let out: ADNumber = expr.into();
        out.propagate_to_start();
        assert!((x.adjoint() - (1.0 / 3.0)).abs() < 1e-12);
        assert!((y.adjoint() + (6.0 / 9.0)).abs() < 1e-12);
        assert_eq!(out.adjoint(), 1.0);
    }

    #[test]
    fn check_fabs_derivative() {
        let x = ADNumber::new(-2.0);
        let expr = fabs(x);
        let out: ADNumber = expr.into();
        out.propagate_to_start();
        assert_eq!(x.adjoint(), -1.0);
        assert_eq!(out.adjoint(), 1.0);
    }

    #[test]
    fn check_pow_variable_exponent() {
        let x = ADNumber::new(2.0);
        let y = ADNumber::new(3.0);
        let expr = pow(x, y);
        let out: ADNumber = expr.into();
        out.propagate_to_start();
        assert_eq!(x.adjoint(), 3.0 * 2.0f64.powf(2.0));
        assert!((y.adjoint() - (8.0 * 2.0f64.ln())).abs() < 1e-12);
        assert_eq!(out.adjoint(), 1.0);
    }

    #[test]
    fn check_max_derivative_x_greater() {
        let x = ADNumber::new(5.0);
        let y = ADNumber::new(3.0);
        let expr = max(x, y);
        let out: ADNumber = expr.into();
        out.propagate_to_start();
        assert_eq!(x.adjoint(), 1.0);
        assert_eq!(y.adjoint(), 0.0);
        assert_eq!(out.adjoint(), 1.0);
    }

    #[test]
    fn check_min_derivative_y_less() {
        let x = ADNumber::new(5.0);
        let y = ADNumber::new(3.0);
        let expr = min(x, y);
        let out: ADNumber = expr.into();
        out.propagate_to_start();
        assert_eq!(x.adjoint(), 0.0);
        assert_eq!(y.adjoint(), 1.0);
        assert_eq!(out.adjoint(), 1.0);
    }

    #[test]
    fn check_abs_positive_derivative() {
        let x = ADNumber::new(3.0);
        let expr = abs(x);
        let out: ADNumber = expr.into();
        out.propagate_to_start();
        assert_eq!(x.adjoint(), 1.0);
        assert_eq!(out.adjoint(), 1.0);
    }
}
