mod settings;
mod state;

pub use settings::WorldSettings;
pub use state::{EntityId, SimulationState};

use std::any::Any;

/// Base trait for all simulation components
pub trait Component: Any + Send + Sync {
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
    fn update(&mut self, dt: f64) -> crate::utils::errors::Result<()>;
}

/// Extension trait for spatial components
pub trait SpatialComponent: Component {
    fn position(&self) -> nalgebra::Vector3<f64>;
    fn set_position(
        &mut self,
        position: nalgebra::Vector3<f64>,
    ) -> crate::utils::errors::Result<()>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ComponentType(pub &'static str);

impl ComponentType {
    pub const PHYSICS: ComponentType = ComponentType("physics");
    pub const VEHICLE: ComponentType = ComponentType("vehicle");
    pub const TERRAIN: ComponentType = ComponentType("terrain");
    pub const CAMERA: ComponentType = ComponentType("camera");
}
