use super::enums::Compounding;
use crate::{
    time::{date::Date, enums::Frequency},
    utils::{errors::Result, num::Real},
};

/// # HasReferenceDate
/// Implement this trait for a struct that has a reference date.
pub trait HasReferenceDate {
    fn reference_date(&self) -> Date;
}

/// # YieldProvider
/// Implement this trait for a struct that provides yield information.
pub trait YieldProvider<T: Real>: HasReferenceDate {
    fn discount_factor(&self, date: Date) -> Result<T>;
    fn forward_rate(
        &self,
        start_date: Date,
        end_date: Date,
        comp: Compounding,
        freq: Frequency,
    ) -> Result<T>;
}
