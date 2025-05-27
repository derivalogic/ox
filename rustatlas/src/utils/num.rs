use std::ops::{Add, Sub, Mul, Div, Neg};

/// Trait implemented by numeric types used in pricing calculations.
pub trait Real:
    Copy
    + Add<Output = Self>
    + Sub<Output = Self>
    + Mul<Output = Self>
    + Div<Output = Self>
    + Neg<Output = Self>
    + From<f64>
{
    fn ln(self) -> Self;
    fn exp(self) -> Self;
    fn powf(self, rhs: Self) -> Self;
}

impl Real for f64 {
    fn ln(self) -> Self {
        f64::ln(self)
    }

    fn exp(self) -> Self {
        f64::exp(self)
    }

    fn powf(self, rhs: Self) -> Self {
        f64::powf(self, rhs)
    }
}

impl Real for crate::math::ad::Var {
    fn ln(self) -> Self {
        self.ln()
    }

    fn exp(self) -> Self {
        self.exp()
    }

    fn powf(self, rhs: Self) -> Self {
        self.powf(rhs)
    }
}
