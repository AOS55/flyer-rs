pub mod aerodynamics;
mod agent;
mod camera;
mod collisions;
mod controller;
mod dubins;
pub mod physics;
mod propulsion;
mod render;
mod termination;
pub mod terrain;
pub mod trim;

pub use aerodynamics::{
    aero_force_system, air_data_system, calculate_aerodynamic_forces_moments, calculate_air_data,
    AirDataValues,
};
pub use agent::{
    calculate_reward, collect_state, determine_terminated, handle_render_response,
    handle_reset_response, render_frame, reset_env, running_physics, sending_response,
    waiting_for_action,
};
pub use camera::camera_follow_system;
pub use collisions::{collision_detection_system, get_terrain_at_position, TerrainInfo};
pub use controller::{dubins_gym_control_system, dubins_keyboard_system};
pub use dubins::dubins_aircraft_system;
pub use physics::{
    calculate_net_forces_moments, force_calculator_system, physics_integrator_system,
};
pub use propulsion::{
    calculate_engine_outputs, propulsion_system, update_powerplant_state, EngineOutputs,
};
pub use render::{aircraft_render_system, spawn_aircraft_sprite, spawn_runway_sprite};
pub use terrain::{ChunkManagerPlugin, TerrainGeneratorSystem};
pub use trim::{handle_trim_requests, params_to_state_inputs, trim_aircraft_system, TrimProblem};
