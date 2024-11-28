use super::{AssetError, Result};
use std::any::Any;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};

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

    pub fn load<T: 'static + Send + Sync + Clone>(
        &self,
        id: &str,
        asset_type: AssetType,
        loader: impl FnOnce(&Path) -> Result<T>,
    ) -> Result<Arc<T>> {
        let mut assets = self.assets.write().unwrap();

        if let Some(entry) = assets.get_mut(id) {
            entry.refs += 1;
            if let Some(asset) = entry.data.downcast_ref::<T>() {
                return Ok(Arc::new(asset.clone()));
            }
        }

        let path = self.resolve_path(id, &asset_type)?;
        let data = loader(&path)?;
        let asset = Arc::new(data);

        assets.insert(
            id.to_string(),
            AssetEntry {
                data: asset.clone(),
                path,
                asset_type,
                refs: 1,
            },
        );

        Ok(asset)
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
