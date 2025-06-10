use crate::prelude::*;
use rustatlas::prelude::*;

pub trait MonteCarloEngine {
    fn generate_scenario(
        &self,
        event_dates: Vec<Date>,
        request: &Vec<SimulationDataRequest>,
    ) -> Result<Scenario>;

    fn generate_scenarios(
        &self,
        event_dates: Vec<Date>,
        request: &Vec<SimulationDataRequest>,
        num_scenarios: usize,
    ) -> Result<Vec<Scenario>> {
        let scenarios = (0..num_scenarios)
            .into_iter()
            .map(|_| self.generate_scenario(event_dates.clone(), request))
            .collect::<Result<Vec<Scenario>>>()?;
        Ok(scenarios)
    }
}
