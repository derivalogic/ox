use crate::prelude::*;

/// # DayCountProvider
/// Day count convention trait.
pub trait DayCountProvider {
    fn day_count(start: Date, end: Date) -> i64;
    fn year_fraction<T: GenericNumber>(start: Date, end: Date) -> T;
}
