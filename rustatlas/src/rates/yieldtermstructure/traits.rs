use std::sync::Arc;

use crate::{
    rates::traits::{HasReferenceDate, YieldProvider},
    time::{date::Date, period::Period},
    utils::{errors::Result, num::Real},
};

// /// # YieldTermStructureTraitClone
// /// Trait for cloning a given object.
// pub trait YieldTermStructureTraitClone {
//     fn clone_box(&self) -> Box<dyn YieldTermStructureTrait>;
// }

// /// # YieldTermStructureTraitClone for T
// /// Implementation of YieldTermStructureTraitClone for T.
// impl<T: 'static + YieldTermStructureTrait + Clone> YieldTermStructureTraitClone for T {
//     fn clone_box(&self) -> Box<dyn YieldTermStructureTrait> {
//         Box::new(self.clone())
//     }
// }

// /// # Clone for Box<dyn YieldTermStructureTrait>
// impl Clone for Box<dyn YieldTermStructureTrait> {
//     fn clone(&self) -> Self {
//         self.clone_box()
//     }
// }

/// # AdvanceTermStructureInTime
/// Trait for advancing in time a given object. Returns a represation of the object
/// as it would be after the given period.
pub trait AdvanceTermStructureInTime<T: Real> {
    fn advance_to_period(&self, period: Period) -> Result<Arc<dyn YieldTermStructureTrait<T>>>;
    fn advance_to_date(&self, date: Date) -> Result<Arc<dyn YieldTermStructureTrait<T>>>;
}

/// # YieldTermStructureTrait
/// Trait that defines a yield term structure.
///
/// ## Note
/// This trait is a combination of the following traits:
/// - YieldProvider
/// - HasReferenceDate
/// - AdvanceTermStructureInTime
/// - Send
/// - Sync
///
/// These auto traits are required to be able to share term structures
/// across threads when generating Monte–Carlo scenarios in parallel.
pub trait YieldTermStructureTrait<T: Real>:
    YieldProvider<T>
    + HasReferenceDate
    + AdvanceTermStructureInTime<T>
    + Send
    + Sync
{
}
