use bevy::prelude::*;
use nalgebra::{Vector2, Vector3};
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;
use serde::{Deserialize, Serialize};
use std::f64::consts::PI;

/// Enum for StartConfigurations
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum StartConfig {
    /// Random start configuration
    Random(RandomStartConfig),
    /// Fixed start configuration
    Fixed(FixedStartConfig),
}

impl Default for StartConfig {
    fn default() -> Self {
        Self::Random(RandomStartConfig::default())
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct FixedStartConfig {
    // Position configuration
    pub position: Vector3<f64>,
    // Speed configuration
    pub speed: f64,
    // Heading configuration
    pub heading: f64,
}

/// Configuration for generating random starting conditions.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct RandomStartConfig {
    // Position configuration
    pub position: RandomPosConfig,
    // Speed configuration
    pub speed: RandomSpeedConfig,
    // Heading configuration
    pub heading: RandomHeadingConfig,
    // Common seed for random number generation
    pub seed: Option<u64>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct RandomSpeedConfig {
    // Minimum speed in meters per second
    pub min_speed: f64,
    // Maximum speed in meters per second
    pub max_speed: f64,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct RandomHeadingConfig {
    // Minimum heading in radians (0 is North)
    pub min_heading: f64,
    // Maximum heading in radians (2π is North)
    pub max_heading: f64,
}

/// Configuration for generating a random starting position.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct RandomPosConfig {
    // The origin of the random position in NED frame (m).
    pub origin: Vector2<f64>,
    // The variance of the random position (m).
    pub variance: f64,
    // The minimum altitude of the random position (m).
    pub min_altitude: f64,
    // The maximum altitude of the random position (m).
    pub max_altitude: f64,
}

impl Default for FixedStartConfig {
    fn default() -> Self {
        Self {
            position: Vector3::new(0.0, 0.0, -500.0),
            speed: 80.0,
            heading: 0.0,
        }
    }
}

impl Default for RandomSpeedConfig {
    fn default() -> Self {
        Self {
            min_speed: 40.0,
            max_speed: 200.0,
        }
    }
}

impl Default for RandomHeadingConfig {
    fn default() -> Self {
        Self {
            min_heading: 0.0,      // 0pi rads
            max_heading: 2.0 * PI, // 2pi rads
        }
    }
}

impl Default for RandomPosConfig {
    fn default() -> Self {
        Self {
            origin: Vector2::new(0.0, 0.0),
            variance: 1000.0,
            min_altitude: -300.0,
            max_altitude: -1000.0,
        }
    }
}

impl Default for RandomStartConfig {
    fn default() -> Self {
        Self {
            position: RandomPosConfig::default(),
            speed: RandomSpeedConfig::default(),
            heading: RandomHeadingConfig::default(),
            seed: None,
        }
    }
}

impl RandomStartConfig {
    pub fn generate(&self) -> (Vector3<f64>, f64, f64) {
        info!("Random seed: {:?}", self.seed);
        let mut rng = match self.seed {
            Some(seed) => ChaCha8Rng::seed_from_u64(seed),
            None => {
                warn!("No seed provided, using entropy");
                ChaCha8Rng::from_entropy()
            }
        };

        // Generate position
        let position = self.generate_position(&mut rng);

        // Generate speed
        let speed = self.generate_speed(&mut rng);

        // Generate heading
        let heading = self.generate_heading(&mut rng);

        (position, speed, heading)
    }

    fn generate_position(&self, rng: &mut ChaCha8Rng) -> Vector3<f64> {
        // Ensure min_altitude < max_altitude
        let (min_altitude, max_altitude) =
            if self.position.min_altitude >= self.position.max_altitude {
                warn!(
                "Invalid altitude range: min_altitude ({}) >= max_altitude ({}). Swapping values.",
                self.position.min_altitude, self.position.max_altitude
            );
                (self.position.max_altitude, self.position.min_altitude)
            } else {
                (self.position.min_altitude, self.position.max_altitude)
            };

        // Generate random values
        let u1: f64 = rng.gen();
        let u2: f64 = rng.gen();

        // Convert uniform random variables to polar coordinates
        let radius = self.position.variance * (-2.0 * u1.ln()).sqrt();
        let theta = 2.0 * PI * u2;

        // Convert to cartesian coordinates
        let x = self.position.origin.x + radius * theta.cos();
        let y = self.position.origin.y + radius * theta.sin();
        let z = rng.gen_range(min_altitude..max_altitude);

        let position = Vector3::new(x, y, z);
        position
    }

    fn generate_speed(&self, rng: &mut ChaCha8Rng) -> f64 {
        if self.speed.min_speed >= self.speed.max_speed {
            warn!(
                "Invalid speed range: min_speed ({}) >= max_speed ({}). Using min_speed.",
                self.speed.min_speed, self.speed.max_speed
            );
            return self.speed.min_speed;
        }

        let speed = rng.gen_range(self.speed.min_speed..self.speed.max_speed);
        speed
    }

    fn generate_heading(&self, rng: &mut ChaCha8Rng) -> f64 {
        // Convert min and max headings to radians and normalize within 0 to 2π
        let min_heading =
            (self.heading.min_heading.to_radians() % (2.0 * PI) + 2.0 * PI) % (2.0 * PI);
        let max_heading =
            (self.heading.max_heading.to_radians() % (2.0 * PI) + 2.0 * PI) % (2.0 * PI);

        let heading = if min_heading <= max_heading {
            rng.gen_range(min_heading..max_heading)
        } else {
            // Handle wrap-around case (e.g., min=350°, max=10° converted to radians)
            let rand_val = rng.gen_range(0.0..(2.0 * PI));
            if rand_val >= min_heading || rand_val <= max_heading {
                rand_val
            } else {
                min_heading
            }
        };

        heading
    }
}
