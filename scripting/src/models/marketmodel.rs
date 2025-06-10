use crate::prelude::*;
use rustatlas::prelude::*;

pub trait FxModel {
    fn simulate_fx(&self, request: &ExchangeRateRequest) -> Result<NumericType>;
}

pub trait InterestRateModel {
    fn simulate_df(&self, request: &DiscountFactorRequest) -> Result<NumericType>;
    // fn simulate_fwd(&self, request: &ForwardRateRequest) -> Result<NumericType>;
}

pub trait EquityModel {
    fn simulate_equity(&self, request: &EquityRequest) -> Result<NumericType>;
}

pub trait NumerarieModel {
    fn simulate_numerarie(&self, date: Date) -> Result<NumericType>;
}

pub trait MarketModel: FxModel + InterestRateModel + EquityModel + NumerarieModel {}
