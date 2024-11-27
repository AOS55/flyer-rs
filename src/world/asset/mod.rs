mod manager;

pub use manager::{Asset, AssetId, AssetManager};

use crate::utils::errors::SimError;
use std::path::PathBuf;

/// Trait for types that can be loaded as assets
pub trait LoadableAsset: Sized {
    fn load(path: &PathBuf) -> Result<Self, SimError>;
}

/// Trait for types that manage their own asset loading
pub trait AssetLoader {
    fn load_assets(&mut self, asset_manager: &mut AssetManager) -> Result<(), SimError>;
}
