use std::ops::{Add, Div, Mul, Neg, Sub};

/// Trait implemented by numeric types used in pricing calculations.
pub trait Real:
    Copy
    + PartialEq
    + PartialOrd
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
    fn sqrt(self) -> Self;
    fn sin(self) -> Self;
    fn cos(self) -> Self;
    fn abs(self) -> Self;
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

    fn sqrt(self) -> Self {
        f64::sqrt(self)
    }

    fn sin(self) -> Self {
        f64::sin(self)
    }

    fn cos(self) -> Self {
        f64::cos(self)
    }

    fn abs(self) -> Self {
        f64::abs(self)
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

    fn sqrt(self) -> Self {
        self.sqrt()
    }

    fn sin(self) -> Self {
        self.sin()
    }

    fn cos(self) -> Self {
        self.cos()
    }

    fn abs(self) -> Self {
        self.abs()
    }
}
