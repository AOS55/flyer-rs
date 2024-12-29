use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct RewardWeights;

impl Default for RewardWeights {
    fn default() -> Self {
        RewardWeights
    }
}
