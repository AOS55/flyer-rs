pub const GRAVITY: f64 = 9.80665; // m/s^2
pub const AIR_GAS_CONSTANT: f64 = 287.05287; // J/(kg·K)
pub const ISA_SEA_LEVEL_TEMP: f64 = 288.15; // K
pub const ISA_SEA_LEVEL_PRESSURE: f64 = 101325.0; // Pa
pub const ISA_LAPSE_RATE: f64 = -0.0065; // K/m

pub const MAX_TIMESTEP: f64 = 1.0 / 30.0; // Maximum physics timestep
pub const MIN_TIMESTEP: f64 = 1.0 / 1000.0; // Minimum physics timestep

// Physical limits
pub const MAX_LOAD_FACTOR: f64 = 10.0; // Maximum load factor
pub const MIN_LOAD_FACTOR: f64 = -5.0; // Minimum load factor
pub const MAX_ANGLE_OF_ATTACK: f64 = 20.0; // Maximum angle of attack (degrees)
pub const MAX_SIDESLIP: f64 = 15.0; // Maximum sideslip angle (degrees)
