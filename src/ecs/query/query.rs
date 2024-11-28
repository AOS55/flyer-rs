use crate::ecs::entity::EntityId;
use crate::ecs::{Component, World};
use std::marker::PhantomData;

pub trait QueryItem: Send + Sync {
    type Item<'a>: 'a
    where
        Self: 'a;
    type ItemMut<'a>: 'a
    where
        Self: 'a;

    fn fetch<'a>(world: &'a World, entity: EntityId) -> Option<Self::Item<'a>>;
    fn fetch_mut<'a>(world: &'a mut World, entity: EntityId) -> Option<Self::ItemMut<'a>>;
    fn matches(world: &World, entity: EntityId) -> bool;
}

// Single component query implementation
impl<T: Component + 'static> QueryItem for T {
    type Item<'a> = &'a T;
    type ItemMut<'a> = &'a mut T;

    fn fetch<'a>(world: &'a World, entity: EntityId) -> Option<Self::Item<'a>> {
        world.get_component::<T>(entity).ok()
    }

    fn fetch_mut<'a>(world: &'a mut World, entity: EntityId) -> Option<Self::ItemMut<'a>> {
        world.get_component_mut::<T>(entity).ok()
    }

    fn matches(world: &World, entity: EntityId) -> bool {
        world.has_component::<T>(entity)
    }
}

// Two component query implementation with split borrows
pub struct ComponentPair<A, B>(PhantomData<(A, B)>);

impl<A: Component + 'static, B: Component + 'static> QueryItem for ComponentPair<A, B> {
    type Item<'a> = (&'a A, &'a B);
    type ItemMut<'a> = (&'a mut A, &'a mut B);

    fn fetch<'a>(world: &'a World, entity: EntityId) -> Option<Self::Item<'a>> {
        Some((
            world.get_component::<A>(entity).ok()?,
            world.get_component::<B>(entity).ok()?,
        ))
    }

    fn fetch_mut<'a>(world: &'a mut World, entity: EntityId) -> Option<Self::ItemMut<'a>> {
        // We need to implement split borrows here to avoid the multiple mutable borrow issue
        unsafe {
            let world_ptr = world as *mut World;
            Some((
                (*world_ptr).get_component_mut::<A>(entity).ok()?,
                (*world_ptr).get_component_mut::<B>(entity).ok()?,
            ))
        }
    }

    fn matches(world: &World, entity: EntityId) -> bool {
        world.has_component::<A>(entity) && world.has_component::<B>(entity)
    }
}

pub struct Query<'w, Q: QueryItem + 'w> {
    world: &'w World,
    entities: Vec<EntityId>,
    current: usize,
    _marker: PhantomData<Q>,
}

pub struct QueryMut<'w, Q: QueryItem + 'w> {
    world: &'w mut World,
    entities: Vec<EntityId>,
    current: usize,
    _marker: PhantomData<Q>,
}

impl<'w, Q: QueryItem + 'w> Query<'w, Q> {
    pub(crate) fn new(world: &'w World) -> Self {
        let mut entities = Vec::new();
        for entity_id in world.entities() {
            if Q::matches(world, entity_id) {
                entities.push(entity_id);
            }
        }

        Self {
            world,
            entities,
            current: 0,
            _marker: PhantomData,
        }
    }

    pub fn filter<F>(self, filter_fn: F) -> QueryFilter<'w, Q, F>
    where
        F: FnMut(EntityId, &Q::Item<'_>) -> bool,
    {
        QueryFilter {
            query: self,
            filter_fn,
        }
    }
}

impl<'w, Q: QueryItem + 'w> QueryMut<'w, Q> {
    pub(crate) fn new(world: &'w mut World) -> Self {
        let entities: Vec<_> = world
            .entities()
            .filter(|&entity| Q::matches(world, entity))
            .collect();

        Self {
            world,
            entities,
            current: 0,
            _marker: PhantomData,
        }
    }
}

impl<'w, Q: QueryItem + 'w> Iterator for Query<'w, Q> {
    type Item = (EntityId, Q::Item<'w>);

    fn next(&mut self) -> Option<Self::Item> {
        while self.current < self.entities.len() {
            let entity = self.entities[self.current];
            self.current += 1;

            if let Some(components) = Q::fetch(self.world, entity) {
                return Some((entity, components));
            }
        }
        None
    }
}

impl<'w, Q: QueryItem + 'w> Iterator for QueryMut<'w, Q> {
    type Item = (EntityId, Q::ItemMut<'w>);

    fn next(&mut self) -> Option<Self::Item> {
        while self.current < self.entities.len() {
            let entity = self.entities[self.current];
            self.current += 1;

            // Safety: We know each entity is only processed once due to the iterator
            unsafe {
                let world_ptr = self.world as *mut World;
                if let Some(components) = Q::fetch_mut(&mut *world_ptr, entity) {
                    return Some((entity, components));
                }
            }
        }
        None
    }
}

pub struct QueryFilter<'w, Q: QueryItem + 'w, F> {
    query: Query<'w, Q>,
    filter_fn: F,
}

pub struct QueryFilterMut<'w, Q: QueryItem + 'w, F> {
    query: QueryMut<'w, Q>,
    filter_fn: F,
}

impl<'w, Q, F> Iterator for QueryFilter<'w, Q, F>
where
    Q: QueryItem + 'w,
    F: FnMut(EntityId, &Q::Item<'_>) -> bool,
{
    type Item = (EntityId, Q::Item<'w>);

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(item) = self.query.next() {
            if (self.filter_fn)(item.0, &item.1) {
                return Some(item);
            }
        }
        None
    }
}

impl<'w, Q, F> Iterator for QueryFilterMut<'w, Q, F>
where
    Q: QueryItem + 'w,
    F: FnMut(EntityId, &Q::ItemMut<'_>) -> bool,
{
    type Item = (EntityId, Q::ItemMut<'w>);

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(item) = self.query.next() {
            if (self.filter_fn)(item.0, &item.1) {
                return Some(item);
            }
        }
        None
    }
}
