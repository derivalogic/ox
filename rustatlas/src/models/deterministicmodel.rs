use crate::prelude::*;

/// # Deterministic Model
/// A model that provides market data based in the current market state.
pub trait DeterministicModel {
    fn reference_date(&self) -> Date;
    fn gen_df_data(&self, df: DiscountFactorRequest) -> Result<NumericType>;
    fn gen_fx_data(&self, fx: ExchangeRateRequest) -> Result<NumericType>;
    fn gen_fwd_data(&self, fwd: ForwardRateRequest) -> Result<NumericType>;
    fn gen_numerarie(&self, market_request: NumerarieRequest) -> Result<NumericType>;
    fn gen_node(&self, market_request: &MarketRequest) -> Result<MarketData> {
        let id = market_request.id();
        let df = match market_request.df() {
            Some(df) => Some(self.gen_df_data(df)?),
            None => None,
        };

        let fwd = match market_request.fwd() {
            Some(fwd) => Some(self.gen_fwd_data(fwd)?),
            None => None,
        };

        let fx = match market_request.fx() {
            Some(fx) => Some(self.gen_fx_data(fx)?),
            None => None,
        };

        let numerarie = match market_request.numerarie() {
            Some(num) => self.gen_numerarie(num)?,
            None => NumericType::new(1.0),
        };

        return Ok(MarketData::new(
            id,
            self.reference_date(),
            df,
            fwd,
            fx,
            numerarie,
        ));
    }

    fn gen_market_data(&self, market_request: &[MarketRequest]) -> Result<Vec<MarketData>> {
        market_request.iter().map(|x| self.gen_node(x)).collect()
    }
}
