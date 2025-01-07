use bevy::prelude::*;
use nalgebra::{Vector2, Vector3};
use rand::{Rng, SeedableRng};
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
    /// Stores the seed for the random number generator.
    pub seed: Option<u64>,
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
            seed: None,
        }
    }
}

impl RandomStartPosConfig {
    pub fn with_rng(mut self, rng: ChaCha8Rng) -> Self {
        // Extract and store the seed from the provided RNG
        let seed = rng.get_seed();
        self.seed = Some(seed[0] as u64); // ChaCha8Rng uses a 32-byte seed, we'll just use the first 8 bytes
        self
    }

    pub fn build(&self) -> Self {
        // Create a fresh RNG for this build using the stored seed
        let seed = self.seed.unwrap_or_else(|| rand::random());
        info!("Building RandomStartPosConfig with seed: {}", seed);

        Self {
            origin: self.origin,
            variance: self.variance,
            min_altitude: self.min_altitude,
            max_altitude: self.max_altitude,
            seed: Some(seed),
        }
    }

    pub fn generate_position(&self) -> Vector3<f64> {
        info!("Starting position generation with seed: {:?}", self.seed);

        // Create a fresh RNG instance for each position generation
        let mut rng = if let Some(seed) = self.seed {
            info!("Creating new RNG with seed: {}", seed);
            ChaCha8Rng::seed_from_u64(seed)
        } else {
            warn!("No seed provided, using entropy");
            ChaCha8Rng::from_entropy()
        };

        info!("RNG created: {:?}", rng);

        // Ensure min_altitude < max_altitude
        let (min_altitude, max_altitude) = if self.min_altitude >= self.max_altitude {
            warn!(
                "Invalid altitude range: min_altitude ({}) >= max_altitude ({}). Swapping values.",
                self.min_altitude, self.max_altitude
            );
            (self.max_altitude, self.min_altitude)
        } else {
            (self.min_altitude, self.max_altitude)
        };

        // Generate random values
        let u1: f64 = rng.gen();
        let u2: f64 = rng.gen();
        info!("Generated random values: u1={}, u2={}", u1, u2);

        // Convert uniform random variables to polar coordinates
        let radius = self.variance * (-2.0 * u1.ln()).sqrt();
        let theta = 2.0 * std::f64::consts::PI * u2;

        // Convert to cartesian coordinates
        let x = self.origin.x + radius * theta.cos();
        let y = self.origin.y + radius * theta.sin();
        let z = rng.gen_range(min_altitude..max_altitude);

        let position = Vector3::new(x, y, z);
        info!("Generated position: x={}, y={}, z={}", x, y, z);
        position
    }
}
