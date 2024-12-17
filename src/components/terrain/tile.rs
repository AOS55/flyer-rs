use super::BiomeType;
use bevy::prelude::*;

/// Represents a single terrain tile in the game world.
/// A terrain tile is a smaller unit within a terrain chunk and contains:
/// - Biome type (e.g., grass, water, desert).
/// - 2D position in world space.
/// - Sprite index for rendering the correct tile appearance.
///
/// This component is typically attached to entities representing individual tiles
/// within a chunk, enabling fine-grained control over terrain rendering and behavior.
#[derive(Component, Debug, Clone)]
pub struct TerrainTileComponent {
    /// The biome type of the tile, which determines its visual and environmental properties.
    /// Example: `BiomeType::Grass`, `BiomeType::Water`, etc.
    pub biome_type: BiomeType,

    /// The position of the tile in world space (2D coordinates).
    /// This is used for placing the tile at the correct location in the game world.
    pub position: Vec2,

    /// The sprite index used to select the appropriate sprite for rendering this tile.
    /// This allows for different visual representations of tiles based on their biome or state.
    pub sprite_index: usize,
}
