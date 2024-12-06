use bevy::prelude::*;

use crate::components::render::{RenderProperties, SpriteAnimation};
use crate::systems::render::sprite_assets::{load_sprite_assets, SpriteAssets};
use crate::systems::render::sprite_systems::{
    sprite_animation_system, sprite_layer_system, update_sprite_properties,
};

#[derive(Debug, Hash, PartialEq, Eq, Clone, SystemSet)]
pub enum RenderSet {
    Layout,     // For layer and transform updates
    Animation,  // For sprite animations
    Properties, // For property updates
}

// Add this to track the state of sprite asset loading
#[derive(States, Debug, Clone, Copy, Eq, PartialEq, Hash, Default)]
pub enum SpriteLoadState {
    #[default]
    Loading,
    Loaded,
}

pub struct FlightRenderPlugin;

impl Plugin for FlightRenderPlugin {
    fn build(&self, app: &mut App) {
        app
            // Register components
            .register_type::<RenderProperties>()
            .register_type::<SpriteAnimation>()
            // Add resources
            .init_resource::<SpriteAssets>()
            // Add state management
            .init_state::<SpriteLoadState>()
            // Configure system sets
            .configure_sets(
                Update,
                (
                    RenderSet::Layout,
                    RenderSet::Animation,
                    RenderSet::Properties,
                )
                    .chain(),
            )
            // Add systems with conditions
            .add_systems(Startup, load_sprite_assets)
            .add_systems(
                Update,
                (
                    sprite_layer_system.in_set(RenderSet::Layout),
                    sprite_animation_system
                        .in_set(RenderSet::Animation)
                        .run_if(in_state(SpriteLoadState::Loaded)),
                    update_sprite_properties
                        .in_set(RenderSet::Properties)
                        .run_if(in_state(SpriteLoadState::Loaded)),
                ),
            )
            // Add system to check when sprites are loaded
            .add_systems(
                Update,
                check_sprite_loading.run_if(in_state(SpriteLoadState::Loading)),
            );
    }
}

// Add this system to check when sprites are fully loaded
fn check_sprite_loading(
    mut next_state: ResMut<NextState<SpriteLoadState>>,
    sprite_assets: Res<SpriteAssets>,
    asset_server: Res<AssetServer>,
) {
    // Check if all textures are loaded
    let mut all_loaded = true;
    for handle in sprite_assets.textures.values() {
        if !asset_server.is_loaded_with_dependencies(handle) {
            all_loaded = false;
            break;
        }
    }

    if all_loaded {
        next_state.set(SpriteLoadState::Loaded);
    }
}
