use crate::ecs::entity::id::EntityId;
use std::any::Any;
use std::collections::HashMap;

pub trait ComponentStorage: Send + Sync {
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
    fn remove(&mut self, entity: EntityId) -> bool;
    fn contains(&self, entity: EntityId) -> bool;
    fn clear(&mut self);
}

pub struct VecStorage<T: 'static> {
    data: HashMap<EntityId, T>,
}

impl<T: 'static> VecStorage<T> {
    pub fn new() -> Self {
        Self {
            data: HashMap::new(),
        }
    }

    pub fn insert(&mut self, entity: EntityId, component: T) {
        self.data.insert(entity, component);
    }

    pub fn get(&self, entity: EntityId) -> Option<&T> {
        self.data.get(&entity)
    }

    pub fn get_mut(&mut self, entity: EntityId) -> Option<&mut T> {
        self.data.get_mut(&entity)
    }

    pub fn iter(&self) -> impl Iterator<Item = (EntityId, &T)> {
        self.data
            .iter()
            .map(|(entity_id, component)| (*entity_id, component))
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = (EntityId, &mut T)> {
        self.data
            .iter_mut()
            .map(|(entity_id, component)| (*entity_id, component))
    }
}

impl<T: 'static + Send + Sync> ComponentStorage for VecStorage<T> {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn remove(&mut self, entity: EntityId) -> bool {
        self.data.remove(&entity).is_some()
    }

    fn contains(&self, entity: EntityId) -> bool {
        self.data.contains_key(&entity)
    }

    fn clear(&mut self) {
        self.data.clear();
    }
}

pub type StorageType = fn() -> Box<dyn ComponentStorage>;
