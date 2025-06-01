use std::cmp::Ordering;

use crate::prelude::*;

/// # Log-Linear Interpolator
/// Log-linear interpolator.
#[derive(Clone)]
pub struct LogLinearInterpolator {}

#[cfg(feature = "adnumber")]
impl Interpolate for LogLinearInterpolator {
    fn interpolate(
        x: NumericType,
        x_: &Vec<NumericType>,
        y_: &Vec<NumericType>,
        enable_extrapolation: bool,
    ) -> NumericType {
        let index =
            match x_.binary_search_by(|&probe| probe.partial_cmp(&x).unwrap_or(Ordering::Less)) {
                Ok(index) => index,
                Err(index) => index,
            };

        if !enable_extrapolation && (x < *x_.first().unwrap() || x > *x_.last().unwrap()) {
            panic!("Extrapolation is not enabled, and the provided value is outside the range.");
        }

        match index {
            0 => {
                let base = y_[1] / y_[0];
                let exponent = (x - x_[0]) / (x_[1] - x_[0]);
                (y_[0] * base.pow_expr(exponent)).into()
            }
            idx if idx == x_.len() => {
                let base = y_[idx - 1] / y_[idx - 2];
                let exponent = (x - x_[idx - 1]) / (x_[idx - 1] - x_[idx - 2]);
                (y_[idx - 1] * base.pow_expr(exponent)).into()
            }
            _ => {
                let base = y_[index] / y_[index - 1];
                let exponent = (x - x_[index - 1]) / (x_[index] - x_[index - 1]);
                (y_[index - 1] * base.pow_expr(exponent)).into()
            }
        }
    }
}
#[cfg(feature = "f64")]
impl Interpolate for LogLinearInterpolator {
    fn interpolate(
        x: NumericType,
        x_: &Vec<NumericType>,
        y_: &Vec<NumericType>,
        enable_extrapolation: bool,
    ) -> NumericType {
        let index =
            match x_.binary_search_by(|&probe| probe.partial_cmp(&x).unwrap_or(Ordering::Less)) {
                Ok(index) => index,
                Err(index) => index,
            };

        if !enable_extrapolation && (x < *x_.first().unwrap() || x > *x_.last().unwrap()) {
            panic!("Extrapolation is not enabled, and the provided value is outside the range.");
        }

        match index {
            0 => {
                let base = y_[1] / y_[0];
                let exponent = (x - x_[0]) / (x_[1] - x_[0]);
                (y_[0] * base.powf(exponent)).into()
            }
            idx if idx == x_.len() => {
                let base = y_[idx - 1] / y_[idx - 2];
                let exponent = (x - x_[idx - 1]) / (x_[idx - 1] - x_[idx - 2]);
                (y_[idx - 1] * base.powf(exponent)).into()
            }
            _ => {
                let base = y_[index] / y_[index - 1];
                let exponent = (x - x_[index - 1]) / (x_[index] - x_[index - 1]);
                (y_[index - 1] * base.powf(exponent)).into()
            }
        }
    }
}

#[test]
fn test_loglinear_interpolation() {
    let x = NumericType::from(0.5);
    let x_ = vec![NumericType::from(0.0), NumericType::from(1.0)];
    let y_ = vec![NumericType::from(0.1), NumericType::from(1.0)]; // Change from 0.0 to 0.1
    let y = LogLinearInterpolator::interpolate(x, &x_, &y_, true);
    // Adjust the expected value accordingly
    assert!((y.value() - 0.31622776601683794).abs() < 1e-10);
}
