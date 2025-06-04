use crate::{prelude::ScriptingError, utils::errors::Result};
use rustatlas::prelude::*;
#[derive(Debug, Clone, PartialEq, Default)]
pub struct SimulationData {
    numerarie: NumericType,
    dfs: Vec<NumericType>,
    fwds: Vec<NumericType>,
    fxs: Vec<NumericType>,
    equities: Vec<NumericType>,
}

impl SimulationData {
    pub fn new(
        numerarie: NumericType,
        dfs: Vec<NumericType>,
        fwds: Vec<NumericType>,
        fxs: Vec<NumericType>,
        equities: Vec<NumericType>,
    ) -> SimulationData {
        SimulationData {
            numerarie,
            dfs,
            fwds,
            fxs,
            equities,
        }
    }

    pub fn numerarie(&self) -> NumericType {
        self.numerarie
    }

    pub fn dfs(&self) -> &Vec<NumericType> {
        &self.dfs
    }

    pub fn fwds(&self) -> &Vec<NumericType> {
        &self.fwds
    }

    pub fn fxs(&self) -> &Vec<NumericType> {
        &self.fxs
    }

    pub fn equities(&self) -> &Vec<NumericType> {
        &self.equities
    }

    pub fn get_df(&self, index: usize) -> Result<NumericType> {
        self.dfs
            .get(index)
            .cloned()
            .ok_or(ScriptingError::NotFoundError(format!(
                "df at index {}",
                index
            )))
    }

    pub fn get_fwd(&self, index: usize) -> Result<NumericType> {
        self.fwds
            .get(index)
            .cloned()
            .ok_or(ScriptingError::NotFoundError(format!(
                "fwd at index {}",
                index
            )))
    }
    pub fn get_fx(&self, index: usize) -> Result<NumericType> {
        self.fxs
            .get(index)
            .cloned()
            .ok_or(ScriptingError::NotFoundError(format!(
                "fx at index {}",
                index
            )))
    }
    pub fn get_equity(&self, index: usize) -> Result<NumericType> {
        self.equities
            .get(index)
            .cloned()
            .ok_or(ScriptingError::NotFoundError(format!(
                "equity at index {}",
                index
            )))
    }
}

pub type Scenario = Vec<SimulationData>;
