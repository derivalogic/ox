use std::cell::RefCell;

use super::traits::{HasCashflows, Visit};
use crate::math::ad::genericnumber::Real;
use crate::{
    core::{meta::MarketRequest, traits::Registrable},
    utils::errors::Result,
};
/// # IndexingVisitor
/// IndexingVisitor is a visitor that registers the cashflows of an instrument
/// and generates a vector of market requests.
pub struct IndexingVisitor<T: Real> {
    request: RefCell<Vec<MarketRequest>>,
    phatom: std::marker::PhantomData<T>,
}

impl<T: Real> IndexingVisitor<T> {
    pub fn new() -> Self {
        IndexingVisitor {
            request: RefCell::new(Vec::new()),
            phatom: std::marker::PhantomData,
        }
    }

    pub fn request(&self) -> Vec<MarketRequest> {
        self.request.borrow().clone()
    }
}

impl<R: Real, T: HasCashflows<R>> Visit<T> for IndexingVisitor<R> {
    type Output = Result<()>;
    fn visit(&self, has_cashflows: &mut T) -> Self::Output {
        let mut requests = self.request.borrow_mut();
        has_cashflows
            .mut_cashflows()
            .iter_mut()
            .try_for_each(|cf| -> Result<()> {
                cf.set_id(requests.len());
                let request = cf.market_request()?;
                requests.push(request);
                Ok(())
            })?;
        Ok(())
    }
}
