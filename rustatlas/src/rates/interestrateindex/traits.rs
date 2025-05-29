use std::{
    collections::{BTreeMap, HashMap},
    sync::{Arc, RwLock},
};

use crate::{
    math::interpolation::enums::Interpolator,
    rates::{
        traits::{HasReferenceDate, YieldProvider},
        yieldtermstructure::traits::YieldTermStructureTrait,
    },
    time::{date::Date, enums::TimeUnit, period::Period},
    utils::{errors::Result, num::Real},
};

/// # FixingProvider
/// Implement this trait for a struct that provides fixing information.
pub trait FixingProvider<T: Real> {
    fn fixing(&self, date: Date) -> Result<T>;
    fn fixings(&self) -> &HashMap<Date, T>;
    fn add_fixing(&mut self, date: Date, rate: T);

    /// Fill missing fixings using interpolation.
    fn fill_missing_fixings(&mut self, interpolator: Interpolator) {
        if !self.fixings().is_empty() {
            let first_date = self.fixings().keys().min().unwrap().clone();
            let last_date = self.fixings().keys().max().unwrap().clone();

            let aux_btreemap = self
                .fixings()
                .iter()
                .map(|(k, v)| (*k, *v))
                .collect::<BTreeMap<Date, T>>();

            let x: Vec<f64> = aux_btreemap
                .keys()
                .map(|&d| (d - first_date) as f64)
                .collect::<Vec<f64>>();

            let y = aux_btreemap
                .values()
                .map(|r| *r.into())
                .collect::<Vec<f64>>();

            let mut current_date = first_date.clone();

            while current_date <= last_date {
                if !self.fixings().contains_key(&current_date) {
                    let days = (current_date - first_date) as f64;
                    let rate = interpolator.interpolate(days, &x, &y, false);
                    self.add_fixing(current_date, T::from(rate));
                }
                current_date = current_date + Period::new(1, TimeUnit::Days);
            }
        }
    }
}

// /// # InterestRateIndexClone
// /// Trait for cloning a given object.
// pub trait InterestRateIndexClone {
//     fn clone_box(&self) -> Box<dyn InterestRateIndexTrait>;
// }

// /// # InterestRateIndexClone for T
// impl<T: 'static + InterestRateIndexTrait + Clone> InterestRateIndexClone for T {
//     fn clone_box(&self) -> Box<dyn InterestRateIndexTrait> {
//         Box::new(self.clone())
//     }
// }

// /// # Clone for Box<dyn InterestRateIndexTrait>
// /// Implementation of Clone for Box<dyn InterestRateIndexTrait>.
// impl Clone for Box<dyn InterestRateIndexTrait> {
//     fn clone(&self) -> Self {
//         self.clone_box()
//     }
// }

/// # AdvanceInterestRateIndexInTime
/// Trait for advancing in time a given object. Returns a represation of the object
/// as it would be after the given period/time.
pub trait AdvanceInterestRateIndexInTime<T: Real> {
    fn advance_to_period(
        &self,
        period: Period,
    ) -> Result<Arc<RwLock<dyn InterestRateIndexTrait<T>>>>;
    fn advance_to_date(&self, date: Date) -> Result<Arc<RwLock<dyn InterestRateIndexTrait<T>>>>;
}
/// # HasTenor
/// Implement this trait for a struct that holds a tenor.
pub trait HasTenor {
    fn tenor(&self) -> Period;
}

/// # HasTermStructure
/// Implement this trait for a struct that holds a term structure.
pub trait HasTermStructure<T: Real> {
    fn term_structure(&self) -> Result<Arc<dyn YieldTermStructureTrait<T>>>;
}

/// # HasName
/// Implement this trait for a struct that holds a name.
pub trait HasName {
    fn name(&self) -> Result<String>;
}

/// # RelinkableTermStructure
/// Allows to link a term structure to another.
pub trait RelinkableTermStructure<T: Real> {
    fn link_to(&mut self, term_structure: Arc<dyn YieldTermStructureTrait<T>>);
}

/// # InterestRateIndexTrait
/// Implement this trait for a struct that holds interest rate index information.
pub trait InterestRateIndexTrait<T: Real>:
    FixingProvider<T>
    + YieldProvider<T>
    + HasReferenceDate
    + AdvanceInterestRateIndexInTime<T>
    + HasTermStructure<T>
    + RelinkableTermStructure<T>
    + HasTenor
    + HasName
{
}
