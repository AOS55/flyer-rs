use bevy::prelude::*;

use crate::components::terrain::*;
use crate::systems::terrain::{
    terrain_generation_system, ChunkManagerPlugin, TerrainGeneratorSystem, TerrainRenderPlugin,
};

/// Configuration for the terrain plugin
#[derive(Resource, Clone)]
pub struct TerrainPluginConfig {
    pub world_size: IVec2,
    pub chunk_size: u32,
    pub seed: u64,
    pub scale: f32,
    pub max_concurrent_chunks: usize,
}

impl Default for TerrainPluginConfig {
    fn default() -> Self {
        Self {
            world_size: IVec2::new(100, 100),
            chunk_size: 32,
            seed: 42,
            scale: 1.0,
            max_concurrent_chunks: 20,
        }
    }
}

pub struct TerrainPlugin {
    config: TerrainPluginConfig,
}

impl TerrainPlugin {
    pub fn new(config: TerrainPluginConfig) -> Self {
        Self { config }
    }

    fn setup_resources(
        mut commands: Commands,
        asset_server: Res<AssetServer>,
        mut texture_layouts: ResMut<Assets<TextureAtlasLayout>>,
    ) {
        // Load and set up textures
        let tile_layout = TextureAtlasLayout::from_grid(UVec2::new(16, 16), 5, 4, None, None);
        let tile_layout_handle = texture_layouts.add(tile_layout);

        let feature_layout = TextureAtlasLayout::from_grid(UVec2::new(16, 16), 5, 5, None, None);
        let feature_layout_handle = texture_layouts.add(feature_layout);

        // Create and initialize terrain assets resource
        let mut terrain_assets = TerrainAssets {
            tile_texture: asset_server.load("textures/terrain_tiles.png"),
            feature_texture: asset_server.load("textures/terrain_features.png"),
            tile_layout: tile_layout_handle,
            feature_layout: feature_layout_handle,
            tile_mappings: Default::default(),
            feature_mappings: Default::default(),
        };

        // Set up tile and feature mappings
        setup_tile_mappings(&mut terrain_assets);
        setup_feature_mappings(&mut terrain_assets);

        commands.insert_resource(terrain_assets);
    }
}

impl Plugin for TerrainPlugin {
    fn build(&self, app: &mut App) {
        // Initialize terrain state first
        let terrain_state = TerrainState {
            chunk_size: self.config.chunk_size,
            world_size: self.config.world_size,
            scale: self.config.scale,
            seed: self.config.seed,
            active_chunks: Vec::new(),
        };
        app.insert_resource(terrain_state.clone())
            .insert_resource(self.config.clone())
            // Initialize terrain config with default values
            .insert_resource(TerrainConfig::default())
            .add_systems(Startup, Self::setup_resources)
            // Initialize the generator after TerrainState exists
            .add_systems(
                Startup,
                |mut commands: Commands, terrain_state: Res<TerrainState>| {
                    let generator = TerrainGeneratorSystem::new(terrain_state.seed);
                    commands.insert_resource(generator);
                },
            )
            .add_plugins((ChunkManagerPlugin, TerrainRenderPlugin))
            .add_systems(Update, terrain_generation_system);
    }
}

fn setup_tile_mappings(terrain_assets: &mut TerrainAssets) {
    terrain_assets.tile_mappings.insert(BiomeType::Grass, 9);
    terrain_assets.tile_mappings.insert(BiomeType::Forest, 11);
    terrain_assets.tile_mappings.insert(BiomeType::Crops, 13);
    terrain_assets.tile_mappings.insert(BiomeType::Orchard, 5);
    terrain_assets.tile_mappings.insert(BiomeType::Water, 17);
    terrain_assets.tile_mappings.insert(BiomeType::Sand, 15);
}

fn setup_feature_mappings(terrain_assets: &mut TerrainAssets) {
    terrain_assets
        .feature_mappings
        .insert(FeatureType::Tree(TreeVariant::EvergreenFir), 4);
    terrain_assets
        .feature_mappings
        .insert(FeatureType::Tree(TreeVariant::WiltingFir), 4);
    terrain_assets
        .feature_mappings
        .insert(FeatureType::Tree(TreeVariant::AppleTree), 0);
    terrain_assets
        .feature_mappings
        .insert(FeatureType::Tree(TreeVariant::PrunedTree), 2);
    terrain_assets
        .feature_mappings
        .insert(FeatureType::Bush(BushVariant::GreenBushel), 9);
    terrain_assets
        .feature_mappings
        .insert(FeatureType::Bush(BushVariant::RipeBushel), 14);
    terrain_assets
        .feature_mappings
        .insert(FeatureType::Bush(BushVariant::DeadBushel), 3);
    terrain_assets
        .feature_mappings
        .insert(FeatureType::Flower(FlowerVariant::Single), 12);
    terrain_assets
        .feature_mappings
        .insert(FeatureType::Flower(FlowerVariant::Double), 12);
    terrain_assets
        .feature_mappings
        .insert(FeatureType::Flower(FlowerVariant::Quad), 12);
    terrain_assets
        .feature_mappings
        .insert(FeatureType::Flower(FlowerVariant::Cluster), 12);
    terrain_assets
        .feature_mappings
        .insert(FeatureType::Rock, 19);
}
