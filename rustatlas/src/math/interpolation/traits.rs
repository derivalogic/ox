/// # Interpolation trait
/// A trait that defines the interpolation of a function.
use crate::utils::num::Real;

pub trait Interpolate<T: Real> {
    fn interpolate(x: T, x_: &Vec<T>, y_: &Vec<T>, enable_extrapolation: bool) -> T;
}
