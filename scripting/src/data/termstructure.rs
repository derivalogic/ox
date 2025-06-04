use rustatlas::prelude::*;
use std::collections::HashMap;
use std::hash::Hash;
use std::sync::{Arc, RwLock};

use crate::utils::errors::{Result, ScriptingError};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
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

impl TermStructureKey {
    pub fn new(currency: Currency, is_risk_free: bool, name: Option<String>) -> Self {
        TermStructureKey {
            name,
            currency,
            is_risk_free,
        }
    }

    pub fn with_name(mut self, name: String) -> Self {
        self.name = Some(name);
        self
    }

    pub fn with_currency(mut self, currency: Currency) -> Self {
        self.currency = currency;
        self
    }

    pub fn with_risk_free(mut self, is_risk_free: bool) -> Self {
        self.is_risk_free = is_risk_free;
        self
    }
}

pub struct TermStructure<T: Clone> {
    key: TermStructureKey,
    year_fractions: Vec<T>,
    values: Vec<T>,
    interpolator: Interpolator,
    enable_extrapolation: bool,
    rate_definition: RateDefinition,
    term_structure_type: TermStructureType,
}

impl<T: Clone> TermStructure<T> {
    pub fn new(
        key: TermStructureKey,
        year_fractions: Vec<T>,
        values: Vec<T>,
        interpolator: Interpolator,
        enable_extrapolation: bool,
        rate_definition: RateDefinition,
        term_structure_type: TermStructureType,
    ) -> Self {
        TermStructure {
            key,
            year_fractions,
            values,
            interpolator,
            enable_extrapolation,
            rate_definition,
            term_structure_type,
        }
    }

    pub fn interpolator(&self) -> &Interpolator {
        &self.interpolator
    }

    pub fn enable_extrapolation(&self) -> bool {
        self.enable_extrapolation
    }

    pub fn rate_definition(&self) -> RateDefinition {
        self.rate_definition
    }

    pub fn year_fractions(&self) -> &[T] {
        &self.year_fractions
    }
    pub fn values(&self) -> &[T] {
        &self.values
    }

    pub fn nodes(&self) -> Vec<(T, T)> {
        self.year_fractions
            .iter()
            .cloned()
            .zip(self.values.iter().cloned())
            .collect()
    }
}

impl<T: Clone> Clone for TermStructure<T> {
    fn clone(&self) -> Self {
        TermStructure {
            key: self.key.clone(),
            year_fractions: self.year_fractions.clone(),
            values: self.values.clone(),
            interpolator: self.interpolator,
            enable_extrapolation: self.enable_extrapolation,
            rate_definition: self.rate_definition,
            term_structure_type: self.term_structure_type.clone(),
        }
    }
}

#[derive(Clone)]
pub struct TermStructureStore<T: Clone> {
    by_date: HashMap<Date, IndexesForDate<T>>,
}

#[derive(Clone)]
pub struct IndexesForDate<T: Clone> {
    /// risk-free curve, keyed only by currency  (one per currency)
    term_structures: Vec<TermStructure<T>>,
    rf_by_currency: HashMap<Currency, usize>,

    /// any curve that has `Some(name)` – keyed by that name
    by_name: HashMap<String, usize>,
}

impl<T: Clone> Default for IndexesForDate<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Clone> IndexesForDate<T> {
    pub fn new() -> Self {
        IndexesForDate {
            term_structures: Vec::new(),
            rf_by_currency: HashMap::new(),
            by_name: HashMap::new(),
        }
    }
    pub fn get_by_currency(&self, currency: Currency) -> Result<&TermStructure<T>> {
        self.rf_by_currency
            .get(&currency)
            .and_then(|&index| self.term_structures.get(index))
            .ok_or_else(|| {
                ScriptingError::NotFoundError(format!(
                    "No risk-free term structure found for currency: {:?}",
                    currency
                ))
            })
    }

    pub fn get_by_name(&self, name: &str) -> Result<&TermStructure<T>> {
        self.by_name
            .get(name)
            .and_then(|&index| self.term_structures.get(index))
            .ok_or_else(|| {
                ScriptingError::NotFoundError(format!(
                    "No term structure found with name: {}",
                    name
                ))
            })
    }

    // iterator
    pub fn iter(&self) -> impl Iterator<Item = &TermStructure<T>> {
        self.term_structures.iter()
    }
}

impl<T: Clone> TermStructureStore<T> {
    pub fn new() -> Self {
        Self {
            by_date: HashMap::new(),
        }
    }

    pub fn get_term_structures(&self, reference_date: Date) -> Result<IndexesForDate<T>> {
        self.by_date
            .get(&reference_date)
            .ok_or(ScriptingError::NotFoundError(format!(
                "No term structures found for reference date: {}",
                reference_date
            )))
            .cloned()
    }

    pub fn add_term_structure(&mut self, reference_date: Date, term_structure: TermStructure<T>) {
        let entry = self.by_date.entry(reference_date).or_default();
        entry.term_structures.push(term_structure.clone());

        // Add to risk-free currency map if applicable
        if term_structure.key.is_risk_free {
            entry
                .rf_by_currency
                .insert(term_structure.key.currency, entry.term_structures.len() - 1);
        }

        // Add to name map if applicable
        if let Some(name) = &term_structure.key.name {
            entry
                .by_name
                .insert(name.clone(), entry.term_structures.len() - 1);
        }
    }

    pub fn clear(&mut self) {
        self.by_date.clear();
    }
}

impl From<TermStructure<f64>> for TermStructure<Arc<RwLock<NumericType>>> {
    fn from(ts: TermStructure<f64>) -> Self {
        TermStructure {
            key: ts.key,
            values: ts
                .values
                .into_iter()
                .map(|v| Arc::new(RwLock::new(NumericType::new(v))))
                .collect(),
            year_fractions: ts
                .year_fractions
                .into_iter()
                .map(|v| Arc::new(RwLock::new(NumericType::new(v))))
                .collect(),
            interpolator: ts.interpolator,
            enable_extrapolation: ts.enable_extrapolation,
            rate_definition: ts.rate_definition,
            term_structure_type: ts.term_structure_type,
        }
    }
}

impl From<IndexesForDate<f64>> for IndexesForDate<Arc<RwLock<NumericType>>> {
    fn from(idxs: IndexesForDate<f64>) -> Self {
        IndexesForDate {
            term_structures: idxs
                .term_structures
                .into_iter()
                .map(|ts| ts.into())
                .collect(),
            rf_by_currency: idxs.rf_by_currency,
            by_name: idxs.by_name,
        }
    }
}

pub trait DiscountFactorProvider<T> {
    fn discount_factor(&self, from: Date, to: Date) -> Result<T>;
}

pub trait ForwardRateProvider<T>: DiscountFactorProvider<T> {
    fn fwd_rate(&self, from: Date, to: Date) -> Result<T>;
    fn fwd_rate_from_rate_definition(
        &self,
        from: Date,
        to: Date,
        rate_definition: RateDefinition,
    ) -> Result<T>;
}

impl DiscountFactorProvider<NumericType> for TermStructure<Arc<RwLock<NumericType>>> {
    fn discount_factor(&self, from: Date, to: Date) -> Result<NumericType> {
        if to < from {
            return Err(ScriptingError::InvalidOperation(
                "Date needs to be greater than reference date".to_string(),
            ));
        }
        if to == from {
            return Ok(1.0.into());
        }

        match self.term_structure_type {
            TermStructureType::FlatForward => {
                // Flat forward term structure is a special case where the discount factor is constant
                let value = self
                    .values
                    .first()
                    .ok_or(ScriptingError::NotFoundError(
                        "No values found in flat forward term structure".to_string(),
                    ))?
                    .read()
                    .unwrap()
                    .clone();
                let interest_rate = InterestRate::new(
                    value,
                    self.rate_definition.compounding(),
                    self.rate_definition.frequency(),
                    self.rate_definition.day_counter(),
                );
                return Ok(interest_rate.discount_factor(from, to).into());
            }
            TermStructureType::Zero | TermStructureType::Discount => {
                let year_fraction = self.rate_definition.day_counter().year_fraction(from, to);
                let year_fractions = self
                    .year_fractions
                    .iter()
                    .map(|v| v.read().unwrap().clone())
                    .collect::<Vec<_>>();
                let values = self
                    .values
                    .iter()
                    .map(|v| v.read().unwrap().clone())
                    .collect::<Vec<_>>();
                let discount_factor = self.interpolator.interpolate(
                    year_fraction,
                    &year_fractions,
                    &values,
                    self.enable_extrapolation,
                );
                return Ok(discount_factor);
            }
        }

        // we always interpolate?
    }
}

impl ForwardRateProvider<NumericType> for TermStructure<Arc<RwLock<NumericType>>> {
    fn fwd_rate_from_rate_definition(
        &self,
        from: Date,
        to: Date,
        rate_definition: RateDefinition,
    ) -> Result<NumericType> {
        if to < from {
            return Err(ScriptingError::InvalidOperation(
                "Date needs to be greater than reference date".to_string(),
            ));
        }
        if to == from {
            return Ok(0.0.into());
        }

        let discount_from = self.discount_factor(from, to)?;
        let discount_to = self.discount_factor(to, to)?;
        let comp = (discount_to / discount_from).into();
        let year_fraction = self.rate_definition.day_counter().year_fraction(from, to);
        let rate = InterestRate::implied_rate(
            comp,
            rate_definition.day_counter(),
            rate_definition.compounding(),
            rate_definition.frequency(),
            year_fraction,
        )?;
        Ok(rate.rate().into())
    }

    fn fwd_rate(&self, from: Date, to: Date) -> Result<NumericType> {
        self.fwd_rate_from_rate_definition(from, to, self.rate_definition)
    }
}
