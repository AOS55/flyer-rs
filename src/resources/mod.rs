pub mod assets;
pub mod config;
mod environment;
mod time;

pub use assets::{AssetManager, AssetType};
pub use config::{
    environment::{AtmosphereConfig, AtmosphereType, EnvironmentConfig, WindModelConfig},
    physics::PhysicsConfig,
    render::RenderConfig,
    simulation::SimulationConfig,
};
pub use environment::EnvironmentResource;
pub use time::TimeManager;

use std::any::{Any, TypeId};
use std::collections::HashMap;

pub struct ResourceManager {
    resources: HashMap<TypeId, Box<dyn Any + Send + Sync>>,
}

impl ResourceManager {
    pub fn new() -> Self {
        Self {
            resources: HashMap::new(),
        }
    }

    pub fn insert<T: 'static + Send + Sync>(&mut self, resource: T) {
        self.resources.insert(TypeId::of::<T>(), Box::new(resource));
    }

    pub fn get<T: 'static>(&self) -> Option<&T> {
        self.resources
            .get(&TypeId::of::<T>())
            .and_then(|r| r.downcast_ref::<T>())
    }

    pub fn get_mut<T: 'static>(&mut self) -> Option<&mut T> {
        self.resources
            .get_mut(&TypeId::of::<T>())
            .and_then(|r| r.downcast_mut::<T>())
    }

    pub fn remove<T: 'static>(&mut self) -> Option<Box<T>> {
        self.resources
            .remove(&TypeId::of::<T>())
            .and_then(|r| r.downcast().ok())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, PartialEq)]
    struct TestResource(i32);

    #[test]
    fn test_resource_management() {
        let mut manager = ResourceManager::new();
        manager.insert(42i32);

        assert_eq!(manager.get::<i32>(), Some(&42));
        assert_eq!(manager.remove::<i32>(), Some(Box::new(42)));
        assert_eq!(manager.get::<i32>(), None);
    }

    #[test]
    fn test_resource_insertion_and_retrieval() {
        let mut manager = ResourceManager::new();
        manager.insert(TestResource(42));

        let resource = manager.get::<TestResource>();
        assert_eq!(resource, Some(&TestResource(42)));
    }

    #[test]
    fn test_resource_mutation() {
        let mut manager = ResourceManager::new();
        manager.insert(TestResource(42));

        if let Some(resource) = manager.get_mut::<TestResource>() {
            resource.0 = 84;
        }

        assert_eq!(manager.get::<TestResource>(), Some(&TestResource(84)));
    }

    #[test]
    fn test_resource_removal() {
        let mut manager = ResourceManager::new();
        manager.insert(TestResource(42));

        let removed = manager.remove::<TestResource>();
        assert_eq!(removed, Some(Box::new(TestResource(42))));
        assert_eq!(manager.get::<TestResource>(), None);
    }

    #[test]
    fn test_nonexistent_resource() {
        let manager = ResourceManager::new();
        assert_eq!(manager.get::<TestResource>(), None);
    }
}
