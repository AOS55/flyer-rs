use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// Component representing the rendering state of an aircraft.
/// This includes its current attitude, which determines its orientation
/// for visual representation in the simulation.
#[derive(Component)]
pub struct AircraftRenderState {
    /// The current attitude (orientation) of the aircraft.
    pub attitude: Attitude,
}

/// Enum representing the possible attitudes (orientations) of an aircraft.
/// These attitudes are discrete states derived from pitch and roll angles,
/// used for rendering or state-based logic.
#[derive(Debug, Clone, Copy, PartialEq, Hash, Eq, Serialize, Deserialize)]
pub enum Attitude {
    /// Aircraft nose up and rolling to the right.
    UpRight,
    /// Aircraft level pitch, rolling to the right.
    Right,
    /// Aircraft nose down and rolling to the right.
    DownRight,
    /// Aircraft nose up (pitching upward).
    Up,
    /// Aircraft level (neutral pitch and roll).
    Level,
    /// Aircraft nose down (pitching downward).
    Down,
    /// Aircraft nose up and rolling to the left.
    UpLeft,
    /// Aircraft level pitch, rolling to the left.
    LevelLeft,
    /// Aircraft nose down and rolling to the left.
    DownLeft,
}

impl Attitude {
    /// Converts an `Attitude` enum variant into its corresponding index value.
    ///
    /// # Returns
    /// A `usize` index representing the attitude, which can be used for array-based lookups.
    ///
    /// | Attitude       | Index |
    /// |----------------|-------|
    /// | UpRight        | 0     |
    /// | Right          | 1     |
    /// | DownRight      | 2     |
    /// | Up             | 3     |
    /// | Level          | 4     |
    /// | Down           | 5     |
    /// | UpLeft         | 6     |
    /// | LevelLeft      | 7     |
    /// | DownLeft       | 8     |
    pub fn to_index(&self) -> usize {
        match self {
            Attitude::UpRight => 0,
            Attitude::Right => 1,
            Attitude::DownRight => 2,
            Attitude::Up => 3,
            Attitude::Level => 4,
            Attitude::Down => 5,
            Attitude::UpLeft => 6,
            Attitude::LevelLeft => 7,
            Attitude::DownLeft => 8,
        }
    }

    /// Determines the aircraft's `Attitude` based on pitch and roll angles.
    ///
    /// # Arguments
    /// * `pitch` - The pitch angle of the aircraft (radians).
    ///             Positive values indicate nose-up; negative values indicate nose-down.
    /// * `roll` - The roll angle of the aircraft (radians).
    ///            Positive values indicate rolling to the right; negative values indicate rolling to the left.
    ///
    /// # Returns
    /// An `Attitude` enum variant that represents the discrete orientation state of the aircraft.
    ///
    /// # Notes
    /// - `PITCH_THRESHOLD` is set to 10 degrees (π/18 radians).
    /// - `ROLL_THRESHOLD` is set to 5 degrees (π/36 radians).
    ///
    /// These thresholds define the boundaries for determining the attitude of the aircraft.
    pub fn from_angles(pitch: f64, roll: f64) -> Attitude {
        const PITCH_THRESHOLD: f64 = std::f64::consts::PI / 18.0; // 10 degrees
        const ROLL_THRESHOLD: f64 = std::f64::consts::PI / 36.0; // 5 degrees

        match (pitch, roll) {
            // Nose up attitudes
            (p, r) if p > PITCH_THRESHOLD && r < -ROLL_THRESHOLD => Attitude::UpLeft,
            (p, r) if p > PITCH_THRESHOLD && r > ROLL_THRESHOLD => Attitude::UpRight,
            (p, _) if p > PITCH_THRESHOLD => Attitude::Up,

            // Nose down attitudes
            (p, r) if p < -PITCH_THRESHOLD && r < -ROLL_THRESHOLD => Attitude::DownLeft,
            (p, r) if p < -PITCH_THRESHOLD && r > ROLL_THRESHOLD => Attitude::DownRight,
            (p, _) if p < -PITCH_THRESHOLD => Attitude::Down,

            // Level attitudes
            (_, r) if r < -ROLL_THRESHOLD => Attitude::LevelLeft,
            (_, r) if r > ROLL_THRESHOLD => Attitude::Right,
            _ => Attitude::Level,
        }
    }
}
