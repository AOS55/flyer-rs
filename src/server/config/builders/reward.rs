use serde::{Deserialize, Serialize};

use crate::{resources::RewardWeights, server::config::ConfigError};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RewardWeightsBuilder;

impl Default for RewardWeightsBuilder {
    fn default() -> Self {
        RewardWeightsBuilder
    }
}

impl RewardWeightsBuilder {
    // pub fn new() -> Self {
    //     Self::default()
    // }

    pub fn build(self) -> Result<RewardWeights, ConfigError> {
        let weights = RewardWeights::default();

        Ok(weights)
    }
}
