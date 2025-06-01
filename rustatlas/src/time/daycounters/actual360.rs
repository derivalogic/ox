use super::traits::DayCountProvider;
use crate::{math::ad::num::Real, time::date::Date};

/// # Actual360
/// Actual/360 day count convention.
/// Calculates the day count fraction according to the formula:
/// $$
/// \frac{ActualDays}{360}
/// $$
/// where ActualDays is the number of days between the start date and the end date.
/// # Example
/// ```
/// use rustatlas::prelude::*;
///
/// let start = Date::new(2020, 1, 1);
/// let end = Date::new(2020, 2, 1);
/// assert_eq!(Actual360::day_count(start, end), 31);
/// assert_eq!(Actual360::year_fraction::<f64>(start, end), 31.0 / 360.0);
/// ```
pub struct Actual360;

impl DayCountProvider for Actual360 {
    fn day_count(start: Date, end: Date) -> i64 {
        return end - start;
    }

    fn year_fraction<T: Real>(start: Date, end: Date) -> T {
        T::from(Actual360::day_count(start, end) as f64) / T::from(360.0)
    }
}
