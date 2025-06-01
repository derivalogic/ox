use crate::prelude::*;

pub trait Interpolate {
    fn interpolate(
        x: ADNumber,
        x_: &Vec<ADNumber>,
        y_: &Vec<ADNumber>,
        enable_extrapolation: bool,
    ) -> ADNumber;
}
