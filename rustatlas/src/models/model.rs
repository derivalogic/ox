use crate::prelude::*;

/// Enumeration of available models
#[derive(Clone)]
pub enum Model<'a> {
    Simple(SimpleModel<'a>),
    BlackScholes(BlackScholesModel<'a>),
}

impl<'a> Model<'a> {
    /// Prepare the model for serial execution. Currently a no-op.
    pub fn prepare(model: Self) -> Self {
        model
    }

    /// Prepare clones of the model suitable for running in parallel.
    pub fn prepare4parallelization(&self, n: usize) -> Vec<Self> {
        (0..n)
            .map(|i| match self {
                Model::Simple(m) => Model::Simple(m.clone()),
                Model::BlackScholes(m) => Model::BlackScholes(m.clone().with_seed(i as u64)),
            })
            .collect()
    }
}

