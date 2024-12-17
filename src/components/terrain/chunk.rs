use super::{BiomeType, TerrainFeatureComponent};
use crate::resources::terrain::TerrainState;
use bevy::prelude::*;
use std::collections::HashMap;

/// Represents a single terrain chunk in the game world.
/// A terrain chunk is a segment of the terrain grid and contains:
/// - Position information.
/// - Height and moisture data for procedural generation.
/// - Biome data for determining the type of terrain (e.g., grass, desert).
/// - Features like trees, rocks, or other terrain objects.
///
/// This component is critical for managing terrain rendering and simulation.
#[derive(Component, Debug, Clone)]
#[require(Sprite)]
pub struct TerrainChunkComponent {
    /// The position of the chunk in a 2D grid, measured in chunk coordinates.
    /// Each chunk is identified by an integer grid position (IVec2).
    pub position: IVec2,

    /// A vector storing the height map of the terrain chunk.
    /// Each value represents the height (elevation) at a specific tile within the chunk.
    pub height_map: Vec<f32>,

    /// A vector storing the moisture map of the terrain chunk.
    /// Each value represents the moisture level at a specific tile, which can affect biome types.
    pub moisture_map: Vec<f32>,

    /// A vector storing the biome type for each tile in the chunk.
    /// Biomes determine the visual appearance and behavior of the terrain (e.g., grass, water, desert).
    pub biome_map: Vec<BiomeType>,

    /// A map of terrain features (e.g., trees, rocks) within the chunk.
    /// The key is the tile index within the chunk, and the value is the `TerrainFeatureComponent`.
    pub features: HashMap<usize, TerrainFeatureComponent>,
}

impl TerrainChunkComponent {
    /// Creates a new `TerrainChunkComponent` with default values for a given position and terrain state.
    ///
    /// # Arguments
    /// * `position` - The position of the chunk in the terrain grid.
    /// * `state` - A reference to the `TerrainState`, which provides configuration for the terrain grid.
    ///
    /// # Returns
    /// A new `TerrainChunkComponent` initialized with:
    /// - Flat height values (0.0) for all tiles.
    /// - Default moisture values (0.0) for all tiles.
    /// - Grass biome as the default for all tiles.
    /// - An empty set of terrain features.
    pub fn new(position: IVec2, state: &TerrainState) -> Self {
        Self {
            position,
            // Initialize height map with zero elevation for all tiles.
            height_map: vec![0.0; state.chunk_tile_count()],
            // Initialize moisture map with zero moisture for all tiles.
            moisture_map: vec![0.0; state.chunk_tile_count()],
            // Initialize biome map with "Grass" as the default biome.
            biome_map: vec![BiomeType::Grass; state.chunk_tile_count()],
            // Initialize an empty set of terrain features.
            features: HashMap::new(),
        }
    }
}
