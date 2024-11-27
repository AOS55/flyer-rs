use super::manager::{Asset, AssetId, AssetManager};
use crate::utils::errors::SimError;
use std::path::PathBuf;
use std::sync::Arc;

pub trait AssetLoader {
    fn load_asset(&self, path: &PathBuf) -> Result<Asset, SimError>;
}

pub struct TextureLoader;
impl AssetLoader for TextureLoader {
    fn load_asset(&self, path: &PathBuf) -> Result<Asset, SimError> {
        let pixmap = tiny_skia::Pixmap::load_png(path)
            .map_err(|e| SimError::AssetError(format!("Failed to load texture: {}", e)))?;
        Ok(Asset::Texture(Arc::new(pixmap)))
    }
}

pub struct TerrainDataLoader;
impl AssetLoader for TerrainDataLoader {
    fn load_asset(&self, path: &PathBuf) -> Result<Asset, SimError> {
        let data = std::fs::read(path)
            .map_err(|e| SimError::AssetError(format!("Failed to load terrain data: {}", e)))?;
        Ok(Asset::TerrainData(Arc::new(data)))
    }
}

impl AssetManager {
    pub fn load_with_loader<L: AssetLoader>(
        &mut self,
        loader: &L,
        path: &PathBuf,
        id: AssetId,
    ) -> Result<(), SimError> {
        let asset = loader.load_asset(path)?;
        self.cache_asset(id, asset);
        Ok(())
    }

    pub fn load_texture(&mut self, path: &PathBuf, id: AssetId) -> Result<(), SimError> {
        self.load_with_loader(&TextureLoader, path, id)
    }

    pub fn load_terrain_data(&mut self, path: &PathBuf, id: AssetId) -> Result<(), SimError> {
        self.load_with_loader(&TerrainDataLoader, path, id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn test_texture_loading() {
        let loader = TextureLoader;
        let path = Path::new("test_assets/texture.png").to_path_buf();
        let result = loader.load_asset(&path);
        assert!(result.is_err()); // Test asset doesn't exist
    }
}
