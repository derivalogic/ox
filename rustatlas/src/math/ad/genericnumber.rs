use core::fmt::{Debug, Display};
use core::ops::*;

pub trait GenericNumber
where
    /* basic capabilities ------------------------------------------------- */
    Self: Copy
        + Debug
        + Display
        + PartialEq
        + PartialOrd
        /* operators we want to use in generic code ------------------------ */
        + Add<Self>
        + Add<f64>
        + Sub<Self>
        + Sub<f64>
        + Mul<Self>
        + Mul<f64>
        + Div<Self>
        + Div<f64>
        + Neg
        /* literal conversions -------------------------------------------- */
        + From<f64>
        + From<f32>
        + From<i32>,
    /* every operator’s output can be turned *back* into `Self` ------------ */
    Self: From<<Self as Add<Self>>::Output>
        + From<<Self as Add<f64>>::Output>
        + From<<Self as Sub<Self>>::Output>
        + From<<Self as Sub<f64>>::Output>
        + From<<Self as Mul<Self>>::Output>
        + From<<Self as Mul<f64>>::Output>
        + From<<Self as Div<Self>>::Output>
        + From<<Self as Div<f64>>::Output>
        + From<<Self as Neg>::Output>,
{
    /* ───── elementary maths ───── */
    fn ln(self) -> Self;
    fn exp(self) -> Self;
    fn powf(self, rhs: Self) -> Self;
    fn sqrt(self) -> Self;
    fn sin(self) -> Self;
    fn cos(self) -> Self;
    fn abs(self) -> Self;

    /* min / max left unchanged */
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

    /* helpers so `c ? x` compile in generic code */
    #[inline]
    fn add_to_const(c: f64, x: Self) -> Self {
        (x + c).into()
    }
    #[inline]
    fn sub_from_const(c: f64, x: Self) -> Self {
        (Self::from(c) - x).into()
    }
    #[inline]
    fn mul_to_const(c: f64, x: Self) -> Self {
        (x * c).into()
    }
    #[inline]
    fn div_from_const(c: f64, x: Self) -> Self {
        (Self::from(c) / x).into()
    }
}
