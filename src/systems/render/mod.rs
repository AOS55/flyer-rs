pub mod renderer;

use self::renderer::RenderSystem;
use crate::components::{RenderComponent, SpatialComponent};
use crate::ecs::system::System;
use crate::ecs::world::World;
use crate::resources::assets::AssetManager;

pub struct RenderSystemPlugin;

impl RenderSystemPlugin {
    pub fn new() -> Self {
        Self
    }

    pub fn build(&self, world: &mut World) {
        world.add_system(RenderSystem::new());
        world.add_resource(AssetManager::new());
    }
}

pub struct RenderSystemGroup {
    renderer: RenderSystem,
}

impl RenderSystemGroup {
    pub fn new() -> Self {
        Self {
            renderer: RenderSystem::new(),
        }
    }
}

impl System for RenderSystemGroup {
    fn update(&mut self, world: &mut World, dt: f64) {
        self.renderer.update(world, dt);
    }

    fn cleanup(&mut self) {
        self.renderer.cleanup();
    }
}
