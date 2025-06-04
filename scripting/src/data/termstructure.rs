use rustatlas::prelude::*;
use std::hash::Hash;
use std::sync::{Arc, RwLock};
use std::{collections::HashMap, ops::Index};

use crate::utils::errors::{Result, ScriptingError};

pub enum TermStructureType {
    Discount,
    FlatForward,
    Zero,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct TermStructureKey {
    pub name: Option<String>,
    pub currency: Currency,
    pub is_risk_free: bool,
}

pub struct TermStructure<T> {
    key: TermStructureKey,
    values: HashMap<Date, T>,
    interpolation: Interpolator,
    enable_extrapolation: bool,
    day_counter: DayCounter,
    compounding: Compounding,
    frequency: Frequency,
}

impl<T> TermStructure<T> {
    pub fn new(
        key: TermStructureKey,
        values: HashMap<Date, T>,
        interpolation: Interpolator,
        enable_extrapolation: bool,
        day_counter: DayCounter,
        compounding: Compounding,
        frequency: Frequency,
    ) -> Self {
        TermStructure {
            key,
            values,
            interpolation,
            enable_extrapolation,
            day_counter,
            compounding,
            frequency,
        }
    }

    pub fn key(&self) -> &TermStructureKey {
        &self.key
    }

    pub fn values(&self) -> &HashMap<Date, T> {
        &self.values
    }

    pub fn interpolation(&self) -> &Interpolator {
        &self.interpolation
    }

    pub fn enable_extrapolation(&self) -> bool {
        self.enable_extrapolation
    }

    pub fn day_counter(&self) -> DayCounter {
        self.day_counter
    }

    pub fn compounding(&self) -> Compounding {
        self.compounding
    }

    pub fn frequency(&self) -> Frequency {
        self.frequency
    }
}

pub struct TermStructureStore<T> {
    /// one entry per reference date
    by_date: HashMap<Date, IndexesForDate<T>>,
}

struct IndexesForDate<T> {
    /// master table – the curve lives exactly here
    by_key: HashMap<TermStructureKey, Arc<TermStructure<T>>>,

    /// risk-free curve, keyed only by currency  (one per currency)
    rf_by_currency: HashMap<Currency, Arc<TermStructure<T>>>,

    /// any curve that has `Some(name)` – keyed by that name
    by_name: HashMap<String, Arc<TermStructure<T>>>,
}
impl<T> IndexesForDate<T> {
    pub fn get_by_key(&self, key: &TermStructureKey) -> Option<&Arc<TermStructure<T>>> {
        self.by_key.get(key)
    }

    pub fn get_by_currency(&self, currency: Currency) -> Option<&Arc<TermStructure<T>>> {
        self.rf_by_currency.get(&currency)
    }

    pub fn get_by_name(&self, name: &str) -> Option<&Arc<TermStructure<T>>> {
        self.by_name.get(name)
    }

    pub fn get_all(&self) -> Vec<&Arc<TermStructure<T>>> {
        self.by_key.values().collect()
    }
}

impl<T> TermStructureStore<T> {
    pub fn new() -> Self {
        Self {
            by_date: HashMap::new(),
        }
    }

    pub fn add_term_structure(
        &mut self,
        reference_date: Date,
        curve: TermStructure<T>,
    ) -> Result<()> {
        let idx = self
            .by_date
            .entry(reference_date)
            .or_insert_with(|| IndexesForDate {
                by_key: HashMap::new(),
                rf_by_currency: HashMap::new(),
                by_name: HashMap::new(),
            });

        let key = curve.key().clone();
        let curve = Arc::new(curve);

        // ── 1. master table ───────────────────────────────────────────────────
        if idx.by_key.insert(key.clone(), Arc::clone(&curve)).is_some() {
            return Err(ScriptingError::InvalidOperation(format!(
                "duplicate term structure {:?} for {reference_date}",
                key
            )));
        }

        // ── 2. risk-free index (only one per currency is allowed) ────────────
        if key.is_risk_free {
            if idx
                .rf_by_currency
                .insert(key.currency, Arc::clone(&curve))
                .is_some()
            {
                return Err(ScriptingError::InvalidOperation(format!(
                    "more than one risk-free curve for {reference_date} / {:?}",
                    key.currency
                )));
            }
        }

        // ── 3. name index ────────────────────────────────────────────────────
        if let Some(ref name) = key.name {
            idx.by_name.insert(name.clone(), Arc::clone(&curve));
        }
        Ok(())
    }

    pub fn get_term_structures(&self, reference_date: Date) -> Result<&IndexesForDate<T>> {
        self.by_date
            .get(&reference_date)
            .ok_or(ScriptingError::NotFoundError(format!(
                "No term structures found for reference date: {}",
                reference_date
            )))
    }

    pub fn clear(&mut self) {
        self.by_date.clear();
    }
}

impl From<TermStructure<f64>> for TermStructure<RwLock<NumericType>> {
    fn from(ts: TermStructure<f64>) -> Self {
        TermStructure::new(
            ts.key,
            ts.values
                .into_iter()
                .map(|(k, v)| (k, RwLock::new(NumericType::new(v))))
                .collect(),
            ts.interpolation,
            ts.enable_extrapolation,
            ts.day_counter,
            ts.compounding,
            ts.frequency,
        )
    }
}

impl From<IndexesForDate<f64>> for IndexesForDate<RwLock<NumericType>> {
    fn from(idxs: IndexesForDate<f64>) -> Self {
        IndexesForDate {
            by_key: idxs
                .by_key
                .into_iter()
                .map(|(k, v)| (k, Arc::new(TermStructure::<RwLock<NumericType>>::from(v))))
                .collect(),
            rf_by_currency: idxs
                .rf_by_currency
                .into_iter()
                .map(|(k, v)| (k, Arc::new(TermStructure::<RwLock<NumericType>>::from(*v))))
                .collect(),
            by_name: idxs
                .by_name
                .into_iter()
                .map(|(k, v)| (k, Arc::new(TermStructure::<RwLock<NumericType>>::from(*v))))
                .collect(),
        }
    }
}
