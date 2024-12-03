use std::any::TypeId;
use std::collections::HashSet;

#[derive(Debug, Clone, Default)]
pub struct ComponentAccess {
    pub reads: HashSet<TypeId>,
    pub writes: HashSet<TypeId>,
}

impl ComponentAccess {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn read<T: 'static>(&mut self) {
        self.reads.insert(TypeId::of::<T>());
    }

    pub fn write<T: 'static>(&mut self) {
        self.writes.insert(TypeId::of::<T>());
    }

    /// Returns true if this access pattern conflicts with another
    pub fn conflicts_with(&self, other: &ComponentAccess) -> bool {
        // Write-Write and Write-Read conflicts
        !self.writes.is_disjoint(&other.writes)
            || !self.writes.is_disjoint(&other.reads)
            || !self.reads.is_disjoint(&other.writes)
    }
}
