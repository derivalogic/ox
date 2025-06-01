use std::cmp::Ordering;

use crate::prelude::*;

/// # Linear Interpolator
/// Basic linear interpolator.
#[derive(Clone)]
pub struct LinearInterpolator {}

impl Interpolate for LinearInterpolator {
    fn interpolate(
        x: NumericType,
        x_: &Vec<NumericType>,
        y_: &Vec<NumericType>,
        enable_extrapolation: bool,
    ) -> NumericType {
        let index =
            match x_.binary_search_by(|&probe| probe.partial_cmp(&x).unwrap_or(Ordering::Equal)) {
                Ok(index) => index,
                Err(index) => index,
            };

        if !enable_extrapolation {
            if x < *x_.first().unwrap() || x > *x_.last().unwrap() {
                panic!(
                    "Extrapolation is not enabled, and the provided value is outside the range."
                );
            }
        }

        match index {
            0 => (y_[0] + (x - x_[0]) * (y_[1] - y_[0]) / (x_[1] - x_[0])).into(),
            index if index == x_.len() => (y_[index - 1]
                + (x - x_[index - 1]) * (y_[index - 1] - y_[index - 2])
                    / (x_[index - 1] - x_[index - 2]))
                .into(),
            _ => (y_[index - 1]
                + (x - x_[index - 1]) * (y_[index] - y_[index - 1]) / (x_[index] - x_[index - 1]))
                .into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_linear_interpolation() {
        let x = 0.5.into();
        let x_ = vec![0.0.into(), 1.0.into()];
        let y_ = vec![0.0.into(), 1.0.into()];
        let y = LinearInterpolator::interpolate(x, &x_, &y_, true);
        assert_eq!(y, 0.5);
    }
}
