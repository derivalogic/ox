use serde::{Deserialize, Serialize};

use crate::{
    time::{date::Date, daycounter::DayCounter, enums::Frequency},
    utils::{
        errors::{AtlasError, Result},
        num::Real,
    },
};

use super::enums::Compounding;

/// # RateDefinition
/// Struct that defines a rate.
/// # Example
/// ```
/// use rustatlas::prelude::*;
/// let rate_definition = RateDefinition::new(DayCounter::Actual360, Compounding::Simple, Frequency::Annual);
/// assert_eq!(rate_definition.compounding(), Compounding::Simple);
/// assert_eq!(rate_definition.frequency(), Frequency::Annual);
/// assert_eq!(rate_definition.day_counter(), DayCounter::Actual360);
/// ```
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RateDefinition {
    day_counter: DayCounter,
    compounding: Compounding,
    frequency: Frequency,
}

impl RateDefinition {
    pub fn new(
        day_counter: DayCounter,
        compounding: Compounding,
        frequency: Frequency,
    ) -> RateDefinition {
        RateDefinition {
            day_counter,
            compounding,
            frequency,
        }
    }

    pub fn compounding(&self) -> Compounding {
        return self.compounding;
    }

    pub fn frequency(&self) -> Frequency {
        return self.frequency;
    }

    pub fn day_counter(&self) -> DayCounter {
        return self.day_counter;
    }
}

impl Default for RateDefinition {
    fn default() -> Self {
        RateDefinition::new(
            DayCounter::Actual360,
            Compounding::Simple,
            Frequency::Annual,
        )
    }
}

/// # InterestRate
/// Struct that defines an interest rate.
///
/// ## Example
/// ```
/// use rustatlas::prelude::*;
/// let rate = InterestRate::new(0.05, Compounding::Simple, Frequency::Annual, DayCounter::Actual360);
/// assert_eq!(rate.rate(), 0.05);
/// assert_eq!(rate.compounding(), Compounding::Simple);
/// assert_eq!(rate.frequency(), Frequency::Annual);
/// assert_eq!(rate.day_counter(), DayCounter::Actual360);
/// ```
///
/// Using the AD variable type:
/// ```
/// use rustatlas::prelude::*;
/// use rustatlas::math::ad::Var;
/// let rate = InterestRate::<Var>::new(
///     Var::from(0.05),
///     Compounding::Simple,
///     Frequency::Annual,
///     DayCounter::Actual360,
/// );
/// assert_eq!(rate.rate().value(), 0.05);
/// ```
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
pub struct InterestRate<T: Real = f64> {
    rate: T,
    rate_definition: RateDefinition,
}

impl<T: Real> InterestRate<T> {
    pub fn new(
        rate: T,
        compounding: Compounding,
        frequency: Frequency,
        day_counter: DayCounter,
    ) -> InterestRate<T> {
        InterestRate {
            rate,
            rate_definition: RateDefinition::new(day_counter, compounding, frequency),
        }
    }

    pub fn from_rate_definition(rate: T, rate_definition: RateDefinition) -> InterestRate<T> {
        InterestRate {
            rate,
            rate_definition,
        }
    }

    pub fn rate(&self) -> T {
        return self.rate;
    }

    pub fn rate_definition(&self) -> RateDefinition {
        return self.rate_definition;
    }

    pub fn compounding(&self) -> Compounding {
        return self.rate_definition.compounding();
    }

    pub fn frequency(&self) -> Frequency {
        return self.rate_definition.frequency();
    }

    pub fn day_counter(&self) -> DayCounter {
        return self.rate_definition.day_counter();
    }

    pub fn implied_rate(
        compound: T,
        result_dc: DayCounter,
        comp: Compounding,
        freq: Frequency,
        t: T,
    ) -> Result<InterestRate<T>> {
        if compound <= T::from(0.0) {
            return Err(AtlasError::InvalidValueErr(
                "Positive compound factor required".to_string(),
            ));
        }
        let r: T;
        let f = T::from(freq as i64 as f64);
        if compound == T::from(1.0) {
            if t < T::from(0.0) {
                return Err(AtlasError::InvalidValueErr(
                    "Non-negative time required".to_string(),
                ));
            }
            r = T::from(0.0);
        } else {
            if t <= T::from(0.0) {
                return Err(AtlasError::InvalidValueErr(
                    "Positive time required".to_string(),
                ));
            }
            match comp {
                Compounding::Simple => r = (compound - T::from(1.0)) / t,
                Compounding::Compounded => {
                    r = (compound.powf(T::from(1.0) / (f * t)) - T::from(1.0)) * f
                }

                Compounding::Continuous => r = compound.ln() / t,
                Compounding::SimpleThenCompounded => {
                    if t <= T::from(1.0) / f {
                        r = (compound - T::from(1.0)) / t
                    } else {
                        r = (compound.powf(T::from(1.0) / (f * t)) - T::from(1.0)) * f
                    }
                }
                Compounding::CompoundedThenSimple => {
                    if t > T::from(1.0) / f {
                        r = (compound - T::from(1.0)) / t
                    } else {
                        r = (compound.powf(T::from(1.0) / (f * t)) - T::from(1.0)) * f
                    }
                }
            }
        }
        return Ok(InterestRate::new(r, comp, freq, result_dc));
    }

    pub fn compound_factor(&self, start: Date, end: Date) -> T {
        let day_counter = self.day_counter();
        let year_fraction = day_counter.year_fraction::<T>(start, end);
        return self.compound_factor_from_yf(year_fraction);
    }

    pub fn compound_factor_from_yf(&self, year_fraction: T) -> T {
        let rate = self.rate();
        let compounding = self.compounding();
        let f = self.frequency() as i64 as f64;
        match compounding {
            Compounding::Simple => rate * year_fraction + T::from(1.0),
            Compounding::Compounded => (T::from(1.0) + rate / f).powf(year_fraction * f),

            Compounding::Continuous => (rate * year_fraction).exp(),
            Compounding::SimpleThenCompounded => {
                if year_fraction <= T::from(1.0) / f {
                    rate * year_fraction + T::from(1.0)
                } else {
                    (T::from(1.0) + rate / f).powf(year_fraction * f)
                }
            }
            Compounding::CompoundedThenSimple => {
                if year_fraction > T::from(1.0) / f {
                    T::from(1.0) + rate * year_fraction
                } else {
                    (T::from(1.0) + rate / f).powf(year_fraction * f)
                }
            }
        }
    }

    pub fn discount_factor(&self, start: Date, end: Date) -> T {
        return T::from(1.0) / self.compound_factor(start, end);
    }

    pub fn forward_rate(
        &self,
        _start_date: Date,
        _end_date: Date,
        _comp: Compounding,
        _freq: Frequency,
    ) -> T {
        return self.rate;
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        rates::{
            enums::Compounding,
            interestrate::{InterestRate, RateDefinition},
        },
        time::{daycounter::DayCounter, enums::Frequency},
    };

    struct InterestRateData {
        rate: f64,
        compounding: Compounding,
        frequency: Frequency,
        time: f64,
        compounding2: Compounding,
        frequency2: Frequency,
        rate2: f64,
        precision: i64,
    }
    fn test_cases() -> Vec<InterestRateData> {
        let test_cases = vec![
            InterestRateData {
                rate: 0.0800,
                compounding: Compounding::Compounded,
                frequency: Frequency::Quarterly,
                time: 1.00,
                compounding2: Compounding::Continuous,
                frequency2: Frequency::Annual,
                rate2: 0.0792,
                precision: 4,
            },
            InterestRateData {
                rate: 0.1200,
                compounding: Compounding::Continuous,
                frequency: Frequency::Annual,
                time: 1.00,
                compounding2: Compounding::Compounded,
                frequency2: Frequency::Annual,
                rate2: 0.1275,
                precision: 4,
            },
            InterestRateData {
                rate: 0.0800,
                compounding: Compounding::Compounded,
                frequency: Frequency::Quarterly,
                time: 1.00,
                compounding2: Compounding::Compounded,
                frequency2: Frequency::Annual,
                rate2: 0.0824,
                precision: 4,
            },
            InterestRateData {
                rate: 0.0700,
                compounding: Compounding::Compounded,
                frequency: Frequency::Quarterly,
                time: 1.00,
                compounding2: Compounding::Compounded,
                frequency2: Frequency::Semiannual,
                rate2: 0.0706,
                precision: 4,
            },
            InterestRateData {
                rate: 0.0100,
                compounding: Compounding::Compounded,
                frequency: Frequency::Annual,
                time: 1.00,
                compounding2: Compounding::Simple,
                frequency2: Frequency::Annual,
                rate2: 0.0100,
                precision: 4,
            },
            InterestRateData {
                rate: 0.0200,
                compounding: Compounding::Simple,
                frequency: Frequency::Annual,
                time: 1.00,
                compounding2: Compounding::Compounded,
                frequency2: Frequency::Annual,
                rate2: 0.0200,
                precision: 4,
            },
            InterestRateData {
                rate: 0.0300,
                compounding: Compounding::Compounded,
                frequency: Frequency::Semiannual,
                time: 0.50,
                compounding2: Compounding::Simple,
                frequency2: Frequency::Annual,
                rate2: 0.0300,
                precision: 4,
            },
            InterestRateData {
                rate: 0.0400,
                compounding: Compounding::Simple,
                frequency: Frequency::Annual,
                time: 0.50,
                compounding2: Compounding::Compounded,
                frequency2: Frequency::Semiannual,
                rate2: 0.0400,
                precision: 4,
            },
            InterestRateData {
                rate: 0.0500,
                compounding: Compounding::Compounded,
                frequency: Frequency::EveryFourthMonth,
                time: 1.0 / 3.0,
                compounding2: Compounding::Simple,
                frequency2: Frequency::Annual,
                rate2: 0.0500,
                precision: 4,
            },
            InterestRateData {
                rate: 0.0600,
                compounding: Compounding::Simple,
                frequency: Frequency::Annual,
                time: 1.0 / 3.0,
                compounding2: Compounding::Compounded,
                frequency2: Frequency::EveryFourthMonth,
                rate2: 0.0600,
                precision: 4,
            },
            InterestRateData {
                rate: 0.0500,
                compounding: Compounding::Compounded,
                frequency: Frequency::Quarterly,
                time: 0.25,
                compounding2: Compounding::Simple,
                frequency2: Frequency::Annual,
                rate2: 0.0500,
                precision: 4,
            },
            InterestRateData {
                rate: 0.0600,
                compounding: Compounding::Simple,
                frequency: Frequency::Annual,
                time: 0.25,
                compounding2: Compounding::Compounded,
                frequency2: Frequency::Quarterly,
                rate2: 0.0600,
                precision: 4,
            },
            InterestRateData {
                rate: 0.0700,
                compounding: Compounding::Compounded,
                frequency: Frequency::Bimonthly,
                time: 1.0 / 6.0,
                compounding2: Compounding::Simple,
                frequency2: Frequency::Annual,
                rate2: 0.0700,
                precision: 4,
            },
            InterestRateData {
                rate: 0.0800,
                compounding: Compounding::Simple,
                frequency: Frequency::Annual,
                time: 1.0 / 6.0,
                compounding2: Compounding::Compounded,
                frequency2: Frequency::Bimonthly,
                rate2: 0.0800,
                precision: 4,
            },
            InterestRateData {
                rate: 0.0900,
                compounding: Compounding::Compounded,
                frequency: Frequency::Monthly,
                time: 1.0 / 12.0,
                compounding2: Compounding::Simple,
                frequency2: Frequency::Annual,
                rate2: 0.0900,
                precision: 4,
            },
            InterestRateData {
                rate: 0.1000,
                compounding: Compounding::Simple,
                frequency: Frequency::Annual,
                time: 1.0 / 12.0,
                compounding2: Compounding::Compounded,
                frequency2: Frequency::Monthly,
                rate2: 0.1000,
                precision: 4,
            },
            InterestRateData {
                rate: 0.0300,
                compounding: Compounding::SimpleThenCompounded,
                frequency: Frequency::Semiannual,
                time: 0.25,
                compounding2: Compounding::Simple,
                frequency2: Frequency::Annual,
                rate2: 0.0300,
                precision: 4,
            },
            InterestRateData {
                rate: 0.0300,
                compounding: Compounding::SimpleThenCompounded,
                frequency: Frequency::Semiannual,
                time: 0.25,
                compounding2: Compounding::Simple,
                frequency2: Frequency::Semiannual,
                rate2: 0.0300,
                precision: 4,
            },
            InterestRateData {
                rate: 0.0300,
                compounding: Compounding::SimpleThenCompounded,
                frequency: Frequency::Semiannual,
                time: 0.25,
                compounding2: Compounding::Simple,
                frequency2: Frequency::Quarterly,
                rate2: 0.0300,
                precision: 4,
            },
            InterestRateData {
                rate: 0.0300,
                compounding: Compounding::SimpleThenCompounded,
                frequency: Frequency::Semiannual,
                time: 0.50,
                compounding2: Compounding::Simple,
                frequency2: Frequency::Annual,
                rate2: 0.0300,
                precision: 4,
            },
            InterestRateData {
                rate: 0.0300,
                compounding: Compounding::SimpleThenCompounded,
                frequency: Frequency::Semiannual,
                time: 0.50,
                compounding2: Compounding::Simple,
                frequency2: Frequency::Semiannual,
                rate2: 0.0300,
                precision: 4,
            },
            InterestRateData {
                rate: 0.0300,
                compounding: Compounding::SimpleThenCompounded,
                frequency: Frequency::Semiannual,
                time: 0.75,
                compounding2: Compounding::Compounded,
                frequency2: Frequency::Semiannual,
                rate2: 0.0300,
                precision: 4,
            },
            InterestRateData {
                rate: 0.0400,
                compounding: Compounding::Simple,
                frequency: Frequency::Semiannual,
                time: 0.25,
                compounding2: Compounding::SimpleThenCompounded,
                frequency2: Frequency::Quarterly,
                rate2: 0.0400,
                precision: 4,
            },
            InterestRateData {
                rate: 0.0400,
                compounding: Compounding::Simple,
                frequency: Frequency::Semiannual,
                time: 0.25,
                compounding2: Compounding::SimpleThenCompounded,
                frequency2: Frequency::Semiannual,
                rate2: 0.0400,
                precision: 4,
            },
            InterestRateData {
                rate: 0.0400,
                compounding: Compounding::Simple,
                frequency: Frequency::Semiannual,
                time: 0.25,
                compounding2: Compounding::SimpleThenCompounded,
                frequency2: Frequency::Annual,
                rate2: 0.0400,
                precision: 4,
            },
            InterestRateData {
                rate: 0.0400,
                compounding: Compounding::Compounded,
                frequency: Frequency::Quarterly,
                time: 0.50,
                compounding2: Compounding::SimpleThenCompounded,
                frequency2: Frequency::Quarterly,
                rate2: 0.0400,
                precision: 4,
            },
            InterestRateData {
                rate: 0.0400,
                compounding: Compounding::Simple,
                frequency: Frequency::Semiannual,
                time: 0.50,
                compounding2: Compounding::SimpleThenCompounded,
                frequency2: Frequency::Semiannual,
                rate2: 0.0400,
                precision: 4,
            },
            InterestRateData {
                rate: 0.0400,
                compounding: Compounding::Simple,
                frequency: Frequency::Semiannual,
                time: 0.50,
                compounding2: Compounding::SimpleThenCompounded,
                frequency2: Frequency::Annual,
                rate2: 0.0400,
                precision: 4,
            },
            InterestRateData {
                rate: 0.0400,
                compounding: Compounding::Compounded,
                frequency: Frequency::Quarterly,
                time: 0.75,
                compounding2: Compounding::SimpleThenCompounded,
                frequency2: Frequency::Quarterly,
                rate2: 0.0400,
                precision: 4,
            },
            InterestRateData {
                rate: 0.0400,
                compounding: Compounding::Compounded,
                frequency: Frequency::Semiannual,
                time: 0.75,
                compounding2: Compounding::SimpleThenCompounded,
                frequency2: Frequency::Semiannual,
                rate2: 0.0400,
                precision: 4,
            },
        ];
        return test_cases;
    }
    #[test]
    fn test_all_cases() {
        let test_cases = test_cases();
        for test_case in test_cases {
            let rate = InterestRate::new(
                test_case.rate,
                test_case.compounding,
                test_case.frequency,
                DayCounter::Actual360,
            );
            let implied_rate = InterestRate::implied_rate(
                rate.compound_factor_from_yf(test_case.time),
                DayCounter::Actual360,
                test_case.compounding2,
                test_case.frequency2,
                test_case.time,
            )
            .unwrap();
            assert!(
                (implied_rate.rate() - test_case.rate2).abs()
                    < (test_case.precision as f64) / 100.0
            );
        }
    }

    #[test]
    fn test_rate_definition_new() {
        let rd = RateDefinition::new(
            DayCounter::Actual360,
            Compounding::Simple,
            Frequency::Annual,
        );
        assert_eq!(rd.compounding(), Compounding::Simple);
        assert_eq!(rd.frequency(), Frequency::Annual);
        assert_eq!(rd.day_counter(), DayCounter::Actual360);
    }

    #[test]
    fn test_rate_definition_common_definition() {
        let rd = RateDefinition::default();
        assert_eq!(rd.compounding(), Compounding::Simple);
        assert_eq!(rd.frequency(), Frequency::Annual);
        assert_eq!(rd.day_counter(), DayCounter::Actual360);
    }

    #[test]
    fn test_interest_rate_new() {
        let ir = InterestRate::new(
            0.05,
            Compounding::Simple,
            Frequency::Annual,
            DayCounter::Actual360,
        );
        assert_eq!(ir.rate(), 0.05);
        assert_eq!(ir.compounding(), Compounding::Simple);
        assert_eq!(ir.frequency(), Frequency::Annual);
        assert_eq!(ir.day_counter(), DayCounter::Actual360);
    }

    #[test]
    fn test_interest_rate_from_rate_definition() {
        let rd = RateDefinition::new(
            DayCounter::Actual360,
            Compounding::Simple,
            Frequency::Annual,
        );
        let ir = InterestRate::from_rate_definition(0.05, rd);
        assert_eq!(ir.rate(), 0.05);
        assert_eq!(ir.compounding(), Compounding::Simple);
        assert_eq!(ir.frequency(), Frequency::Annual);
        assert_eq!(ir.day_counter(), DayCounter::Actual360);
    }

    const EPSILON: f64 = 1e-9; // or any other small value that you choose

    #[test]
    fn test_implied_rate() {
        // Choose parameters that make sense for your implied_rate function
        // For example:
        let ir = InterestRate::implied_rate(
            1.05,
            DayCounter::Actual360,
            Compounding::Simple,
            Frequency::Annual,
            1.0,
        )
        .unwrap();
        let expected_rate = 0.05;
        assert!((ir.rate() - expected_rate).abs() < EPSILON);
    }

    #[test]
    fn test_implied_rate_panic() {
        let err = InterestRate::implied_rate(
            0.0,
            DayCounter::Actual360,
            Compounding::Simple,
            Frequency::Annual,
            1.0,
        );
        assert!(err.is_err());
    }

    #[test]
    fn test_ad_compound_factor_simple() {
        use crate::math::ad::{backward, reset_tape, Var};

        reset_tape();
        let rate = Var::new(0.05);
        let t = Var::new(2.0);
        let ir = InterestRate::new(
            rate,
            Compounding::Simple,
            Frequency::Annual,
            DayCounter::Actual360,
        );
        let cf = ir.compound_factor_from_yf(t);
        let grad = backward(&cf);

        assert!((cf.value() - 1.1).abs() < 1e-12);
        assert!((grad[rate.id()] - 2.0).abs() < 1e-12);
        assert!((grad[t.id()] - 0.05).abs() < 1e-12);
    }

    #[test]
    fn test_ad_compound_factor_continuous() {
        use crate::math::ad::{backward, reset_tape, Var};

        reset_tape();
        let rate = Var::new(0.07);
        let t = Var::new(1.5);
        let ir = InterestRate::new(
            rate,
            Compounding::Continuous,
            Frequency::Annual,
            DayCounter::Actual360,
        );
        let cf = ir.compound_factor_from_yf(t);
        let grad = backward(&cf);

        let expected_cf = (rate.value() * t.value()).exp();
        assert!((cf.value() - expected_cf).abs() < 1e-12);
        assert!((grad[rate.id()] - t.value() * expected_cf).abs() < 1e-12);
        assert!((grad[t.id()] - rate.value() * expected_cf).abs() < 1e-12);
    }

    #[test]
    fn test_ad_discount_factor_simple_time() {
        use crate::math::ad::{backward, reset_tape, Var};
        use crate::utils::num::Real;

        reset_tape();
        let rate = Var::new(0.05);
        let t = Var::new(3.0);
        let ir = InterestRate::new(
            rate,
            Compounding::Simple,
            Frequency::Annual,
            DayCounter::Actual360,
        );
        let cf = ir.compound_factor_from_yf(t);
        let df = <Var as Real>::div_from_const(1.0, cf);
        let grad = backward(&df);

        let expected_df = 1.0 / (1.0 + rate.value() * t.value());
        assert!((df.value() - expected_df).abs() < 1e-12);
        let expected_dt = -rate.value() / (1.0 + rate.value() * t.value()).powi(2);
        assert!((grad[t.id()] - expected_dt).abs() < 1e-12);
    }

    #[test]
    fn test_ad_discount_factor_continuous_time() {
        use crate::math::ad::{backward, reset_tape, Var};
        use crate::utils::num::Real;

        reset_tape();
        let rate = Var::new(0.07);
        let t = Var::new(2.0);
        let ir = InterestRate::new(
            rate,
            Compounding::Continuous,
            Frequency::Annual,
            DayCounter::Actual360,
        );
        let cf = ir.compound_factor_from_yf(t);
        let df = <Var as Real>::div_from_const(1.0, cf);
        let grad = backward(&df);

        let expected_df = (-rate.value() * t.value()).exp();
        assert!((df.value() - expected_df).abs() < 1e-12);
        let expected_dt = -rate.value() * expected_df;
        assert!((grad[t.id()] - expected_dt).abs() < 1e-12);
    }
}
