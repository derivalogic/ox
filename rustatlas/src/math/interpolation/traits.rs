use crate::prelude::*;

pub trait Interpolate<T: GenericNumber> {
    fn interpolate(x: T, x_: &Vec<T>, y_: &Vec<T>, enable_extrapolation: bool) -> T;
}
