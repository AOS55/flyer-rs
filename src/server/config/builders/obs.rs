use serde::{Deserialize, Serialize};

use crate::server::{config::ConfigError, obs::ContinuousObservationSpace, ObservationSpace};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObservationSpaceBuilder {
    obs_space: Option<ObservationSpace>,
}

impl Default for ObservationSpaceBuilder {
    fn default() -> Self {
        Self {
            obs_space: Some(ObservationSpace::Continuous(
                ContinuousObservationSpace::DubinsAircraft,
            )),
        }
    }
}

impl ObservationSpaceBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn obs_space(mut self, obs_type: ObservationSpace) -> Self {
        self.obs_space = Some(obs_type);
        self
    }

    pub fn build(self) -> Result<ObservationSpace, ConfigError> {
        self.obs_space.ok_or(ConfigError::MissingObservationSpace)
    }
}
