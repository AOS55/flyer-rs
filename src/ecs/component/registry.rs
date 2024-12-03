use super::storage::{ComponentStorage, StorageType};
use std::any::TypeId;
use std::collections::HashMap;

pub struct ComponentRegistry {
    storages: HashMap<TypeId, Box<dyn ComponentStorage>>,
    storage_constructors: HashMap<TypeId, StorageType>,
}

impl ComponentRegistry {
    pub fn new() -> Self {
        Self {
            storages: HashMap::new(),
            storage_constructors: HashMap::new(),
        }
    }

    pub fn register<T: 'static>(&mut self, storage_constructor: StorageType) {
        let type_id = TypeId::of::<T>();
        self.storage_constructors
            .insert(type_id, storage_constructor);
    }

    pub fn ensure_capacity(&mut self, capacity: usize) {
        for storage in self.storages.values_mut() {
            storage.resize(capacity);
        }
    }

    pub fn get_storage<T: 'static>(&self) -> Option<&dyn ComponentStorage> {
        self.storages.get(&TypeId::of::<T>()).map(|s| s.as_ref())
    }

    pub fn get_storage_mut<T: 'static>(&mut self) -> &mut dyn ComponentStorage {
        let type_id = TypeId::of::<T>();

        if !self.storages.contains_key(&type_id) {
            if let Some(constructor) = self.storage_constructors.get(&type_id) {
                self.storages.insert(type_id, constructor());
            }
        }

        self.storages.get_mut(&type_id).map(|s| s.as_mut()).unwrap()
    }

    pub fn clear(&mut self) {
        for storage in self.storages.values_mut() {
            storage.clear();
        }
    }
}
