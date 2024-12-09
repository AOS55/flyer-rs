use bevy::prelude::*;
use std::collections::HashMap;

use crate::components::terrain::*;
use crate::resources::terrain::{TerrainAssets, TerrainConfig, TerrainState};
use crate::systems::terrain::{ChunkManagerPlugin, TerrainGeneratorSystem, TerrainRenderPlugin};

pub struct TerrainPlugin;

impl TerrainPlugin {
    fn setup_config(mut commands: Commands) {
        // Initialize with default config as single source of truth
        commands.insert_resource(TerrainConfig::default());
    }

    fn setup_state(
        mut commands: Commands,
        existing_config: Option<Res<TerrainConfig>>,
        existing_state: Option<Res<TerrainState>>,
    ) {
        if existing_config.is_none() {
            commands.insert_resource(TerrainConfig::default());
        }

        if existing_state.is_none() {
            let terrain_state = TerrainState {
                // Core parameters from config
                chunk_size: 16,
                scale: 1.0,
                seed: rand::random(), // Or from config

                // Runtime state
                active_chunks: Vec::new(),
                tile_size: 16.0,
                chunks_to_load: Default::default(),
                chunks_to_unload: Default::default(),

                // Loading parameters
                loading_radius: 5,
                max_chunks_per_frame: 8,
            };

            commands.insert_resource(terrain_state);
        }
    }

    fn setup_assets(
        mut commands: Commands,
        asset_server: Res<AssetServer>,
        mut texture_layouts: ResMut<Assets<TextureAtlasLayout>>,
    ) {
        // Crate texture atlas layouts
        let tile_layout = TextureAtlasLayout::from_grid(UVec2::new(16, 16), 3, 3, None, None);
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
            .add_plugins((ChunkManagerPlugin));
    }
}

fn setup_sprite_mappings(terrain_assets: &mut TerrainAssets) {
    // Tile mappings
    let tile_mappings = [
        (BiomeType::Grass, 0),
        (BiomeType::Forest, 1),
        (BiomeType::Crops, 2),
        (BiomeType::Orchard, 3),
        (BiomeType::Water, 4),
        (BiomeType::Beach, 5),
        (BiomeType::Desert, 6),
        (BiomeType::Mountain, 7),
        (BiomeType::Snow, 8),
    ];

    // Feature mappings
    let feature_mappings = [
        (FeatureType::Tree(TreeVariant::AppleTree), 0),
        (FeatureType::Tree(TreeVariant::BananaTree), 1),
        (FeatureType::Tree(TreeVariant::EvergreenFir), 2),
        (FeatureType::Tree(TreeVariant::PrunedTree), 3),
        (FeatureType::Tree(TreeVariant::CoconutPalm), 4),
        (FeatureType::Tree(TreeVariant::Palm), 5),
        (FeatureType::Tree(TreeVariant::WiltingFir), 6),
        (FeatureType::Tree(TreeVariant::Cactus), 7),
        (FeatureType::Bush(BushVariant::GreenBushel), 8),
        (FeatureType::Bush(BushVariant::DeadBushel), 9),
        (FeatureType::Bush(BushVariant::RipeBushel), 10),
        (FeatureType::Flower(FlowerVariant::BerryBush), 11),
        (FeatureType::Flower(FlowerVariant::MushroomCluster), 12),
        (FeatureType::Flower(FlowerVariant::Reeds), 13),
        (FeatureType::Flower(FlowerVariant::WildFlower), 14),
        (FeatureType::Snow(SnowVariant::SnowMan), 15),
        (FeatureType::Rock(RockVariant::Log), 16),
        (FeatureType::Rock(RockVariant::RoundRock), 17),
        (FeatureType::Rock(RockVariant::CrackedRock), 18),
        (FeatureType::Rock(RockVariant::IrregularRock), 19),
        (FeatureType::Rock(RockVariant::BrownRock), 20),
        (FeatureType::Rock(RockVariant::JaggedRock), 21),
    ];

    // Initialize mappings
    for (biome, index) in tile_mappings {
        terrain_assets.tile_mappings.insert(biome, index);
    }

    for (feature, index) in feature_mappings {
        terrain_assets.feature_mappings.insert(feature, index);
    }
}
