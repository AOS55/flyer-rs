use crate::ecs::entity::id::EntityId;
use std::any::Any;

pub trait ComponentStorage: Send + Sync {
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
    fn remove(&mut self, entity: EntityId) -> bool;
    fn contains(&self, entity: EntityId) -> bool;
    fn clear(&mut self);
    fn resize(&mut self, new_size: usize);
}

pub struct VecStorage<T: 'static> {
    data: Vec<Option<T>>,
}

impl<T: 'static> VecStorage<T> {
    pub fn new() -> Self {
        Self { data: Vec::new() }
    }

    #[inline]
    pub fn insert(&mut self, entity: EntityId, component: T) {
        let index = entity.index();
        if index >= self.data.len() {
            self.data.resize_with(index + 1, || None);
        }
        self.data[index] = Some(component);
    }

    #[inline]
    pub fn get(&self, entity: EntityId) -> Option<&T> {
        self.data.get(entity.index()).and_then(Option::as_ref)
    }

    #[inline]
    pub fn get_mut(&mut self, entity: EntityId) -> Option<&mut T> {
        self.data.get_mut(entity.index()).and_then(Option::as_mut)
    }

    #[inline]
    pub fn iter(&self) -> impl Iterator<Item = (EntityId, &T)> {
        self.data
            .iter()
            .enumerate()
            .filter_map(|(index, component)| {
                component.as_ref().map(|c| (EntityId::new(index as u32), c))
            })
    }

    #[inline]
    pub fn iter_mut(&mut self) -> impl Iterator<Item = (EntityId, &mut T)> {
        self.data
            .iter_mut()
            .enumerate()
            .filter_map(|(index, component)| {
                component.as_mut().map(|c| (EntityId::new(index as u32), c))
            })
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
        let index = entity.index();
        if index < self.data.len() {
            if self.data[index].is_some() {
                self.data[index] = None;
                return true;
            }
        }
        false
    }

    fn contains(&self, entity: EntityId) -> bool {
        let index = entity.index();
        index < self.data.len() && self.data[index].is_some()
    }

    fn clear(&mut self) {
        self.data.clear();
    }

    fn resize(&mut self, new_size: usize) {
        if new_size > self.data.len() {
            self.data.resize_with(new_size, || None);
        }
    }
}

pub type StorageType = fn() -> Box<dyn ComponentStorage>;
