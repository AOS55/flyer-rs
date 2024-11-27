use nalgebra::Vector3;
use std::collections::HashMap;
use uuid::Uuid;

use super::{Component, ComponentType, WorldSettings};
use crate::environment::Terrain;
use crate::utils::errors::SimError;
use crate::vehicles::Vehicle;
use crate::world::systems::camera::Camera;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct EntityId(Uuid);

impl EntityId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

pub struct ComponentRegistry {
    components: HashMap<(EntityId, ComponentType), Box<dyn Component>>,
}

impl ComponentRegistry {
    pub fn new() -> Self {
        Self {
            components: HashMap::new(),
        }
    }

    pub fn add_component(
        &mut self,
        entity: EntityId,
        component_type: ComponentType,
        component: Box<dyn Component>,
    ) {
        self.components.insert((entity, component_type), component);
    }

    pub fn get_component(
        &self,
        entity: EntityId,
        component_type: ComponentType,
    ) -> Option<&dyn Component> {
        self.components
            .get(&(entity, component_type))
            .map(|c| c.as_ref())
    }

    pub fn get_component_mut(
        &mut self,
        entity: EntityId,
        component_type: ComponentType,
    ) -> Option<&mut dyn Component> {
        self.components
            .get_mut(&(entity, component_type))
            .map(|c| c.as_mut())
    }
}

/// Unified state management for the simulation
pub struct SimulationState {
    registry: ComponentRegistry,
    vehicles: Vec<Box<dyn Vehicle>>,
    terrain: Option<Terrain>,
    camera: Camera,
}

impl SimulationState {
    pub fn new(settings: &WorldSettings) -> Result<Self, SimError> {
        Ok(Self {
            registry: ComponentRegistry::new(),
            vehicles: Vec::new(),
            terrain: None,
            camera: Camera::default(),
        })
    }

    pub fn step(&mut self, dt: f64) -> Result<(), SimError> {
        // Update all components
        for component in self.registry.components.values_mut() {
            component.update(dt)?;
        }

        // Update all vehicles
        for vehicle in &mut self.vehicles {
            vehicle.update(dt)?;
        }

        // Update camera if following a vehicle
        if let Some(vehicle) = self.vehicles.first() {
            let pos = vehicle.get_state().position();
            self.camera.move_to(pos);
        }

        Ok(())
    }

    pub fn add_vehicle(&mut self, vehicle: Box<dyn Vehicle>) {
        self.vehicles.push(vehicle);
    }

    pub fn camera_position(&self) -> Vector3<f64> {
        Vector3::new(self.camera.x, self.camera.y, self.camera.z)
    }

    pub fn set_terrain(&mut self, terrain: Terrain) {
        self.terrain = Some(terrain);
    }

    pub fn create_entity() -> EntityId {
        EntityId::new()
    }

    pub fn add_component(
        &mut self,
        entity: EntityId,
        component_type: ComponentType,
        component: Box<dyn Component>,
    ) {
        self.registry
            .add_component(entity, component_type, component);
    }

    pub fn get_component(
        &self,
        entity: EntityId,
        component_type: ComponentType,
    ) -> Option<&dyn Component> {
        self.registry.get_component(entity, component_type)
    }

    pub fn get_component_mut(
        &mut self,
        entity: EntityId,
        component_type: ComponentType,
    ) -> Option<&mut dyn Component> {
        self.registry.get_component_mut(entity, component_type)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_entity_creation() {
        let entity1 = EntityId::new();
        let entity2 = EntityId::new();
        assert_ne!(entity1, entity2);
    }

    #[test]
    fn test_component_registry() {
        // Add test implementation
    }
}
