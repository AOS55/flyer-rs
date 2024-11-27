use glam::Vec2;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct Tile {
    pub name: String,
    pub asset: String,
    pub pos: Vec2,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct StaticObject {
    pub name: String,
    pub asset: String,
    pub pos: Vec2,
}

// Implementation of tile types
impl Tile {
    pub fn grass(pos: Vec2) -> Self {
        Self {
            name: "Grass".to_string(),
            asset: "grass".to_string(),
            pos,
        }
    }

    pub fn sand(pos: Vec2) -> Self {
        Self {
            name: "Sand".to_string(),
            asset: "sand".to_string(),
            pos,
        }
    }

    pub fn water(pos: Vec2) -> Self {
        Self {
            name: "Water".to_string(),
            asset: "water".to_string(),
            pos,
        }
    }
}
