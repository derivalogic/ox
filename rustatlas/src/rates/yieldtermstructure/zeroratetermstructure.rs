use std::sync::Arc;

use crate::math::ad::num::Real;
use crate::{
    math::interpolation::enums::Interpolator,
    rates::{
        enums::Compounding,
        interestrate::{InterestRate, RateDefinition},
        traits::{HasReferenceDate, YieldProvider},
    },
    time::{
        date::Date,
        enums::{Frequency, TimeUnit},
        period::Period,
    },
    utils::errors::{AtlasError, Result},
};

use super::traits::{AdvanceTermStructureInTime, YieldTermStructureTrait};

/// # ZeroRateTermStructure
/// Struct that defines a zero rate term structure.
///
/// # Example
/// ```
/// use rustatlas::prelude::*;
/// let ref_date = Date::new(2021, 1, 1);
/// let dates = vec![
///    Date::new(2021, 1, 1),
///    Date::new(2021, 4, 1),
///    Date::new(2021, 7, 1),
///    Date::new(2021, 10, 1),
///    Date::new(2022, 1, 1),
/// ];
///
/// let rates = vec![0.0, 0.01, 0.02, 0.03, 0.04];
/// let rate_definition = RateDefinition::default();
/// let interpolator = Interpolator::Linear;
/// let enable_extrapolation = true;
/// let zero_rate_curve = ZeroRateTermStructure::new(ref_date, dates, rates, rate_definition, interpolator, enable_extrapolation).unwrap();
/// assert_eq!(zero_rate_curve.reference_date(), ref_date);
/// assert_eq!(zero_rate_curve.rate_definition().day_counter(), DayCounter::Actual360);
/// ```
///
/// Using the AD variable type:
/// ```
/// use rustatlas::prelude::*;
/// use rustatlas::math::ad::real::Var;
/// let ref_date = Date::new(2021, 1, 1);
/// let dates = vec![ref_date, ref_date + Period::new(1, TimeUnit::Years)];
/// let rates = vec![Var::from(0.0), Var::from(0.01)];
/// let curve = ZeroRateTermStructure::<Var>::new(
///     ref_date,
///     dates,
///     rates,
///     RateDefinition::default(),
///     Interpolator::Linear,
///     true,
/// ).unwrap();
/// assert_eq!(curve.reference_date(), ref_date);
/// ```
#[derive(Clone)]
pub struct ZeroRateTermStructure<T: Real = f64> {
    reference_date: Date,
    dates: Vec<Date>,
    year_fractions: Vec<T>,
    rates: Vec<T>,
    rate_definition: RateDefinition,
    interpolator: Interpolator,
    enable_extrapolation: bool,
}

impl<T: Real> ZeroRateTermStructure<T> {
    pub fn new(
        reference_date: Date,
        dates: Vec<Date>,
        rates: Vec<T>,
        rate_definition: RateDefinition,
        interpolator: Interpolator,
        enable_extrapolation: bool,
    ) -> Result<ZeroRateTermStructure<T>> {
        // check if dates and rates have the same size
        if dates.len() != rates.len() {
            return Err(AtlasError::InvalidValueErr(
                "Dates and rates need to have the same size".to_string(),
            ));
        }

        // year_fractions[0] needs to be 0.0
        if dates[0] != reference_date {
            return Err(AtlasError::InvalidValueErr(
                "First date needs to be equal to reference date".to_string(),
            ));
        }

        let year_fractions: Vec<T> = dates
            .iter()
            .map(|x| {
                rate_definition
                    .day_counter()
                    .year_fraction::<T>(reference_date, *x)
            })
            .collect();

        Ok(ZeroRateTermStructure {
            reference_date,
            dates,
            year_fractions,
            rates,
            rate_definition,
            interpolator,
            enable_extrapolation,
        })
    }

    pub fn dates(&self) -> &Vec<Date> {
        return &self.dates;
    }

    pub fn rates(&self) -> &Vec<T> {
        return &self.rates;
    }

    pub fn rate_definition(&self) -> RateDefinition {
        return self.rate_definition;
    }

    pub fn enable_extrapolation(&self) -> bool {
        return self.enable_extrapolation;
    }

    pub fn interpolator(&self) -> Interpolator {
        return self.interpolator;
    }
}

impl<T: Real> HasReferenceDate for ZeroRateTermStructure<T> {
    fn reference_date(&self) -> Date {
        return self.reference_date;
    }
}

impl<T: Real> YieldProvider<T> for ZeroRateTermStructure<T> {
    fn discount_factor(&self, date: Date) -> Result<T> {
        let year_fraction = self
            .rate_definition()
            .day_counter()
            .year_fraction::<T>(self.reference_date(), date);

        let rate = self.interpolator.interpolate(
            year_fraction,
            &self.year_fractions,
            &self.rates,
            self.enable_extrapolation,
        );
        let rt = InterestRate::from_rate_definition(rate, self.rate_definition());
        let compound = rt.compound_factor_from_yf(year_fraction);
        Ok(T::from(1.0) / compound)
    }

    fn forward_rate(
        &self,
        start_date: Date,
        end_date: Date,
        comp: Compounding,
        freq: Frequency,
    ) -> Result<T> {
        let df_to_star = self.discount_factor(start_date)?;
        let df_to_end = self.discount_factor(end_date)?;

        let comp_factor = df_to_star / df_to_end;

        let t = self
            .rate_definition()
            .day_counter()
            .year_fraction::<T>(start_date, end_date);

        let forward_rate = InterestRate::implied_rate(
            comp_factor,
            self.rate_definition().day_counter(),
            comp,
            freq,
            t,
        )?
        .rate();

        Ok(forward_rate)
    }
}

/// # AdvanceTermStructureInTime for ZeroRateTermStructure
impl<T: Real + Send + Sync + 'static> AdvanceTermStructureInTime<T> for ZeroRateTermStructure<T> {
    fn advance_to_period(&self, period: Period) -> Result<Arc<dyn YieldTermStructureTrait<T>>> {
        let new_reference_date = self
            .reference_date()
            .advance(period.length(), period.units());

        let new_dates: Vec<Date> = self
            .dates()
            .iter()
            .map(|x| x.advance(period.length(), period.units()))
            .collect();

        let start_df = self.discount_factor(new_dates[0])?;
        let shifted_dfs: Result<Vec<T>> = new_dates
            .iter()
            .map(|x| {
                let df = self.discount_factor(*x)?;
                Ok(df / start_df)
            })
            .collect();

        Ok(Arc::new(ZeroRateTermStructure::new(
            new_reference_date,
            new_dates,
            shifted_dfs?,
            self.rate_definition(),
            self.interpolator(),
            self.enable_extrapolation(),
        )?))
    }

    fn advance_to_date(&self, date: Date) -> Result<Arc<dyn YieldTermStructureTrait<T>>> {
        let days = (date - self.reference_date()) as i32;
        if days < 0 {
            return Err(AtlasError::InvalidValueErr(format!(
                "Date {:?} is before reference date {:?}",
                date,
                self.reference_date()
            )));
        }
        let period = Period::new(days, TimeUnit::Days);
        return self.advance_to_period(period);
    }
}

impl<T: Real + Send + Sync + 'static> YieldTermStructureTrait<T> for ZeroRateTermStructure<T> {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::time::daycounter::DayCounter;

    #[test]
    fn test_zero_rate_curve() {
        let reference_date = Date::new(2020, 1, 1);
        let dates = vec![
            Date::new(2020, 1, 1),
            Date::new(2020, 4, 1),
            Date::new(2020, 7, 1),
            Date::new(2020, 10, 1),
            Date::new(2021, 1, 1),
        ];
        let rates = vec![0.0, 0.01, 0.02, 0.03, 0.04];
        let rate_definition = RateDefinition::default();

        let zero_rate_curve = ZeroRateTermStructure::new(
            reference_date,
            dates,
            rates,
            rate_definition,
            Interpolator::Linear,
            true,
        )
        .unwrap();

        assert_eq!(zero_rate_curve.reference_date(), reference_date);
        assert_eq!(
            zero_rate_curve.dates(),
            &vec![
                Date::new(2020, 1, 1),
                Date::new(2020, 4, 1),
                Date::new(2020, 7, 1),
                Date::new(2020, 10, 1),
                Date::new(2021, 1, 1)
            ]
        );
        assert_eq!(zero_rate_curve.rates(), &vec![0.0, 0.01, 0.02, 0.03, 0.04]);
        assert_eq!(
            zero_rate_curve.rate_definition().day_counter(),
            DayCounter::Actual360
        );
    }

    #[test]
    fn test_forward_rate() {
        let reference_date = Date::new(2020, 1, 1);
        let dates = vec![
            Date::new(2020, 1, 1),
            Date::new(2021, 1, 1),
            Date::new(2022, 1, 1),
            Date::new(2023, 1, 1),
            Date::new(2024, 1, 1),
        ];
        let rates = vec![0.0, 0.01, 0.02, 0.03, 0.04];
        let rate_definition = RateDefinition::default();

        let zero_rate_curve = ZeroRateTermStructure::new(
            reference_date,
            dates,
            rates,
            rate_definition,
            Interpolator::Linear,
            true,
        )
        .unwrap();

        let fr = zero_rate_curve.forward_rate(
            Date::new(2021, 1, 1),
            Date::new(2022, 1, 1),
            rate_definition.compounding(),
            rate_definition.frequency(),
        );

        println!("fr: {:?}", fr);
        assert!(fr.unwrap() - 0.02972519115024655 < 0.000000001);
    }
}
