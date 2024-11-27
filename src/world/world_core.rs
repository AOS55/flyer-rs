use crate::vehicles::traits::Vehicle;
use crate::world::state::WorldState;
use crate::world::SimWorld;
use std::path::PathBuf;

impl SimWorld {
    pub fn step(&mut self, dt: f64) {
        for vehicle in &mut self.state.vehicles {
            vehicle.update_state(dt);
        }

        if let Some(vehicle) = self.state.vehicles.first() {
            let pos = vehicle.get_state().position();
            self.state.camera.move_to(pos);
            self.state.log_position(pos);
        }
    }

    pub fn reset(&mut self) {
        self.state = WorldState::new(self.state.screen_dimensions, self.state.scale);
        self.create_map();
    }

    pub fn add_vehicle(&mut self, vehicle: Box<dyn Vehicle>) {
        self.state.vehicles.push(vehicle);
    }

    pub fn update_vehicle(&mut self, vehicle: Box<dyn Vehicle>, id: usize) {
        if id < self.state.vehicles.len() {
            self.state.vehicles[id] = vehicle;
        }
    }

    pub fn set_assets_dir(&mut self, path: PathBuf) {
        self.assets_dir = path;
    }

    pub fn set_terrain_data_dir(&mut self, path: PathBuf) {
        self.terrain_data_dir = path;
    }

    pub fn update_settings(
        &mut self,
        simulation_frequency: Option<f64>,
        policy_frequency: Option<f64>,
        render_frequency: Option<f64>,
    ) {
        if let Some(freq) = simulation_frequency {
            self.settings.simulation_frequency = freq;
        }
        if let Some(freq) = policy_frequency {
            self.settings.policy_frequency = freq;
        }
        if let Some(freq) = render_frequency {
            self.settings.render_frequency = freq;
        }
    }
}
