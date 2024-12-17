use nalgebra::Vector2;
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use serde::{Deserialize, Serialize};

/// Configuration for generating a random starting position.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RandomStartPosConfig {
    // The origin of the random position in NED frame (m).
    pub origin: Vector2<f64>,
    // The variance of the random position (m).
    pub variance: f64,
    // The minimum altitude of the random position (m).
    pub min_altitude: f64,
    // The maximum altitude of the random position (m).
    pub max_altitude: f64,
    /// Random number generator used to produce consistent random values.
    #[serde(skip, default = "create_rng")]
    pub rng: ChaCha8Rng,
}

fn create_rng() -> ChaCha8Rng {
    ChaCha8Rng::from_entropy()
}

impl Default for RandomStartPosConfig {
    // Provides a default configuration for random starting positions.
    ///
    /// # Default Values:
    /// - Origin: (0.0, 0.0)
    /// - Variance: 1000.0 meters
    /// - Minimum Altitude: -300.0 meters
    /// - Maximum Altitude: -1000.0 meters
    /// - Random Number Generator: Seeded from entropy for reproducibility.
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
