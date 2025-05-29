use serde::{Deserialize, Serialize};

use crate::{
    core::{
        meta::{ForwardRateRequest, MarketRequest},
        traits::{HasCurrency, HasDiscountCurveId, HasForecastCurveId, Registrable},
    },
    currencies::enums::Currency,
    rates::interestrate::{InterestRate, RateDefinition},
    time::date::Date,
    utils::{
        errors::{AtlasError, Result},
        num::Real,
    },
};

use super::{
    cashflow::Side,
    simplecashflow::SimpleCashflow,
    traits::{Expires, InterestAccrual, Payable, RequiresFixingRate},
};

/// # FloatingRateCoupon
/// A floating rate coupon is a cashflow that pays a floating rate of interest on a notional amount.
///
/// ## Parameters
/// * `notional` - The notional amount of the coupon
/// * `spread` - The spread over the floating rate
/// * `accrual_start_date` - The date from which the coupon accrues interest
/// * `accrual_end_date` - The date until which the coupon accrues interest
/// * `payment_date` - The date on which the coupon is paid
/// * `fixing_date` - The date from which the floating rate is observed
/// * `rate_definition` - The definition of the floating rate
/// * `discount_curve_id` - The ID of the discount curve used to calculate the present value of the coupon
/// * `forecast_curve_id` - The ID of the forecast curve used to calculate the present value of the coupon
/// * `currency` - The currency of the coupon
/// * `side` - The side of the coupon (Pay or Receive)
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct FloatingRateCoupon<T: Real> {
    notional: f64,
    spread: T,
    accrual_start_date: Date,
    accrual_end_date: Date,
    fixing_date: Option<Date>,
    rate_definition: RateDefinition,
    cashflow: SimpleCashflow<T>,
    fixing_rate: Option<T>,
    forecast_curve_id: Option<usize>,
}

impl<T: Real> FloatingRateCoupon<T> {
    pub fn new(
        notional: f64,
        spread: T,
        accrual_start_date: Date,
        accrual_end_date: Date,
        payment_date: Date,
        fixing_date: Option<Date>,
        rate_definition: RateDefinition,
        currency: Currency,
        side: Side,
    ) -> FloatingRateCoupon<T> {
        FloatingRateCoupon {
            notional,
            spread,
            fixing_rate: None,
            accrual_start_date,
            accrual_end_date,
            fixing_date,
            rate_definition,
            forecast_curve_id: None,
            cashflow: SimpleCashflow::new(payment_date, currency, side),
        }
    }

    pub fn with_discount_curve_id(self, id: usize) -> FloatingRateCoupon<T> {
        self.cashflow.with_discount_curve_id(id);
        self
    }

    pub fn with_forecast_curve_id(mut self, id: usize) -> FloatingRateCoupon<T> {
        self.forecast_curve_id = Some(id);
        self
    }

    pub fn set_discount_curve_id(&mut self, id: usize) {
        self.cashflow.set_discount_curve_id(id);
    }

    pub fn set_forecast_curve_id(&mut self, id: usize) {
        self.forecast_curve_id = Some(id);
    }

    pub fn set_spread(&mut self, spread: T) {
        self.spread = spread;
        // if fixing rate is set, update the cashflow
        match self.fixing_rate {
            Some(fixing_rate) => {
                self.set_fixing_rate(fixing_rate);
            }
            None => {}
        }
    }

    pub fn set_notional(&mut self, notional: f64) {
        self.notional = notional;
    }

    pub fn notional(&self) -> f64 {
        self.notional
    }

    pub fn spread(&self) -> T {
        self.spread
    }

    pub fn rate_definition(&self) -> RateDefinition {
        self.rate_definition
    }

    pub fn fixing_date(&self) -> Date {
        match self.fixing_date {
            Some(date) => date,
            None => self.accrual_start_date,
        }
    }

    pub fn fixing_rate(&self) -> Option<T> {
        self.fixing_rate
    }
}

impl<T: Real> InterestAccrual<T> for FloatingRateCoupon<T> {
    fn accrual_start_date(&self) -> Result<Date> {
        return Ok(self.accrual_start_date);
    }
    fn accrual_end_date(&self) -> Result<Date> {
        return Ok(self.accrual_end_date);
    }
    fn accrued_amount(&self, start_date: Date, end_date: Date) -> Result<T> {
        let fixing = self
            .fixing_rate
            .ok_or(AtlasError::ValueNotSetErr("Fixing rate".to_string()))?;
        let rate = InterestRate::from_rate_definition(fixing + self.spread, self.rate_definition);

        let (d1, d2) = self.relevant_accrual_dates(self.accrual_start_date, end_date)?;
        let acc_1 = (rate.compound_factor(d1, d2) - 1.0) * self.notional;

        let (d1, d2) = self.relevant_accrual_dates(self.accrual_start_date, start_date)?;
        let acc_2 = (rate.compound_factor(d1, d2) - 1.0) * self.notional;

        return Ok(acc_1 - acc_2);
    }
}

impl<T: Real> RequiresFixingRate<T> for FloatingRateCoupon<T> {
    fn set_fixing_rate(&mut self, fixing_rate: T) {
        self.fixing_rate = Some(fixing_rate);
        let accrual = self
            .accrued_amount(self.accrual_start_date, self.accrual_end_date)
            .unwrap();
        self.cashflow = self.cashflow.with_amount(accrual);
    }
}

impl<T: Real> Payable<T> for FloatingRateCoupon<T> {
    fn amount(&self) -> Result<T> {
        return self.cashflow.amount();
    }
    fn side(&self) -> Side {
        return self.cashflow.side();
    }
    fn payment_date(&self) -> Date {
        return self.cashflow.payment_date();
    }
}

impl<T: Real> HasCurrency for FloatingRateCoupon<T> {
    fn currency(&self) -> Result<Currency> {
        self.cashflow.currency()
    }
}

impl<T: Real> HasDiscountCurveId for FloatingRateCoupon<T> {
    fn discount_curve_id(&self) -> Result<usize> {
        self.cashflow.discount_curve_id()
    }
}

impl<T: Real> HasForecastCurveId for FloatingRateCoupon<T> {
    fn forecast_curve_id(&self) -> Result<usize> {
        self.forecast_curve_id
            .ok_or(AtlasError::ValueNotSetErr("Forecast curve id".to_string()))
    }
}

impl<T: Real> Registrable for FloatingRateCoupon<T> {
    fn id(&self) -> Result<usize> {
        self.cashflow.id()
    }

    fn set_id(&mut self, id: usize) {
        self.cashflow.set_id(id);
    }

    fn market_request(&self) -> Result<MarketRequest> {
        let tmp = self.cashflow.market_request()?;
        let forecast_curve_id = self.forecast_curve_id()?;
        let fixing_date = self.fixing_date();
        let forecast = ForwardRateRequest::new(
            forecast_curve_id,
            fixing_date,
            self.accrual_start_date,
            self.accrual_end_date,
            self.rate_definition.compounding(),
            self.rate_definition.frequency(),
        );
        Ok(MarketRequest::new(
            tmp.id(),
            tmp.df(),
            Some(forecast),
            tmp.fx(),
        ))
    }
}

impl<T: Real> Expires for FloatingRateCoupon<T> {
    fn is_expired(&self, date: Date) -> bool {
        self.cashflow.payment_date() < date
    }
}
