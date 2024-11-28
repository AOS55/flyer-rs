use super::Result;
use std::path::Path;
use tiny_skia::Pixmap;

pub trait AssetLoader<T> {
    fn load(path: &Path) -> Result<T>;
}

pub struct TextureLoader;

impl AssetLoader<Pixmap> for TextureLoader {
    fn load(path: &Path) -> Result<Pixmap> {
        Pixmap::load_png(path).map_err(|e| super::AssetError::InvalidFormat(e.to_string()))
    }
}

pub struct TerrainLoader;

impl AssetLoader<Vec<u8>> for TerrainLoader {
    fn load(path: &Path) -> Result<Vec<u8>> {
        std::fs::read(path).map_err(Into::into)
    }
}
