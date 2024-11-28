use super::id::{EntityId, Generation};
use std::collections::VecDeque;

pub struct EntityManager {
    generations: Vec<Generation>,
    free_indices: VecDeque<u32>,
    len: usize,
}

impl EntityManager {
    pub fn new() -> Self {
        Self {
            generations: Vec::new(),
            free_indices: VecDeque::new(),
            len: 0,
        }
    }

    pub fn create(&mut self) -> EntityId {
        if let Some(index) = self.free_indices.pop_front() {
            let generation = self.generations[index as usize];
            EntityId::new(index, generation)
        } else {
            let index = self.generations.len() as u32;
            self.generations.push(Generation::default());
            self.len += 1;
            EntityId::new(index, Generation::default())
        }
    }

    pub fn remove_entity(&mut self, id: EntityId) -> bool {
        let index = id.index() as usize;
        if index >= self.generations.len() || self.generations[index] != id.generation() {
            return false;
        }

        self.generations[index] = self.generations[index].increment();
        self.free_indices.push_back(index as u32);
        self.len -= 1;
        true
    }

    pub fn is_alive(&self, id: EntityId) -> bool {
        let index = id.index() as usize;
        index < self.generations.len() && self.generations[index] == id.generation()
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn clear(&mut self) {
        self.generations.clear();
        self.free_indices.clear();
        self.len = 0;
    }
}

#[cfg(test)]
mod tests {
    use crate::ecs::entity::EntityManager;

    #[test]
    fn test_entity_creation() {
        let mut manager = EntityManager::new();
        let entity = manager.create();
        assert!(manager.is_alive(entity));
        assert_eq!(manager.len(), 1);
    }

    #[test]
    fn test_entity_removal() {
        let mut manager = EntityManager::new();
        let entity = manager.create();
        assert!(manager.remove_entity(entity));
        assert!(!manager.is_alive(entity));
        assert_eq!(manager.len(), 0);
    }

    #[test]
    fn test_entity_generation() {
        let mut manager = EntityManager::new();
        let entity1 = manager.create();
        assert!(manager.remove_entity(entity1));
        let entity2 = manager.create();
        assert_ne!(entity1.generation(), entity2.generation());
    }
}
