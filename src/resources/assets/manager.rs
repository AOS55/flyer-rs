use super::{AssetError, Result};
use std::any::Any;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AssetType {
    Texture,
    Model,
    Sound,
    Shader,
    Data,
}

struct AssetEntry {
    data: Arc<dyn Any + Send + Sync>,
    refs: usize,
    asset_type: AssetType,
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

        if let Some(entry) = assets.get_mut(id) {
            entry.refs += 1;
            return entry
                .data
                .downcast_ref::<Arc<T>>()
                .map(Arc::clone)
                .ok_or_else(|| AssetError::TypeMismatch(id.to_string()));
        }

        let path = self.resolve_path(id, asset_type)?;
        let data = Arc::new(loader(&path)?);

        assets.insert(
            id.to_string(),
            AssetEntry {
                data: Arc::new(data.clone()),
                refs: 1,
                asset_type,
            },
        );

        Ok(data)
    }

    pub fn unload(&self, id: &str) -> bool {
        let mut assets = self.assets.write().unwrap();
        if let Some(entry) = assets.get_mut(id) {
            entry.refs -= 1;
            if entry.refs == 0 {
                assets.remove(id);
                return true;
            }
        }
        false
    }

    fn resolve_path(&self, id: &str, asset_type: AssetType) -> Result<PathBuf> {
        let mut path = self.base_path.clone();
        path.push(match asset_type {
            AssetType::Texture => "textures",
            AssetType::Model => "models",
            AssetType::Sound => "sounds",
            AssetType::Shader => "shaders",
            AssetType::Data => "data",
        });
        path.push(id);

        if !path.exists() {
            return Err(AssetError::NotFound(id.to_string()));
        }
        Ok(path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_asset_loading() -> Result<()> {
        let temp_dir = TempDir::new().unwrap();
        fs::create_dir_all(temp_dir.path().join("textures")).unwrap();

        let test_data = b"test asset data".to_vec();
        let test_path = temp_dir.path().join("textures/test.dat");
        fs::write(&test_path, &test_data).unwrap();

        let manager = AssetManager::new(temp_dir.path());
        let loaded = manager.load("test.dat", AssetType::Texture, |path| Ok(fs::read(path)?))?;

        assert_eq!(*loaded, test_data);
        Ok(())
    }

    #[test]
    fn test_asset_not_found() {
        let temp_dir = TempDir::new().unwrap();
        let manager = AssetManager::new(temp_dir.path());

        let result = manager.load("nonexistent.dat", AssetType::Texture, |path| {
            Ok(fs::read(path)?)
        });

        assert!(matches!(result, Err(AssetError::NotFound(_))));
    }
}
