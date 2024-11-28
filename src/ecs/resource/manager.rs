use super::{Resource, ResourceError};
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

    pub fn insert_resource<R: Resource + 'static>(
        &mut self,
        resource: R,
    ) -> Result<(), ResourceError> {
        let type_id = TypeId::of::<R>();
        if self.resources.contains_key(&type_id) {
            return Err(ResourceError::AlreadyExists(
                std::any::type_name::<R>().to_string(),
            ));
        }
        self.resources.insert(type_id, Box::new(resource));
        Ok(())
    }

    pub fn get_resource<R: Resource + 'static>(&self) -> Result<&R, ResourceError> {
        let type_id = TypeId::of::<R>();
        let resource = self
            .resources
            .get(&type_id)
            .ok_or_else(|| ResourceError::NotFound(std::any::type_name::<R>().to_string()))?;

        resource
            .downcast_ref::<R>()
            .ok_or_else(|| ResourceError::TypeMismatch(std::any::type_name::<R>().to_string()))
    }

    pub fn get_mut<R: Resource + 'static>(&mut self) -> Result<&mut R, ResourceError> {
        let type_id = TypeId::of::<R>();
        let resource = self
            .resources
            .get_mut(&type_id)
            .ok_or_else(|| ResourceError::NotFound(std::any::type_name::<R>().to_string()))?;

        resource
            .downcast_mut::<R>()
            .ok_or_else(|| ResourceError::TypeMismatch(std::any::type_name::<R>().to_string()))
    }

    pub fn remove_resource<R: Resource + 'static>(&mut self) -> Result<R, ResourceError> {
        let type_id = TypeId::of::<R>();
        let resource = self
            .resources
            .remove(&type_id)
            .ok_or_else(|| ResourceError::NotFound(std::any::type_name::<R>().to_string()))?;

        let boxed = resource
            .downcast::<R>()
            .map_err(|_| ResourceError::TypeMismatch(std::any::type_name::<R>().to_string()))?;

        Ok(*boxed)
    }

    pub fn contains<R: Resource + 'static>(&self) -> bool {
        self.resources.contains_key(&TypeId::of::<R>())
    }

    pub fn clear(&mut self) {
        self.resources.clear();
    }
}

impl Default for ResourceManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use crate::ecs::resource::{ResourceError, ResourceManager};

    #[derive(Debug, PartialEq, Clone)]
    struct GameState {
        score: i32,
    }

    #[test]
    fn test_resource_insertion() {
        let mut manager = ResourceManager::new();
        let state = GameState { score: 100 };

        assert!(manager.insert_resource(state).is_ok());
        assert!(manager.contains::<GameState>());
    }

    #[test]
    fn test_resource_retrieval() {
        let mut manager = ResourceManager::new();
        let state = GameState { score: 100 };

        manager.insert_resource(state).unwrap();

        let retrieved = manager.get_resource::<GameState>().unwrap();
        assert_eq!(retrieved.score, 100);
    }

    #[test]
    fn test_resource_modification() {
        let mut manager = ResourceManager::new();
        let state = GameState { score: 100 };

        manager.insert_resource(state).unwrap();

        {
            let state = manager.get_mut::<GameState>().unwrap();
            state.score = 200;
        }

        let retrieved = manager.get_resource::<GameState>().unwrap();
        assert_eq!(retrieved.score, 200);
    }

    #[test]
    fn test_resource_removal() {
        let mut manager = ResourceManager::new();
        let state = GameState { score: 100 };

        manager.insert_resource(state.clone()).unwrap();
        assert!(manager.contains::<GameState>());

        let removed = manager.remove_resource::<GameState>().unwrap();
        assert_eq!(removed.score, 100);
        assert!(!manager.contains::<GameState>());
    }

    #[test]
    fn test_duplicate_resource() {
        let mut manager = ResourceManager::new();
        let state1 = GameState { score: 100 };
        let state2 = GameState { score: 200 };

        assert!(manager.insert_resource(state1).is_ok());
        assert!(matches!(
            manager.insert_resource(state2),
            Err(ResourceError::AlreadyExists(_))
        ));
    }
}
