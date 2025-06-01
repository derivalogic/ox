use std::{
    collections::{BTreeMap, HashMap},
    sync::{Arc, RwLock},
};

use crate::prelude::*;
/// # FixingProvider
/// Implement this trait for a struct that provides fixing information.
pub trait FixingProvider {
    fn fixing(&self, date: Date) -> Result<NumericType>;
    fn fixings(&self) -> &HashMap<Date, NumericType>;
    fn add_fixing(&mut self, date: Date, rate: NumericType);

    /// Fill missing fixings using interpolation.
    fn fill_missing_fixings(&mut self, interpolator: Interpolator) {
        if !self.fixings().is_empty() {
            let first_date = self.fixings().keys().min().unwrap().clone();
            let last_date = self.fixings().keys().max().unwrap().clone();

            let aux_btreemap = self
                .fixings()
                .iter()
                .map(|(k, v)| (*k, *v))
                .collect::<BTreeMap<Date, NumericType>>();

            let x: Vec<NumericType> = aux_btreemap
                .keys()
                .map(|&d| NumericType::new(d - first_date))
                .collect();

            let y = aux_btreemap.values().cloned().collect::<Vec<NumericType>>();

            let mut current_date = first_date.clone();

            while current_date <= last_date {
                if !self.fixings().contains_key(&current_date) {
                    let days = NumericType::new(current_date - first_date);
                    let rate = interpolator.interpolate(days, &x, &y, false);
                    self.add_fixing(current_date, rate);
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
pub trait AdvanceInterestRateIndexInTime {
    fn advance_to_period(&self, period: Period) -> Result<Arc<RwLock<dyn InterestRateIndexTrait>>>;
    fn advance_to_date(&self, date: Date) -> Result<Arc<RwLock<dyn InterestRateIndexTrait>>>;
}
/// # HasTenor
/// Implement this trait for a struct that holds a tenor.
pub trait HasTenor {
    fn tenor(&self) -> Period;
}

/// # HasTermStructure
/// Implement this trait for a struct that holds a term structure.
pub trait HasTermStructure {
    fn term_structure(&self) -> Result<Arc<dyn YieldTermStructureTrait>>;
}

/// # HasName
/// Implement this trait for a struct that holds a name.
pub trait HasName {
    fn name(&self) -> Result<String>;
}

/// # RelinkableTermStructure
/// Allows to link a term structure to another.
pub trait RelinkableTermStructure {
    fn link_to(&mut self, term_structure: Arc<dyn YieldTermStructureTrait>);
}

/// # InterestRateIndexTrait
/// Implement this trait for a struct that holds interest rate index information.
///
/// The trait is required to be [`Send`] and [`Sync`] so that references to index
/// objects can be safely shared across threads during parallel Monte-Carlo
/// simulations.
pub trait InterestRateIndexTrait:
    FixingProvider
    + YieldProvider
    + HasReferenceDate
    + AdvanceInterestRateIndexInTime
    + HasTermStructure
    + RelinkableTermStructure
    + HasTenor
    + HasName
    + Send
    + Sync
{
}
