use bevy::prelude::*;
use std::collections::HashMap;

use crate::components::terrain::*;
use crate::resources::terrain::{TerrainAssets, TerrainConfig, TerrainState};
use crate::systems::terrain::{
    terrain_generation_system, ChunkManagerPlugin, TerrainGeneratorSystem, TerrainRenderPlugin,
};

pub struct TerrainPlugin;

impl TerrainPlugin {
    fn setup_config(mut commands: Commands) {
        // Initialize with default config as single source of truth
        commands.insert_resource(TerrainConfig::default());
    }

    fn setup_state(mut commands: Commands, config: Res<TerrainConfig>) {
        let terrain_state = TerrainState {
            // Core parameters from config
            chunk_size: config.render.tile_size as u32,
            world_size: IVec2::new(1000, 1000), // Could come from config
            scale: config.render.tile_size,
            seed: rand::random(), // Or from config

            // Runtime state
            active_chunks: Vec::new(),
            chunks_to_load: Default::default(),
            chunks_to_unload: Default::default(),

            // Loading parameters
            loading_radius: 5,
            max_chunks_per_frame: 8,
        };

        commands.insert_resource(terrain_state);
    }

    fn setup_assets(
        mut commands: Commands,
        asset_server: Res<AssetServer>,
        mut texture_layouts: ResMut<Assets<TextureAtlasLayout>>,
    ) {
        // Crate texture atlas layouts
        let tile_layout = TextureAtlasLayout::from_grid(UVec2::new(16, 16), 5, 4, None, None);
        let feature_layout = TextureAtlasLayout::from_grid(UVec2::new(16, 16), 5, 5, None, None);

        let tile_layout_handle = texture_layouts.add(tile_layout);
        let feature_layout_handle = texture_layouts.add(feature_layout);

        // Initialize terrain assets
        let mut terrain_assets = TerrainAssets {
            tile_texture: asset_server.load("textures/terrain_tiles.png"),
            feature_texture: asset_server.load("textures/terrain_features.png"),
            tile_layout: tile_layout_handle,
            feature_layout: feature_layout_handle,
            tile_mappings: HashMap::new(),
            feature_mappings: HashMap::new(),
        };

        setup_sprite_mappings(&mut terrain_assets);
        commands.insert_resource(terrain_assets);
    }

    fn setup_generator(
        mut commands: Commands,
        terrain_state: Res<TerrainState>,
        terrain_config: Res<TerrainConfig>,
    ) {
        let generator = TerrainGeneratorSystem::new(&terrain_state, &terrain_config);
        commands.insert_resource(generator);
    }
}

impl Plugin for TerrainPlugin {
    fn build(&self, app: &mut App) {
        app
            // Resource initialization in correct order
            .add_systems(
                Startup,
                (
                    Self::setup_config,
                    Self::setup_state.after(Self::setup_config),
                    Self::setup_assets.after(Self::setup_state),
                    Self::setup_generator.after(Self::setup_assets),
                )
                    .chain(),
            )
            // Add sub-plugins
            .add_plugins((ChunkManagerPlugin, TerrainRenderPlugin))
            // Main systems
            .add_systems(Update, (terrain_generation_system,));
    }
}

fn setup_sprite_mappings(terrain_assets: &mut TerrainAssets) {
    // Tile mappings
    let tile_mappings = [
        (BiomeType::Grass, 9),
        (BiomeType::Forest, 11),
        (BiomeType::Crops, 13),
        (BiomeType::Orchard, 5),
        (BiomeType::Water, 17),
        (BiomeType::Sand, 15),
    ];

    // Feature mappings
    let feature_mappings = [
        (FeatureType::Tree(TreeVariant::EvergreenFir), 4),
        (FeatureType::Tree(TreeVariant::WiltingFir), 4),
        (FeatureType::Tree(TreeVariant::AppleTree), 0),
        (FeatureType::Tree(TreeVariant::PrunedTree), 2),
        (FeatureType::Bush(BushVariant::GreenBushel), 9),
        (FeatureType::Bush(BushVariant::RipeBushel), 14),
        (FeatureType::Bush(BushVariant::DeadBushel), 3),
        (FeatureType::Flower(FlowerVariant::Single), 12),
        (FeatureType::Flower(FlowerVariant::Double), 12),
        (FeatureType::Flower(FlowerVariant::Quad), 12),
        (FeatureType::Flower(FlowerVariant::Cluster), 12),
        (FeatureType::Rock, 19),
    ];

    // Initialize mappings
    for (biome, index) in tile_mappings {
        terrain_assets.tile_mappings.insert(biome, index);
    }

    for (feature, index) in feature_mappings {
        terrain_assets.feature_mappings.insert(feature, index);
    }
}
