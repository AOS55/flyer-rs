use super::registry::ComponentRegistry;
use super::storage::VecStorage;
use crate::ecs::entity::EntityId;

pub struct ComponentManager {
    registry: ComponentRegistry,
}

impl ComponentManager {
    pub fn new() -> Self {
        Self {
            registry: ComponentRegistry::new(),
        }
    }

    pub fn register<T: 'static + Send + Sync>(&mut self) {
        self.registry
            .register::<T>(|| Box::new(VecStorage::<T>::new()));
    }

    pub fn ensure_capacity(&mut self, capacity: usize) {
        self.registry.ensure_capacity(capacity);
    }

    pub fn insert_component<T: 'static + Send + Sync>(&mut self, entity: EntityId, component: T) {
        let storage = self.registry.get_storage_mut::<T>();
        storage
            .as_any_mut()
            .downcast_mut::<VecStorage<T>>()
            .expect("Storage type mismatch")
            .insert(entity, component);
    }

    #[inline]
    pub fn get_component<T: 'static + Send + Sync>(&self, entity: EntityId) -> Option<&T> {
        self.registry
            .get_storage::<T>()?
            .as_any()
            .downcast_ref::<VecStorage<T>>()
            .and_then(|storage| storage.get(entity))
    }

    #[inline]
    pub fn get_component_mut<T: 'static + Send + Sync>(
        &mut self,
        entity: EntityId,
    ) -> Option<&mut T> {
        self.registry
            .get_storage_mut::<T>()
            .as_any_mut()
            .downcast_mut::<VecStorage<T>>()
            .and_then(|storage| storage.get_mut(entity))
    }

    pub fn remove_component<T: 'static + Send + Sync>(&mut self, entity: EntityId) -> bool {
        let storage = self.registry.get_storage_mut::<T>();
        storage.remove(entity)
    }

    #[inline]
    pub fn has_component<T: 'static + Send + Sync>(&self, entity: EntityId) -> bool {
        self.registry
            .get_storage::<T>()
            .map_or(false, |storage| storage.contains(entity))
    }

    pub fn clear(&mut self) {
        self.registry.clear();
    }

    pub fn query<T: 'static + Send + Sync>(&self) -> Box<dyn Iterator<Item = (EntityId, &T)> + '_> {
        if let Some(storage) = self.registry.get_storage::<T>() {
            if let Some(vec_storage) = storage.as_any().downcast_ref::<VecStorage<T>>() {
                return Box::new(vec_storage.iter());
            }
        }
        Box::new(std::iter::empty())
    }

    pub fn query_mut<T: 'static + Send + Sync>(
        &mut self,
    ) -> Box<dyn Iterator<Item = (EntityId, &mut T)> + '_> {
        let storage = self.registry.get_storage_mut::<T>();
        if let Some(vec_storage) = storage.as_any_mut().downcast_mut::<VecStorage<T>>() {
            return Box::new(vec_storage.iter_mut());
        }
        Box::new(std::iter::empty())
    }
}

#[cfg(test)]
mod tests {
    use crate::ecs::component::ComponentManager;
    use crate::ecs::entity::EntityManager;

    #[derive(Debug, PartialEq)]
    struct Position {
        x: f32,
        y: f32,
    }

    #[test]
    fn test_component_insertion() {
        let mut entity_manager = EntityManager::new();
        let mut component_manager = ComponentManager::new();

        let entity = entity_manager.create();
        component_manager.register::<Position>();

        let pos = Position { x: 1.0, y: 2.0 };
        component_manager.insert_component(entity, pos);

        let retrieved = component_manager.get_component::<Position>(entity);
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().x, 1.0);
        assert_eq!(retrieved.unwrap().y, 2.0);
    }

    #[test]
    fn test_component_removal() {
        let mut entity_manager = EntityManager::new();
        let mut component_manager = ComponentManager::new();

        let entity = entity_manager.create();
        component_manager.register::<Position>();

        let pos = Position { x: 1.0, y: 2.0 };
        component_manager.insert_component(entity, pos);

        assert!(component_manager.remove_component::<Position>(entity));
        assert!(component_manager
            .get_component::<Position>(entity)
            .is_none());
    }

    #[test]
    fn test_component_query() {
        let mut entity_manager = EntityManager::new();
        let mut component_manager = ComponentManager::new();

        let entity1 = entity_manager.create();
        let entity2 = entity_manager.create();

        component_manager.register::<Position>();

        component_manager.insert_component(entity1, Position { x: 1.0, y: 1.0 });
        component_manager.insert_component(entity2, Position { x: 2.0, y: 2.0 });

        let positions: Vec<(_, &Position)> = component_manager.query::<Position>().collect();
        assert_eq!(positions.len(), 2);
    }
}
