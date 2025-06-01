use core::ops::*;

/// Trait used internally for generic number operations.
pub trait GenericNumber:
    Copy
    + PartialEq
    + PartialOrd
    + Add<Self, Output = Self>
    + Sub<Self, Output = Self>
    + Mul<Self, Output = Self>
    + Div<Self, Output = Self>
    + Neg<Output = Self>
    + From<f64>
    + Into<f64>
{
    fn ln(self) -> Self;
    fn exp(self) -> Self;
    fn powf(self, rhs: Self) -> Self;
    fn sqrt(self) -> Self;
    fn sin(self) -> Self;
    fn cos(self) -> Self;
    fn abs(self) -> Self;

    #[inline]
    fn min(self, other: Self) -> Self {
        if self < other { self } else { other }
    }
    #[inline]
    fn max(self, other: Self) -> Self {
        if self > other { self } else { other }
    }
}

impl GenericNumber for f64 {
    #[inline]
    fn ln(self) -> Self { self.ln() }
    #[inline]
    fn exp(self) -> Self { self.exp() }
    #[inline]
    fn powf(self, rhs: Self) -> Self { self.powf(rhs) }
    #[inline]
    fn sqrt(self) -> Self { self.sqrt() }
    #[inline]
    fn sin(self) -> Self { self.sin() }
    #[inline]
    fn cos(self) -> Self { self.cos() }
    #[inline]
    fn abs(self) -> Self { self.abs() }
}

impl GenericNumber for crate::math::ad::adnumber::ADNumber {
    #[inline]
    fn ln(self) -> Self { self.ln().into() }
    #[inline]
    fn exp(self) -> Self { self.exp().into() }
    #[inline]
    fn powf(self, rhs: Self) -> Self { self.pow_expr(rhs).into() }
    #[inline]
    fn sqrt(self) -> Self { self.sqrt().into() }
    #[inline]
    fn sin(self) -> Self { self.sin().into() }
    #[inline]
    fn cos(self) -> Self { self.cos().into() }
    #[inline]
    fn abs(self) -> Self { self.abs().into() }
}
