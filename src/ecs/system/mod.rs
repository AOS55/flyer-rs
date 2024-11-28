mod manager;
mod scheduler;

pub use manager::SystemManager;
pub use scheduler::{Stage, SystemScheduler};

use crate::ecs::error::Result;
use crate::ecs::World;

pub trait System: Send + Sync {
    fn name(&self) -> &str;
    fn run(&self, world: &mut World) -> Result<()>;
    fn dependencies(&self) -> Vec<&str> {
        Vec::new()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SystemId(pub(crate) usize);
