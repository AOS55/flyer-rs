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
mod trim;

pub use aerodynamics::{aero_force_system, air_data_system};
pub use agent::{
    calculate_reward, collect_state, determine_terminated, handle_render_response,
    handle_reset_response, render_frame, reset_env, running_physics, sending_response,
    waiting_for_action,
};
pub use camera::camera_follow_system;
pub use collisions::{collision_detection_system, get_terrain_at_position, TerrainInfo};
pub use controller::{dubins_gym_control_system, dubins_keyboard_system};
pub use dubins::dubins_aircraft_system;
pub use physics::{force_calculator_system, physics_integrator_system};
pub use propulsion::propulsion_system;
pub use render::{aircraft_render_system, spawn_aircraft_sprite};
pub use terrain::{ChunkManagerPlugin, TerrainGeneratorSystem};
pub use trim::{handle_trim_requests, trim_aircraft_system, VirtualPhysics};
