use crate::ecs::component::Component;
use crate::ecs::entity::EntityId;
use crate::ecs::World;
use std::marker::PhantomData;

/// Single component immutable query
pub struct Query<'w, T: Component> {
    world: &'w World,
    entities: Vec<EntityId>,
    _marker: PhantomData<T>,
}

/// Single component mutable query
pub struct QueryMut<'w, T: Component> {
    world: &'w mut World,
    entities: Vec<EntityId>,
    _marker: PhantomData<T>,
}

/// Two component immutable query
pub struct QueryPair<'w, A: Component, B: Component> {
    world: &'w World,
    entities: Vec<EntityId>,
    _marker: PhantomData<(A, B)>,
}

/// Two component mutable query
pub struct QueryPairMut<'w, A: Component, B: Component> {
    world: &'w mut World,
    entities: Vec<EntityId>,
    _marker: PhantomData<(A, B)>,
}

// Iterator types
pub struct QueryIter<'w, T: Component> {
    query: &'w Query<'w, T>,
    index: usize,
}

pub struct QueryMutIter<'w, T: Component> {
    query: &'w mut QueryMut<'w, T>,
    index: usize,
}

pub struct QueryPairIter<'w, A: Component, B: Component> {
    query: &'w QueryPair<'w, A, B>,
    index: usize,
}

pub struct QueryPairMutIter<'w, A: Component, B: Component> {
    query: &'w mut QueryPairMut<'w, A, B>,
    index: usize,
}

// Single component query implementations
impl<'w, T: Component> Query<'w, T> {
    pub(crate) fn new(world: &'w World) -> Self {
        let entities = world
            .entities()
            .filter(|&entity| world.has_component::<T>(entity))
            .collect();

        Self {
            world,
            entities,
            _marker: PhantomData,
        }
    }

    pub fn iter(&'w self) -> QueryIter<'w, T> {
        QueryIter {
            query: self,
            index: 0,
        }
    }
}

impl<'w, T: Component> QueryMut<'w, T> {
    pub(crate) fn new(world: &'w mut World) -> Self {
        let entities = world
            .entities()
            .filter(|&entity| world.has_component::<T>(entity))
            .collect();

        Self {
            world,
            entities,
            _marker: PhantomData,
        }
    }

    pub fn iter_mut(&'w mut self) -> QueryMutIter<'w, T> {
        QueryMutIter {
            query: self,
            index: 0,
        }
    }
}

// Two component query implementations
impl<'w, A: Component, B: Component> QueryPair<'w, A, B> {
    pub(crate) fn new(world: &'w World) -> Self {
        let entities = world
            .entities()
            .filter(|&entity| world.has_component::<A>(entity) && world.has_component::<B>(entity))
            .collect();

        Self {
            world,
            entities,
            _marker: PhantomData,
        }
    }

    pub fn iter(&'w self) -> QueryPairIter<'w, A, B> {
        QueryPairIter {
            query: self,
            index: 0,
        }
    }
}

impl<'w, A: Component, B: Component> QueryPairMut<'w, A, B> {
    pub(crate) fn new(world: &'w mut World) -> Self {
        let entities = world
            .entities()
            .filter(|&entity| world.has_component::<A>(entity) && world.has_component::<B>(entity))
            .collect();

        Self {
            world,
            entities,
            _marker: PhantomData,
        }
    }

    pub fn iter_mut(&'w mut self) -> QueryPairMutIter<'w, A, B> {
        QueryPairMutIter {
            query: self,
            index: 0,
        }
    }
}

// Iterator implementations
impl<'w, T: Component> Iterator for QueryIter<'w, T> {
    type Item = (EntityId, &'w T);

    fn next(&mut self) -> Option<Self::Item> {
        while self.index < self.query.entities.len() {
            let entity = self.query.entities[self.index];
            self.index += 1;

            if let Ok(component) = self.query.world.get_component::<T>(entity) {
                return Some((entity, component));
            }
        }
        None
    }
}

impl<'w, T: Component> Iterator for QueryMutIter<'w, T> {
    type Item = (EntityId, &'w mut T);

    fn next(&mut self) -> Option<Self::Item> {
        while self.index < self.query.entities.len() {
            let entity = self.query.entities[self.index];
            self.index += 1;

            // Safety: we only visit each entity once due to the incrementing index
            unsafe {
                let world_ptr = self.query.world as *mut World;
                if let Ok(component) = (*world_ptr).get_component_mut::<T>(entity) {
                    return Some((entity, component));
                }
            }
        }
        None
    }
}

impl<'w, A: Component, B: Component> Iterator for QueryPairIter<'w, A, B> {
    type Item = (EntityId, &'w A, &'w B);

    fn next(&mut self) -> Option<Self::Item> {
        while self.index < self.query.entities.len() {
            let entity = self.query.entities[self.index];
            self.index += 1;

            let a = self.query.world.get_component::<A>(entity).ok()?;
            let b = self.query.world.get_component::<B>(entity).ok()?;
            return Some((entity, a, b));
        }
        None
    }
}

impl<'w, A: Component, B: Component> Iterator for QueryPairMutIter<'w, A, B> {
    type Item = (EntityId, &'w mut A, &'w mut B);

    fn next(&mut self) -> Option<Self::Item> {
        while self.index < self.query.entities.len() {
            let entity = self.query.entities[self.index];
            self.index += 1;

            // Safety: we only visit each entity once, and A and B are different types
            unsafe {
                let world_ptr = self.query.world as *mut World;
                let a = (*world_ptr).get_component_mut::<A>(entity).ok()?;
                let b = (*world_ptr).get_component_mut::<B>(entity).ok()?;
                return Some((entity, a, b));
            }
        }
        None
    }
}
