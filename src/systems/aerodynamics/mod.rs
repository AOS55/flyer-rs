mod aerso_adapter;
mod air_data;
mod force_calculator;

use crate::components::AerodynamicsComponent;
use crate::ecs::{Result, System, World};
pub use air_data::AirDataSystem;
pub use force_calculator::AeroForceSystem;

pub struct AerodynamicsSystemGroup {
    air_data: AirDataSystem,
    force_calculator: AeroForceSystem,
}

impl AerodynamicsSystemGroup {
    pub fn new(component: AerodynamicsComponent) -> Self {
        Self {
            air_data: AirDataSystem,
            force_calculator: AeroForceSystem::new(component),
        }
    }
}

impl System for AerodynamicsSystemGroup {
    fn name(&self) -> &str {
        "Aerodynamics System Group"
    }

    fn run(&mut self, world: &mut World) -> Result<()> {
        // Execute systems in order
        self.air_data.run(world)?;
        self.force_calculator.run(world)?;
        Ok(())
    }

    fn dependencies(&self) -> Vec<&str> {
        // This system group should run after physics integration
        // but before the next physics step
        vec!["Physics System"]
    }
}
