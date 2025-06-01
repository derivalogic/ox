use crate::prelude::*;
use std::cell::RefCell;

/// # IndexingVisitor
/// IndexingVisitor is a visitor that registers the cashflows of an instrument
/// and generates a vector of market requests.
pub struct IndexingVisitor<T: GenericNumber> {
    request: RefCell<Vec<MarketRequest>>,
    phatom: std::marker::PhantomData<T>,
}

impl<T: GenericNumber> IndexingVisitor<T> {
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

impl<R: GenericNumber, T: HasCashflows<R>> Visit<T> for IndexingVisitor<R> {
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
