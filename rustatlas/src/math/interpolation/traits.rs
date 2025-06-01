use crate::prelude::*;

pub trait Interpolate {
    fn interpolate(
        x: NumericType,
        x_: &Vec<NumericType>,
        y_: &Vec<NumericType>,
        enable_extrapolation: bool,
    ) -> NumericType;
}
