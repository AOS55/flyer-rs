use super::id::EntityId;

pub struct EntityManager {
    generations: Vec<bool>,
    free_indices: Vec<u32>,
    len: usize,
}

impl EntityManager {
    pub fn new() -> Self {
        Self {
            generations: Vec::new(),
            free_indices: Vec::new(),
            len: 0,
        }
    }

    pub fn create(&mut self) -> EntityId {
        if let Some(index) = self.free_indices.pop() {
            debug_assert!(!self.generations[index as usize]);
            self.generations[index as usize] = true;
            self.len += 1;
            EntityId::new(index)
        } else {
            let index = self.generations.len() as u32;
            self.generations.push(true);
            self.len += 1;
            EntityId::new(index)
        }
    }

    pub fn remove(&mut self, id: EntityId) -> bool {
        let index = id.index();
        if index >= self.generations.len() || !self.generations[index] {
            return false;
        }

        self.generations[index] = false;
        self.free_indices.push(index as u32);
        self.len -= 1;
        true
    }

    #[inline]
    pub fn is_alive(&self, id: EntityId) -> bool {
        let index = id.index();
        index < self.generations.len() && self.generations[index]
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.len
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn clear(&mut self) {
        self.generations.fill(false);
        self.free_indices.clear();
        self.len = 0;
    }

    // Iterator over active entities
    pub fn iter(&self) -> impl Iterator<Item = EntityId> + '_ {
        self.generations
            .iter()
            .enumerate()
            .filter_map(|(index, &active)| {
                if active {
                    Some(EntityId::new(index as u32))
                } else {
                    None
                }
            })
    }

    // Get capacity (useful for component storage sizing)
    #[inline]
    pub fn capacity(&self) -> usize {
        self.generations.len()
    }
}
