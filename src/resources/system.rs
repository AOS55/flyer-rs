use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use super::assets::{AssetManager, AssetType};
use super::config::SimulationConfig;
use super::errors::{ResourceError, Result};

pub struct ResourceSystemBuilder {
    base_path: Option<PathBuf>,
    config: Option<SimulationConfig>,
    default_resources: HashMap<TypeId, Box<dyn Any + Send + Sync>>,
}

impl ResourceSystemBuilder {
    pub fn new() -> Self {
        Self {
            base_path: None,
            config: None,
            default_resources: HashMap::new(),
        }
    }

    pub fn with_base_path<P: Into<PathBuf>>(mut self, path: P) -> Self {
        self.base_path = Some(path.into());
        self
    }

    pub fn with_config(mut self, config: SimulationConfig) -> Self {
        self.config = Some(config);
        self
    }

    pub fn with_resource<T: 'static + Send + Sync>(mut self, resource: T) -> Self {
        self.default_resources
            .insert(TypeId::of::<T>(), Box::new(resource));
        self
    }

    pub fn build(self) -> Result<ResourceSystem> {
        let config = self.config.unwrap_or_default();
        let base_path = self
            .base_path
            .clone()
            .unwrap_or_else(|| PathBuf::from("assets"));

        Ok(ResourceSystem {
            assets: AssetManager::new(base_path.clone()),
            resources: self.default_resources,
            config,
            base_path: Some(base_path),
        })
    }
}

pub struct ResourceSystem {
    assets: AssetManager,
    resources: HashMap<TypeId, Box<dyn Any + Send + Sync>>,
    config: SimulationConfig,
    base_path: Option<PathBuf>,
}

impl ResourceSystem {
    pub fn builder() -> ResourceSystemBuilder {
        ResourceSystemBuilder::new()
    }

    pub fn new() -> Result<Self> {
        Self::builder().build()
    }

    pub fn get<T: 'static>(&self) -> Result<&T> {
        self.resources
            .get(&TypeId::of::<T>())
            .and_then(|r| r.downcast_ref())
            .ok_or_else(|| ResourceError::NotFound(std::any::type_name::<T>().to_string()))
    }

    pub fn get_mut<T: 'static>(&mut self) -> Result<&mut T> {
        self.resources
            .get_mut(&TypeId::of::<T>())
            .and_then(|r| r.downcast_mut())
            .ok_or_else(|| ResourceError::NotFound(std::any::type_name::<T>().to_string()))
    }

    pub fn insert<T: 'static + Send + Sync>(&mut self, resource: T) -> Result<()> {
        let type_id = TypeId::of::<T>();
        if self.resources.contains_key(&type_id) {
            return Err(ResourceError::AlreadyExists(
                std::any::type_name::<T>().to_string(),
            ));
        }
        self.resources.insert(type_id, Box::new(resource));
        Ok(())
    }

    pub fn load_asset<T: 'static + Send + Sync>(
        &self,
        id: &str,
        asset_type: AssetType,
        loader: impl FnOnce(&std::path::Path) -> std::result::Result<T, super::assets::AssetError>,
    ) -> Result<Arc<T>> {
        self.assets
            .load(id, asset_type, loader)
            .map_err(ResourceError::Asset)
    }

    pub fn config(&self) -> &SimulationConfig {
        &self.config
    }

    pub fn config_mut(&mut self) -> &mut SimulationConfig {
        &mut self.config
    }

    pub fn reload_config(&mut self) -> Result<()> {
        match &self.base_path {
            Some(base_path) => {
                let config_path = base_path.join("config/simulation.yaml");
                self.config =
                    SimulationConfig::load(config_path.to_str().ok_or_else(|| {
                        ResourceError::Config("Invalid config path".to_string())
                    })?)?;
            }
            None => {
                // If no base_path, keep existing config or reset to default
                self.config = SimulationConfig::default();
            }
        }
        Ok(())
    }

    pub fn set_config(&mut self, config: SimulationConfig) {
        self.config = config;
    }

    pub fn load_config_from_path(&mut self, path: impl AsRef<std::path::Path>) -> Result<()> {
        self.config = SimulationConfig::load(
            path.as_ref()
                .to_str()
                .ok_or_else(|| ResourceError::Config("Invalid config path".to_string()))?,
        )?;
        Ok(())
    }
}
