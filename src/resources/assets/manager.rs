use super::{AssetError, Result};
use std::any::Any;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};
use tiny_skia::Pixmap;

pub enum AssetType {
    Texture,
    Model,
    Sound,
    Terrain,
}

struct AssetEntry {
    data: Arc<dyn Any + Send + Sync>,
    path: PathBuf,
    asset_type: AssetType,
    refs: usize,
}

pub struct AssetManager {
    assets: RwLock<HashMap<String, AssetEntry>>,
    base_path: PathBuf,
}

impl AssetManager {
    pub fn new<P: AsRef<Path>>(base_path: P) -> Self {
        Self {
            assets: RwLock::new(HashMap::new()),
            base_path: base_path.as_ref().to_path_buf(),
        }
    }

    pub fn load<T: 'static + Send + Sync>(
        &self,
        id: &str,
        asset_type: AssetType,
        loader: impl FnOnce(&Path) -> Result<T>,
    ) -> Result<Arc<T>> {
        let mut assets = self.assets.write().unwrap();

        // Check if asset already exists
        if let Some(entry) = assets.get_mut(id) {
            entry.refs += 1;
            if let Some(asset) = entry.data.downcast_ref::<Arc<T>>() {
                return Ok(Arc::clone(asset));
            }
        }

        // Load new asset
        let path = self.resolve_path(id, &asset_type)?;
        let data = loader(&path)?;
        let asset = Arc::new(data);

        // Store in assets map
        assets.insert(
            id.to_string(),
            AssetEntry {
                data: Arc::new(asset.clone()),
                path,
                asset_type,
                refs: 1,
            },
        );

        Ok(asset)
    }

    pub fn get_texture(&self, id: &str) -> Option<Arc<Pixmap>> {
        let assets = self.assets.read().unwrap();
        assets.get(id).and_then(|entry| {
            entry
                .data
                .downcast_ref::<Arc<Pixmap>>()
                .map(|pixmap| Arc::clone(pixmap))
        })
    }

    pub fn unload(&self, id: &str) {
        let mut assets = self.assets.write().unwrap();
        if let Some(entry) = assets.get_mut(id) {
            entry.refs -= 1;
            if entry.refs == 0 {
                assets.remove(id);
            }
        }
    }

    fn resolve_path(&self, id: &str, asset_type: &AssetType) -> Result<PathBuf> {
        let mut path = self.base_path.clone();

        match asset_type {
            AssetType::Texture => path.push("textures"),
            AssetType::Model => path.push("models"),
            AssetType::Sound => path.push("sounds"),
            AssetType::Terrain => path.push("terrain"),
        }

        path.push(id);

        if !path.exists() {
            return Err(AssetError::NotFound(id.to_string()));
        }

        Ok(path)
    }
}

impl Drop for AssetManager {
    fn drop(&mut self) {
        self.assets.write().unwrap().clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn setup_test_assets() -> (TempDir, AssetManager) {
        let temp_dir = TempDir::new().unwrap();
        fs::create_dir_all(temp_dir.path().join("textures")).unwrap();
        fs::create_dir_all(temp_dir.path().join("models")).unwrap();

        let manager = AssetManager::new(temp_dir.path());
        (temp_dir, manager)
    }

    #[test]
    fn test_asset_loading() {
        let (temp_dir, manager) = setup_test_assets();

        // Create a test file
        let test_file = temp_dir.path().join("textures/test.png");
        fs::write(&test_file, b"test data").unwrap();

        let result = manager.load("test.png", AssetType::Texture, |path| {
            Ok(Vec::from(fs::read(path)?))
        });

        assert!(result.is_ok());
    }

    #[test]
    fn test_asset_not_found() {
        let (_temp_dir, manager) = setup_test_assets();

        let result = manager.load("nonexistent.png", AssetType::Texture, |path| {
            Ok(Vec::from(fs::read(path)?))
        });

        assert!(matches!(result, Err(AssetError::NotFound(_))));
    }

    #[test]
    fn test_reference_counting() {
        let (temp_dir, manager) = setup_test_assets();

        // Create a test file
        let test_file = temp_dir.path().join("textures/test.png");
        fs::write(&test_file, b"test data").unwrap();

        // Load the asset twice
        let _asset1 = manager
            .load("test.png", AssetType::Texture, |path| {
                Ok(Vec::from(fs::read(path)?))
            })
            .unwrap();

        let _asset2 = manager
            .load("test.png", AssetType::Texture, |path| {
                Ok(Vec::from(fs::read(path)?))
            })
            .unwrap();

        // Check reference count
        assert_eq!(
            manager.assets.read().unwrap().get("test.png").unwrap().refs,
            2
        );

        // Unload once
        manager.unload("test.png");
        assert_eq!(
            manager.assets.read().unwrap().get("test.png").unwrap().refs,
            1
        );
    }
}
