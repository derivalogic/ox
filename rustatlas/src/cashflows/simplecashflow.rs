use serde::{Deserialize, Serialize};

use crate::prelude::*;

/// # SimpleCashflow
/// A simple cashflow that is payable at a given date.
///
/// ## Example
/// ```
/// use rustatlas::prelude::*;
/// let payment_date = Date::new(2020, 1, 1);
/// let cashflow = SimpleCashflow::new(payment_date, Currency::USD, Side::Receive).with_amount(100.0);
/// assert_eq!(cashflow.side(), Side::Receive);
/// assert_eq!(cashflow.payment_date(), payment_date);
/// ```
/// ```
/// use rustatlas::prelude::*;
/// use rustatlas::math::ad::Var;
/// let payment_date = Date::new(2020, 1, 1);
/// let cashflow = SimpleCashflow::new(payment_date, Currency::USD, Side::Receive).with_amount(Var::from(100.0));
/// assert_eq!(cashflow.amount().unwrap().value(), 100.0);
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct SimpleCashflow<T: GenericNumber> {
    payment_date: Date,
    currency: Currency,
    side: Side,
    amount: Option<T>,
    discount_curve_id: Option<usize>,
    id: Option<usize>,
}

impl<T: GenericNumber> SimpleCashflow<T> {
    pub fn new(payment_date: Date, currency: Currency, side: Side) -> SimpleCashflow<T> {
        SimpleCashflow {
            payment_date,
            currency,
            side,
            amount: None,
            discount_curve_id: None,
            id: None,
        }
    }

    pub fn with_amount(mut self, amount: T) -> SimpleCashflow<T> {
        self.amount = Some(amount);
        self
    }

    pub fn with_discount_curve_id(mut self, discount_curve_id: usize) -> SimpleCashflow<T> {
        self.discount_curve_id = Some(discount_curve_id);
        self
    }

    pub fn with_id(mut self, registry_id: usize) -> SimpleCashflow<T> {
        self.id = Some(registry_id);
        self
    }

    pub fn set_discount_curve_id(&mut self, id: usize) {
        self.discount_curve_id = Some(id);
    }

    pub fn set_amount(&mut self, amount: T) {
        self.amount = Some(amount);
    }
}

impl<T: GenericNumber> HasCurrency for SimpleCashflow<T> {
    fn currency(&self) -> Result<Currency> {
        return Ok(self.currency);
    }
}

impl<T: GenericNumber> HasDiscountCurveId for SimpleCashflow<T> {
    fn discount_curve_id(&self) -> Result<usize> {
        return self
            .discount_curve_id
            .ok_or(AtlasError::ValueNotSetErr("Discount curve id".to_string()));
    }
}

impl<T: GenericNumber> HasForecastCurveId for SimpleCashflow<T> {
    fn forecast_curve_id(&self) -> Result<usize> {
        return Err(AtlasError::InvalidValueErr(
            "No forecast curve id for simple cashflow".to_string(),
        ));
    }
}

impl<T: GenericNumber> Registrable for SimpleCashflow<T> {
    fn id(&self) -> Result<usize> {
        return self.id.ok_or(AtlasError::ValueNotSetErr("Id".to_string()));
    }w

    fn set_id(&mut self, id: usize) {
        self.id = Some(id);
    }

    fn market_request(&self) -> Result<MarketRequest> {
        let id = self.id()?;
        let discount_curve_id = self.discount_curve_id()?;
        let currency = self.currency()?;
        let currency_request = ExchangeRateRequest::new(currency, None, None);
        let discount_request = DiscountFactorRequest::new(discount_curve_id, self.payment_date);
        return Ok(MarketRequest::new(
            id,
            Some(discount_request),
            None,
            Some(currency_request),
        ));
    }
}

impl<T: GenericNumber> Payable<T> for SimpleCashflow<T> {
    fn amount(&self) -> Result<T> {
        return self.amount.ok_or(AtlasError::ValueNotSetErr(
            "Amount not set for simple cashflow".to_string(),
        ));
    }
    fn side(&self) -> Side {
        return self.side;
    }

    fn payment_date(&self) -> Date {
        return self.payment_date;
    }
}

impl<T: GenericNumber> Expires for SimpleCashflow<T> {
    fn is_expired(&self, date: Date) -> bool {
        return self.payment_date < date;
    }
}
