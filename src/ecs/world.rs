use super::error::{EcsError, Result};
use crate::ecs::component::{Component, ComponentManager};
use crate::ecs::entity::{EntityId, EntityManager};
use crate::ecs::query::{Query, QueryMut, QueryPair, QueryPairMut};
use crate::ecs::system::{System, SystemId, SystemManager};
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
}

// Entity Management
impl World {
    #[inline]
    pub fn spawn(&mut self) -> EntityId {
        let entity = self.entities.create();
        // Ensure component storage capacity matches entity capacity
        self.components.ensure_capacity(self.entities.capacity());
        entity
    }

    #[inline]
    pub fn despawn(&mut self, entity: EntityId) -> Result<()> {
        if !self.entities.is_alive(entity) {
            return Err(EcsError::InvalidEntity(entity));
        }
        self.entities.remove(entity);
        Ok(())
    }

    #[inline]
    pub fn is_alive(&self, entity: EntityId) -> bool {
        self.entities.is_alive(entity)
    }

    pub fn entities(&self) -> impl Iterator<Item = EntityId> + '_ {
        self.entities.iter()
    }

    pub fn clear(&mut self) {
        self.entities.clear();
        self.components.clear();
    }
}

// Component Management
impl World {
    pub fn add_component<T: Component>(&mut self, entity: EntityId, component: T) -> Result<()> {
        if !self.entities.is_alive(entity) {
            return Err(EcsError::InvalidEntity(entity));
        }

        // Ensure the component type is registered
        self.components.register::<T>();

        // Insert the component
        self.components.insert_component(entity, component);
        Ok(())
    }

    #[inline]
    pub fn get_component<T: Component>(&self, entity: EntityId) -> Result<&T> {
        if !self.entities.is_alive(entity) {
            return Err(EcsError::InvalidEntity(entity));
        }
        self.components
            .get_component::<T>(entity)
            .ok_or_else(|| EcsError::ComponentError("Component not found".to_string()))
    }

    #[inline]
    pub fn get_component_mut<T: Component>(&mut self, entity: EntityId) -> Result<&mut T> {
        if !self.entities.is_alive(entity) {
            return Err(EcsError::InvalidEntity(entity));
        }
        self.components
            .get_component_mut::<T>(entity)
            .ok_or_else(|| EcsError::ComponentError("Component not found".to_string()))
    }

    #[inline]
    pub fn has_component<T: Component>(&self, entity: EntityId) -> bool {
        self.entities.is_alive(entity) && self.components.has_component::<T>(entity)
    }
}

// Query System
impl World {
    pub fn query<T: Component>(&self) -> Query<T> {
        Query::new(self)
    }

    pub fn query_mut<T: Component>(&mut self) -> QueryMut<T> {
        QueryMut::new(self)
    }

    pub fn query_pair<A: Component, B: Component>(&self) -> QueryPair<A, B> {
        QueryPair::new(self)
    }

    pub fn query_pair_mut<A: Component, B: Component>(&mut self) -> QueryPairMut<A, B> {
        QueryPairMut::new(self)
    }
}

// System Management
impl World {
    pub fn add_system<S: System + 'static>(&mut self, system: S) -> SystemId {
        self.systems.insert_system(Box::new(system))
    }

    pub fn add_system_with_dependencies<S: System + 'static>(
        &mut self,
        system: S,
        dependencies: Vec<SystemId>,
    ) -> Result<SystemId> {
        let id = self.systems.insert_system(Box::new(system));
        for dep_id in dependencies {
            self.systems.add_dependency(id, dep_id)?;
        }
        Ok(id)
    }

    pub fn run_systems(&mut self) -> Result<()> {
        // Create new instances to swap with
        let mut temp_world = World::default();
        let mut temp_manager = SystemManager::new();

        // Swap the contents instead of moving ownership
        std::mem::swap(self, &mut temp_world);
        std::mem::swap(&mut self.systems, &mut temp_manager);

        // Get the scheduler reference before moving temp_world
        let scheduler = &self.scheduler;

        // Execute systems
        let result = scheduler.execute_systems(temp_world, temp_manager);

        if let Ok((modified_world, modified_manager)) = result {
            // Swap back the modified versions
            *self = modified_world;
            self.systems = modified_manager;
            Ok(())
        } else {
            // Handle error case - might want to restore original state
            Err(EcsError::SystemError("System execution failed".to_string()))
        }
    }
}

// Resource Management
impl World {
    pub fn add_resource<T: 'static + Send + Sync>(&mut self, resource: T) {
        self.resources
            .insert(resource)
            .expect("Failed to insert resource");
    }

    pub fn get_resource<T: 'static>(&self) -> Result<&T> {
        self.resources.get().map_err(EcsError::ResourceError)
    }

    pub fn get_resource_mut<T: 'static>(&mut self) -> Result<&mut T> {
        self.resources.get_mut().map_err(EcsError::ResourceError)
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
    fn test_world_query() {}

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
