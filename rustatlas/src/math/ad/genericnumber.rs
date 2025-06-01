// use core::fmt::{Debug, Display};
// use core::ops::*;

// pub trait GenericNumber
// where
//     /* ─── base capabilities ─────────────────────────────────────────── */
//     Self: Copy
//         + Debug
//         + Display
//         + PartialEq
//         + PartialOrd
//         + Add<Self>
//         + Add<f64>
//         + Sub<Self>
//         + Sub<f64>
//         + Mul<Self>
//         + Mul<f64>
//         + Div<Self>
//         + Div<f64>
//         + Neg
//         + From<f64>
//         + From<f32>
//         + From<i32>,
// {
//     /* maths (unchanged) */
//     fn ln(self) -> Self;
//     fn exp(self) -> Self;
//     fn powf(self, rhs: Self) -> Self;
//     fn sqrt(self) -> Self;
//     fn sin(self) -> Self;
//     fn cos(self) -> Self;
//     fn abs(self) -> Self;

//     #[inline]
//     fn min(self, other: Self) -> Self {
//         if self < other {
//             self
//         } else {
//             other
//         }
//     }
//     #[inline]
//     fn max(self, other: Self) -> Self {
//         if self > other {
//             self
//         } else {
//             other
//         }
//     }

//     #[inline]
//     fn add_to_const(c: f64, x: Self) -> Self {
//         (x + c).into()
//     }
//     #[inline]
//     fn sub_from_const(c: f64, x: Self) -> Self {
//         (Self::from(c) - x).into()
//     }
//     #[inline]
//     fn mul_to_const(c: f64, x: Self) -> Self {
//         (x * c).into()
//     }
//     #[inline]
//     fn div_from_const(c: f64, x: Self) -> Self {
//         (Self::from(c) / x).into()
//     }
// }
