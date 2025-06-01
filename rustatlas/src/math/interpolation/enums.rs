use serde::{Deserialize, Serialize};

use crate::prelude::*;

/// # Interpolator
/// Enum that represents the type of interpolation.
///
/// ## Example
/// ```
/// use rustatlas::prelude::*;
/// let x = 1.0.into();
/// let x_ = vec![0.0.into(), 1.0.into(), 2.0.into()];
/// let y_ = vec![0.0.into(), 1.0.into(), 4.0.into()];
/// let interpolator = Interpolator::Linear;
/// let y = interpolator.interpolate(x, &x_, &y_, true);
/// assert_eq!(y.value(), 1.0);
/// ```
#[derive(Serialize, Deserialize, Clone, Copy, Debug)]
pub enum Interpolator {
    Linear,
    LogLinear,
}

impl Interpolator {
    pub fn interpolate(
        &self,
        x: NumericType,
        x_: &Vec<NumericType>,
        y_: &Vec<NumericType>,
        enable_extrapolation: bool,
    ) -> NumericType {
        match self {
            Interpolator::Linear => {
                LinearInterpolator::interpolate(x, x_, y_, enable_extrapolation)
            }
            Interpolator::LogLinear => {
                LogLinearInterpolator::interpolate(x, x_, y_, enable_extrapolation)
            }
        }
    }
}
