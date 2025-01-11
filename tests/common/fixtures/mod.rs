mod components;
mod physics;
mod terrain;

// Re-export test fixtures
pub use components::{
    aircraft_configs::{advanced_dubins, basic_dubins, basic_full, high_performance},
    climbing_spatial,
    fixtures::TEST_AIRCRAFT_CONFIG,
    neutral_controls, straight_level_spatial, turning_spatial,
};

pub use physics::{
    create_test_physics,
    fixtures::TEST_PHYSICS_CONFIG,
    forces::{drag_force, lift_force, side_force, thrust_force},
    moments::{pitch_moment, roll_moment, yaw_moment},
    physics_configs::{basic_config, high_fidelity_config},
};

pub use terrain::{
    biome_configs::{coastal, mountainous},
    create_test_chunk,
    fixtures::TEST_TERRAIN_CONFIG,
    noise_configs::{detail_noise, mountain_noise, plains_noise},
    utils::{create_bush, create_rock, create_tree},
};
