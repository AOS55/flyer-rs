mod access;
mod manager;
mod scheduler;

pub use access::ComponentAccess;
pub use manager::SystemManager;
pub use scheduler::{Stage, SystemScheduler};

use crate::ecs::error::Result;
use crate::ecs::World;

pub trait System: Send + Sync {
    fn name(&self) -> &str;
    fn run(&mut self, world: &mut World) -> Result<()>;

    fn component_access(&self) -> ComponentAccess {
        ComponentAccess::default()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SystemId(pub(crate) usize);
