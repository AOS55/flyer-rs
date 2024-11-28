use std::num::NonZeroU64;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct EntityId(NonZeroU64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Generation(u32);

impl EntityId {
    pub const fn new(index: u32, generation: Generation) -> Self {
        let id = ((generation.0 as u64) << 32) | (index as u64);
        Self(unsafe { NonZeroU64::new_unchecked(id + 1) })
    }

    pub fn index(&self) -> u32 {
        (self.0.get() - 1) as u32
    }

    pub fn generation(&self) -> Generation {
        Generation(((self.0.get() - 1) >> 32) as u32)
    }
}

impl Generation {
    pub fn increment(self) -> Self {
        Self(self.0.wrapping_add(1))
    }
}

impl Default for Generation {
    fn default() -> Self {
        Self(0)
    }
}
