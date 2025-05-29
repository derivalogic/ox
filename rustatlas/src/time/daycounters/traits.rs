use crate::{time::date::Date, utils::num::Real};


/// # DayCountProvider
/// Day count convention trait.
pub trait DayCountProvider {
    fn day_count(start: Date, end: Date) -> i64;
    fn year_fraction<T: Real>(start: Date, end: Date) -> T;
}
