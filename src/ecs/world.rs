use super::error::{EcsError, Result};
use crate::ecs::component::{Component, ComponentManager};
use crate::ecs::entity::{EntityId, EntityManager, Generation};
use crate::ecs::query::{Query, QueryItem, QueryMut};
use crate::ecs::system::SystemManager;
use crate::resources::ResourceSystem;

pub struct World {
    entities: EntityManager,
    components: ComponentManager,
    systems: SystemManager,
    resources: ResourceSystem,
}

impl World {
    pub fn new() -> Self {
        Self {
            entities: EntityManager::new(),
            components: ComponentManager::new(),
            systems: SystemManager::new(),
            resources: ResourceSystem::builder()
                .build()
                .expect("Failed to create resource system"),
        }
    }

    pub fn with_resources(resources: ResourceSystem) -> Self {
        Self {
            entities: EntityManager::new(),
            components: ComponentManager::new(),
            systems: SystemManager::new(),
            resources,
        }
    }

    pub fn spawn(&mut self) -> EntityId {
        self.entities.create()
    }

    pub fn despawn(&mut self, entity: EntityId) -> Result<()> {
        if !self.entities.is_alive(entity) {
            return Err(EcsError::InvalidEntity(entity));
        }
        self.entities.remove_entity(entity);
        Ok(())
    }

    pub fn is_alive(&self, entity: EntityId) -> bool {
        self.entities.is_alive(entity)
    }

    pub fn add_component<T: Component + 'static + Send + Sync>(
        &mut self,
        entity: EntityId,
        component: T,
    ) -> Result<()> {
        if !self.entities.is_alive(entity) {
            return Err(EcsError::InvalidEntity(entity));
        }
        self.components.register::<T>();
        self.components.insert_component(entity, component);
        Ok(())
    }

    pub fn get_component<T: Component + 'static + Send + Sync>(
        &self,
        entity: EntityId,
    ) -> Result<&T> {
        if !self.entities.is_alive(entity) {
            return Err(EcsError::InvalidEntity(entity));
        }
        self.components
            .get_component::<T>(entity)
            .ok_or_else(|| EcsError::ComponentError("Component not found".to_string()))
    }

    pub fn get_component_mut<T: Component + 'static + Send + Sync>(
        &mut self,
        entity: EntityId,
    ) -> Result<&mut T> {
        if !self.entities.is_alive(entity) {
            return Err(EcsError::InvalidEntity(entity));
        }
        self.components
            .get_component_mut::<T>(entity)
            .ok_or_else(|| EcsError::ComponentError("Component not found".to_string()))
    }

    pub fn get_resource<T: 'static>(&self) -> Result<&T> {
        self.resources.get().map_err(|e| EcsError::ResourceError(e))
    }

    pub fn get_resource_mut<T: 'static>(&mut self) -> Result<&mut T> {
        self.resources
            .get_mut()
            .map_err(|e| EcsError::ResourceError(e))
    }

    pub fn add_resource<T: 'static + Send + Sync>(&mut self, resource: T) {
        self.resources
            .insert(resource)
            .expect("Failed to insert resource"); // Or handle more gracefully if preferred
    }

    pub fn step(&mut self, _dt: f64) -> Result<()> {
        // Create a temporary reference to avoid multiple mutable borrows
        let world_ref = self as *mut World;
        unsafe { self.systems.run_systems(&mut *world_ref) }
    }

    pub fn query<Q: QueryItem + 'static>(&self) -> Query<Q> {
        Query::new(self)
    }

    pub fn query_mut<Q: QueryItem + 'static>(&mut self) -> QueryMut<Q> {
        QueryMut::new(self)
    }

    pub fn has_component<T: Component + 'static>(&self, entity: EntityId) -> bool {
        if !self.entities.is_alive(entity) {
            return false;
        }
        self.components.has_component::<T>(entity)
    }

    pub fn entities(&self) -> impl Iterator<Item = EntityId> + '_ {
        (0..self.entities.len()).filter_map(|i| {
            let entity = EntityId::new(i as u32, Generation::default());
            if self.entities.is_alive(entity) {
                Some(entity)
            } else {
                None
            }
        })
    }
}

impl Default for World {
    fn default() -> Self {
        Self::with_resources(
            ResourceSystem::new().expect("Failed to create default resource system"),
        )
    }
}

#[cfg(test)]
mod tests {
    use crate::ecs::{Component, World};
    use std::any::Any;

    #[derive(Debug, PartialEq, Clone)]
    struct GameState {
        score: i32,
    }

    #[derive(Debug, PartialEq)]
    struct Position {
        x: f32,
        y: f32,
    }

    // Implement Component trait for Position
    impl Component for Position {
        fn as_any(&self) -> &dyn Any {
            self
        }

        fn as_any_mut(&mut self) -> &mut dyn Any {
            self
        }
    }

    #[derive(Debug, PartialEq)]
    struct Velocity {
        x: f32,
        y: f32,
    }

    // Implement Component trait for Velocity
    impl Component for Velocity {
        fn as_any(&self) -> &dyn Any {
            self
        }

        fn as_any_mut(&mut self) -> &mut dyn Any {
            self
        }
    }

    #[test]
    fn test_world_entity_management() {
        let mut world = World::new();

        let entity = world.spawn();
        assert!(world.is_alive(entity));

        world.despawn(entity).unwrap();
        assert!(!world.is_alive(entity));
    }

    #[test]
    fn test_world_component_management() {
        let mut world = World::new();

        let entity = world.spawn();
        let pos = Position { x: 1.0, y: 2.0 };

        world.add_component(entity, pos).unwrap();

        let retrieved = world.get_component::<Position>(entity).unwrap();
        assert_eq!(retrieved.x, 1.0);
        assert_eq!(retrieved.y, 2.0);
    }

    #[test]
    fn test_world_query() {
        let mut world = World::new();

        let entity1 = world.spawn();
        let entity2 = world.spawn();

        world
            .add_component(entity1, Position { x: 1.0, y: 1.0 })
            .unwrap();
        world
            .add_component(entity2, Position { x: 2.0, y: 2.0 })
            .unwrap();

        let positions: Vec<_> = world.query::<Position>().collect();
        assert_eq!(positions.len(), 2);
    }

    #[test]
    fn test_multiple_components() {
        let mut world = World::new();
        let entity = world.spawn();

        world
            .add_component(entity, Position { x: 1.0, y: 1.0 })
            .unwrap();
        world
            .add_component(entity, Velocity { x: 0.5, y: 0.5 })
            .unwrap();

        let pos = world.get_component::<Position>(entity).unwrap();
        let vel = world.get_component::<Velocity>(entity).unwrap();

        assert_eq!(pos.x, 1.0);
        assert_eq!(vel.x, 0.5);
    }

    #[test]
    fn test_world_resource_management() {
        let mut world = World::new();

        let state = GameState { score: 100 };
        world.add_resource(state);

        let retrieved = world.get_resource::<GameState>().unwrap();
        assert_eq!(retrieved.score, 100);

        {
            let state = world.get_resource_mut::<GameState>().unwrap();
            state.score = 200;
        }

        let updated = world.get_resource::<GameState>().unwrap();
        assert_eq!(updated.score, 200);
    }

    #[test]
    fn test_world_component_mut() {
        let mut world = World::new();
        let entity = world.spawn();

        let pos = Position { x: 1.0, y: 2.0 };
        world.add_component(entity, pos).unwrap();

        {
            let pos = world.get_component_mut::<Position>(entity).unwrap();
            pos.x = 3.0;
            pos.y = 4.0;
        }

        let pos = world.get_component::<Position>(entity).unwrap();
        assert_eq!(pos.x, 3.0);
        assert_eq!(pos.y, 4.0);
    }
}
