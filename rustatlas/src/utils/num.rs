use std::ops::{Add, Div, Mul, Neg, Sub};

/// Trait implemented by numeric types used in pricing calculations.
pub trait Real:
    Copy
    + PartialEq
    + PartialOrd
    + Add<Output = Self>
    + Add<f64, Output = Self>
    + Sub<Output = Self>
    + Sub<f64, Output = Self>
    + Mul<Output = Self>
    + Mul<f64, Output = Self>
    + Div<Output = Self>
    + Div<f64, Output = Self>
    + Neg<Output = Self>
    + From<f64>
    + From<f32>
{
    fn ln(self) -> Self;
    fn exp(self) -> Self;
    fn powf(self, rhs: Self) -> Self;
    fn sqrt(self) -> Self;
    fn sin(self) -> Self;
    fn cos(self) -> Self;
    fn abs(self) -> Self;
    fn min(self, other: Self) -> Self {
        if self < other {
            self
        } else {
            other
        }
    }
    fn max(self, other: Self) -> Self {
        if self > other {
            self
        } else {
            other
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::math::ad::{backward, reset_tape, Var};

    #[test]
    fn real_trait_float_ops() {
        reset_tape();
        let x: Var = Var::new(2.0);
        let y = x + 3.0f64 * x - 1.0f64;
        let grad = backward(&y);
        assert!((grad[x.id()] - 4.0).abs() < 1e-12);

        let z: f64 = 2.0;
        let result = z + 1.0f64 - 0.5f64 * 2.0f64;
        assert!((result - 2.0).abs() < 1e-12);
    }
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

    fn min(self, other: Self) -> Self {
        if self < other {
            self
        } else {
            other
        }
    }

    fn max(self, other: Self) -> Self {
        if self > other {
            self
        } else {
            other
        }
    }
}
