use glam::Vec2;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::ecs::component::Component;
use tiny_skia::Pixmap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RenderComponent {
    pub mesh_type: MeshType,
    pub material: Material,
    pub transform: Transform2D,
    pub layer: RenderLayer,
    pub visible: bool,
    pub z_index: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MeshType {
    Sprite(SpriteData),
    Terrain(TerrainData),
    Custom(CustomMeshData),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpriteData {
    pub asset_id: String,
    pub dimensions: Vec2,
    pub pivot: Vec2,
    #[serde(skip)]
    pub pixmap: Option<Pixmap>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerrainData {
    pub tile_size: Vec2,
    pub tile_set: String,
    pub tiles: Vec<TileInstance>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomMeshData {
    pub vertices: Vec<Vec2>,
    pub indices: Vec<u16>,
    pub uvs: Vec<Vec2>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TileInstance {
    pub position: Vec2,
    pub tile_id: u32,
    pub rotation: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Material {
    pub color: Color,
    pub texture_path: Option<PathBuf>,
    pub blend_mode: BlendMode,
    pub shader: Option<Shader>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Transform2D {
    pub position: Vec2,
    pub scale: Vec2,
    pub rotation: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RenderLayer {
    Background,
    Terrain,
    Objects,
    Aircraft,
    Effects,
    UI,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BlendMode {
    Normal,
    Additive,
    Multiply,
    Screen,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Shader {
    Basic,
    Textured,
    Custom(String),
}

impl Default for RenderComponent {
    fn default() -> Self {
        Self {
            mesh_type: MeshType::Sprite(SpriteData {
                asset_id: String::new(),
                dimensions: Vec2::ONE,
                pivot: Vec2::new(0.5, 0.5),
                pixmap: None,
            }),
            material: Material::default(),
            transform: Transform2D::default(),
            layer: RenderLayer::Objects,
            visible: true,
            z_index: 0,
        }
    }
}

impl Default for Material {
    fn default() -> Self {
        Self {
            color: Color {
                r: 255,
                g: 255,
                b: 255,
                a: 255,
            },
            texture_path: None,
            blend_mode: BlendMode::Normal,
            shader: Some(Shader::Basic),
        }
    }
}

impl Default for Transform2D {
    fn default() -> Self {
        Self {
            position: Vec2::ZERO,
            scale: Vec2::ONE,
            rotation: 0.0,
        }
    }
}

impl Component for RenderComponent {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

impl RenderComponent {
    pub fn new_sprite(asset_id: String, dimensions: Vec2) -> Self {
        Self {
            mesh_type: MeshType::Sprite(SpriteData {
                asset_id,
                dimensions,
                pivot: Vec2::new(0.5, 0.5),
                pixmap: None,
            }),
            ..Default::default()
        }
    }

    pub fn new_terrain(tile_size: Vec2, tile_set: String) -> Self {
        Self {
            mesh_type: MeshType::Terrain(TerrainData {
                tile_size,
                tile_set,
                tiles: Vec::new(),
            }),
            layer: RenderLayer::Terrain,
            ..Default::default()
        }
    }

    pub fn set_position(&mut self, position: Vec2) {
        self.transform.position = position;
    }

    pub fn set_rotation(&mut self, rotation: f32) {
        self.transform.rotation = rotation;
    }

    pub fn set_scale(&mut self, scale: Vec2) {
        self.transform.scale = scale;
    }

    pub fn set_color(&mut self, r: u8, g: u8, b: u8, a: u8) {
        self.material.color = Color { r, g, b, a };
    }

    pub fn set_layer(&mut self, layer: RenderLayer) {
        self.layer = layer;
    }

    pub fn set_blend_mode(&mut self, mode: BlendMode) {
        self.material.blend_mode = mode;
    }

    pub fn set_visibility(&mut self, visible: bool) {
        self.visible = visible;
    }

    pub fn add_tile(&mut self, position: Vec2, tile_id: u32, rotation: f32) {
        if let MeshType::Terrain(terrain) = &mut self.mesh_type {
            terrain.tiles.push(TileInstance {
                position,
                tile_id,
                rotation,
            });
        }
    }

    pub fn clear_tiles(&mut self) {
        if let MeshType::Terrain(terrain) = &mut self.mesh_type {
            terrain.tiles.clear();
        }
    }

    pub fn get_transform_matrix(&self) -> [[f32; 3]; 3] {
        let cos_r = self.transform.rotation.cos();
        let sin_r = self.transform.rotation.sin();
        let sx = self.transform.scale.x;
        let sy = self.transform.scale.y;
        let tx = self.transform.position.x;
        let ty = self.transform.position.y;

        [
            [cos_r * sx, -sin_r * sy, tx],
            [sin_r * sx, cos_r * sy, ty],
            [0.0, 0.0, 1.0],
        ]
    }
}
