use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::SystemTime;

use crate::utils::errors::SimError;
use tiny_skia::Pixmap;

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub struct AssetId(String);

#[derive(Debug)]
pub enum Asset {
    Texture(Arc<Pixmap>),
    TerrainData(Arc<Vec<u8>>),
}

struct CacheEntry {
    asset: Asset,
    last_accessed: SystemTime,
    reference_count: usize,
}

pub struct AssetManager {
    cache: HashMap<AssetId, CacheEntry>,
    assets_path: PathBuf,
    terrain_data_path: PathBuf,
    max_cache_size: usize,
}

impl AssetManager {
    pub fn new(assets_path: &Path, terrain_data_path: &Path) -> Result<Self, SimError> {
        Ok(Self {
            cache: HashMap::new(),
            assets_path: assets_path.to_path_buf(),
            terrain_data_path: terrain_data_path.to_path_buf(),
            max_cache_size: 1000, // Default value
        })
    }

    pub fn set_assets_path(&mut self, path: PathBuf) -> Result<(), SimError> {
        if !path.exists() {
            return Err(SimError::AssetError(format!(
                "Asset path does not exist: {}",
                path.display()
            )));
        }
        self.assets_path = path;
        Ok(())
    }

    pub fn cache_asset(&mut self, id: AssetId, asset: Asset) {
        self.cleanup_cache();

        self.cache.insert(
            id,
            CacheEntry {
                asset,
                last_accessed: SystemTime::now(),
                reference_count: 1,
            },
        );
    }

    pub fn get_asset(&mut self, id: &AssetId) -> Option<&Asset> {
        if let Some(entry) = self.cache.get_mut(id) {
            entry.last_accessed = SystemTime::now();
            entry.reference_count += 1;
            Some(&entry.asset)
        } else {
            None
        }
    }

    pub fn release_asset(&mut self, id: &AssetId) {
        if let Some(entry) = self.cache.get_mut(id) {
            entry.reference_count = entry.reference_count.saturating_sub(1);
        }
    }

    fn cleanup_cache(&mut self) {
        if self.cache.len() >= self.max_cache_size {
            // Remove least recently used assets with no references
            let mut entries: Vec<_> = self
                .cache
                .iter()
                .filter(|(_, entry)| entry.reference_count == 0)
                .collect();

            entries.sort_by_key(|(_, entry)| entry.last_accessed);

            for (id, _) in entries
                .iter()
                .take(self.cache.len() - self.max_cache_size + 1)
            {
                self.cache.remove(id);
            }
        }
    }

    pub fn get_terrain(&mut self, name: &str) -> Result<Option<Arc<Vec<u8>>>, SimError> {
        let id = AssetId::new(format!("terrain_{}", name));

        if let Some(Asset::TerrainData(data)) = self.get_asset(&id).cloned() {
            Ok(Some(data))
        } else {
            Ok(None)
        }
    }

    pub fn cache_terrain(&mut self, name: &str, data: Arc<Vec<u8>>) -> Result<(), SimError> {
        let id = AssetId::new(format!("terrain_{}", name));
        self.cache_asset(id, Asset::TerrainData(data));
        Ok(())
    }
}

impl AssetId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }
}
