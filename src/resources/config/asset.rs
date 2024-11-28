use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetConfig {
    pub base_path: PathBuf,
    pub models_path: PathBuf,
    pub textures_path: PathBuf,
}
