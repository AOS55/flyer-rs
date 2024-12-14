use nalgebra::Vector2;
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RandomStartPosConfig {
    pub origin: Vector2<f64>,
    pub variance: f64,
    pub min_altitude: f64,
    pub max_altitude: f64,
    #[serde(skip, default = "create_rng")]
    pub rng: ChaCha8Rng,
}

fn create_rng() -> ChaCha8Rng {
    ChaCha8Rng::from_entropy()
}

impl Default for RandomStartPosConfig {
    fn default() -> Self {
        Self {
            origin: Vector2::new(0.0, 0.0),
            variance: 1000.0,
            min_altitude: -300.0,
            max_altitude: -1000.0,
            rng: ChaCha8Rng::from_entropy(),
        }
    }
}
