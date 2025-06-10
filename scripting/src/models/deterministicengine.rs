use crate::prelude::*;
use rustatlas::prelude::*;

pub trait DeterministicEngine {
    fn generate_scenario(
        &self,
        event_dates: Vec<Date>,
        request: &Vec<SimulationDataRequest>,
    ) -> Result<Scenario>;
}
