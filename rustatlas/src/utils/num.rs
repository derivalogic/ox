//! Reverse-mode AD with constant folding and a `Real` trait.
#![allow(clippy::many_single_char_names)]

use std::{
    fmt::{Debug, Display},
    ops::{Add, Div, Mul, Neg, Sub},
};

/* ========================================================================
 *  1.  Real trait – kept small but expressive
 * ===================================================================== */

/// Numeric type accepted by all pricing algorithms.

pub trait Real:
    Copy
    + Send
    + Sync
    + Debug
    + Display
    + PartialEq
    + PartialOrd
    + Add<Output = Self>
    + Add<f64, Output = Self>         //  x + c
    + Sub<Output = Self>
    + Sub<f64, Output = Self>         //  x - c
    + Mul<Output = Self>
    + Mul<f64, Output = Self>         //  x * c
    + Div<Output = Self>
    + Div<f64, Output = Self>         //  x / c
    + Neg<Output = Self>
    + From<f64>
    + From<f32>
    + From<i32>
{
    /* elementary functions */
    fn ln(self)   -> Self;
    fn exp(self)  -> Self;
    fn powf(self, rhs: Self) -> Self;
    fn sqrt(self) -> Self;
    fn sin(self)  -> Self;
    fn cos(self)  -> Self;
    fn abs(self)  -> Self;

    #[inline] fn min(self, other: Self) -> Self { if self < other { self } else { other } }
    #[inline] fn max(self, other: Self) -> Self { if self > other { self } else { other } }

    /* ---- OPTIONAL helpers so `c ? x` compiles in generic code ---- */
    #[inline] fn add_to_const(c: f64, x: Self) -> Self { x + c }        //  c  + x  →  x + c
    fn sub_from_const(c: f64, x: Self) -> Self { Self::from(c) - x }
    #[inline] fn mul_to_const(c: f64, x: Self) -> Self { x * c }        //  c  * x  →  x * c
    fn div_from_const(c: f64, x: Self) -> Self { Self::from(c) / x }
}

/* ------------ blanket impl for f64 (trivial) ----------------------- */

impl Real for f64 {
    #[inline]
    fn ln(self) -> Self {
        f64::ln(self)
    }
    #[inline]
    fn exp(self) -> Self {
        f64::exp(self)
    }
    #[inline]
    fn powf(self, rhs: Self) -> Self {
        f64::powf(self, rhs)
    }
    #[inline]
    fn sqrt(self) -> Self {
        f64::sqrt(self)
    }
    #[inline]
    fn sin(self) -> Self {
        f64::sin(self)
    }
    #[inline]
    fn cos(self) -> Self {
        f64::cos(self)
    }
    #[inline]
    fn abs(self) -> Self {
        f64::abs(self)
    }
}
