use crate::environment::runway::Runway;
use crate::environment::terrain::{StaticObject, Terrain, Tile};
use crate::rendering::types::RenderConfig;
use crate::utils::errors::SimError;
use crate::world::camera::Camera;

use glam::Vec2;
use rayon::prelude::*;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use tiny_skia::*;

pub struct TerrainRenderer {
    tile_map: HashMap<String, Pixmap>,
    object_map: HashMap<String, Pixmap>,
}

impl TerrainRenderer {
    pub fn new() -> Self {
        Self {
            tile_map: HashMap::new(),
            object_map: HashMap::new(),
        }
    }

    pub fn load_assets(&mut self, assets_dir: &Path) -> Result<(), SimError> {
        // Load tile assets
        let mut tile_path = PathBuf::from(assets_dir);
        tile_path.push("tiles");

        let tile_dir: Vec<_> = match fs::read_dir(&tile_path) {
            Ok(td) => td.filter_map(|entry| Some(entry.ok()?.path())).collect(),
            Err(_) => {
                return Err(SimError::AssetError(format!(
                    "Tiles directory not found: {}",
                    tile_path.display()
                )));
            }
        };

        // Load object assets
        let mut object_path = PathBuf::from(assets_dir);
        object_path.push("objects");

        let object_dir: Vec<_> = match fs::read_dir(&object_path) {
            Ok(od) => od.filter_map(|entry| Some(entry.ok()?.path())).collect(),
            Err(_) => {
                return Err(SimError::AssetError(format!(
                    "Objects directory not found: {}",
                    object_path.display()
                )));
            }
        };

        // Create asset maps
        self.tile_map = self.create_asset_map(tile_dir)?;
        self.object_map = self.create_asset_map(object_dir)?;

        Ok(())
    }

    fn create_asset_map(&self, paths: Vec<PathBuf>) -> Result<HashMap<String, Pixmap>, SimError> {
        let mut asset_map = HashMap::new();

        for path in paths {
            let path_str = path.to_str().unwrap_or_default().to_string();
            match Pixmap::load_png(&path) {
                Ok(pixmap) => {
                    let name = path
                        .file_stem()
                        .and_then(|n| n.to_str())
                        .ok_or_else(|| {
                            SimError::AssetError(format!("Invalid filename: {}", path.display()))
                        })?
                        .to_string();
                    asset_map.insert(name, pixmap);
                }
                Err(err) => {
                    return Err(SimError::AssetError(format!(
                        "Failed to load PNG {}: {}",
                        path_str, err
                    )));
                }
            }
        }

        Ok(asset_map)
    }

    pub fn render(
        &self,
        canvas: &mut Pixmap,
        terrain: &Terrain,
        runway: Option<&Runway>,
        camera: &Camera,
        config: &RenderConfig,
    ) -> Result<(), SimError> {
        let paint = PixmapPaint::default();
        let center = Vec2::new(
            camera.x as f32 + config.origin.x,
            camera.y as f32 + config.origin.y,
        );

        let reconstruction_ratio = camera.f * camera.z as f32;
        let scaling_ratio = Vec2::new(
            config.screen_dims.x / reconstruction_ratio,
            config.screen_dims.y / reconstruction_ratio,
        );

        // Render terrain tiles
        let tile_results: Vec<(Pixmap, Transform)> = terrain
            .tiles
            .par_iter()
            .filter_map(|tile| {
                self.process_tile(
                    tile,
                    center,
                    config.screen_dims,
                    scaling_ratio,
                    config.scale,
                )
            })
            .collect();

        for (pixmap, transform) in tile_results {
            canvas.draw_pixmap(0, 0, pixmap.as_ref(), &paint, transform, None);
        }

        // Render static objects
        let object_results: Vec<(Pixmap, Transform)> = terrain
            .objects
            .par_iter()
            .filter_map(|object| {
                self.process_object(
                    object,
                    center,
                    config.screen_dims,
                    scaling_ratio,
                    config.scale,
                )
            })
            .collect();

        for (pixmap, transform) in object_results {
            canvas.draw_pixmap(0, 0, pixmap.as_ref(), &paint, transform, None);
        }

        // Render runway if present
        if let Some(runway) = runway {
            self.render_runway(canvas, runway, center, config.screen_dims, scaling_ratio)?;
        }

        Ok(())
    }

    fn process_tile(
        &self,
        tile: &Tile,
        center: Vec2,
        screen_dims: Vec2,
        scaling_ratio: Vec2,
        scale: f32,
    ) -> Option<(Pixmap, Transform)> {
        let pos = Vec2::new(tile.pos[0] - center[0], tile.pos[1] - center[1]);
        let pix_pos = pos * scaling_ratio;
        let pix_pos = pix_pos + screen_dims / 2.0;
        let scale = scale * scaling_ratio;

        if self.is_visible(pix_pos, screen_dims) {
            let tile_pixmap = self.tile_map.get(&tile.asset)?;
            let transform = Transform::from_row(
                scale[0] / 16.0,
                0.0,
                0.0,
                scale[1] / 16.0,
                pix_pos[0],
                pix_pos[1],
            );
            Some((tile_pixmap.clone(), transform))
        } else {
            None
        }
    }

    fn process_object(
        &self,
        object: &StaticObject,
        center: Vec2,
        screen_dims: Vec2,
        scaling_ratio: Vec2,
        scale: f32,
    ) -> Option<(Pixmap, Transform)> {
        let pos = Vec2::new(object.pos[0] - center[0], object.pos[1] - center[1]);
        let pix_pos = pos * scaling_ratio;
        let pix_pos = pix_pos + screen_dims / 2.0;
        let scale = scale * scaling_ratio;

        if self.is_visible(pix_pos, screen_dims) {
            let object_pixmap = self.object_map.get(&object.asset)?;
            let transform = Transform::from_row(
                scale[0] / 16.0,
                0.0,
                0.0,
                scale[1] / 16.0,
                pix_pos[0],
                pix_pos[1],
            );
            Some((object_pixmap.clone(), transform))
        } else {
            None
        }
    }

    fn render_runway(
        &self,
        canvas: &mut Pixmap,
        runway: &Runway,
        center: Vec2,
        screen_dims: Vec2,
        scaling_ratio: Vec2,
    ) -> Result<(), SimError> {
        let paint = PixmapPaint::default();
        let screen_center = screen_dims / 2.0;

        let mut runway_corner = runway.pos - (runway.dims / 2.0);
        runway_corner -= Vec2::new(center.x, center.y);

        let pix_pos_corner = runway_corner * scaling_ratio + screen_center;

        let mut runway_center = runway.pos;
        runway_center -= Vec2::new(center.x, center.y);

        let pix_pos_center = runway_center * scaling_ratio + screen_center;

        let scale = Vec2::new(
            scaling_ratio.x * (runway.dims.x / 33.0),
            scaling_ratio.y * (runway.dims.y / 1500.0),
        );

        if let Some(runway_pixmap) = self.object_map.get(&runway.asset) {
            let transform = Transform::from_row(
                scale.x,
                0.0,
                0.0,
                scale.y,
                pix_pos_corner.x,
                pix_pos_corner.y,
            );
            let transform =
                transform.post_rotate_at(90.0 + runway.heading, pix_pos_center.x, pix_pos_center.y);
            canvas.draw_pixmap(0, 0, runway_pixmap.as_ref(), &paint, transform, None);
        }

        Ok(())
    }

    fn is_visible(&self, pix_pos: Vec2, screen_dims: Vec2) -> bool {
        const MARGIN: f32 = 50.0;
        (-MARGIN < pix_pos.x)
            && (-MARGIN < pix_pos.y)
            && (pix_pos.x < screen_dims.x + MARGIN)
            && (pix_pos.y < screen_dims.y + MARGIN)
    }

    pub fn render_terrain(
        &self,
        canvas: &mut Pixmap,
        terrain: &Terrain,
        camera: &Camera,
        config: &RenderConfig,
    ) -> Result<(), SimError> {
        let paint = PixmapPaint::default();
        let center = Vec2::new(
            camera.x as f32 + config.screen_dims.x / 2.0,
            camera.y as f32 + config.screen_dims.y / 2.0,
        );

        let scaling_ratio = Vec2::new(
            config.screen_dims.x / (terrain.width() as f32 * config.scale),
            config.screen_dims.y / (terrain.height() as f32 * config.scale),
        );

        // Render terrain tiles
        for tile in &terrain.tiles {
            let pos = Vec2::new(tile.pos.x - center.x, tile.pos.y - center.y);
            let pix_pos = pos * scaling_ratio + config.screen_dims / 2.0;
            let scale = config.scale * scaling_ratio;

            if self.is_visible(pix_pos, config.screen_dims) {
                if let Some(tile_pixmap) = self.tile_map.get(&tile.asset) {
                    let transform = Transform::from_row(
                        scale.x / 16.0,
                        0.0,
                        0.0,
                        scale.y / 16.0,
                        pix_pos.x,
                        pix_pos.y,
                    );
                    canvas.draw_pixmap(0, 0, tile_pixmap.as_ref(), &paint, transform, None);
                }
            }
        }

        // Render static objects
        for object in &terrain.objects {
            let pos = Vec2::new(object.pos.x - center.x, object.pos.y - center.y);
            let pix_pos = pos * scaling_ratio + config.screen_dims / 2.0;
            let scale = config.scale * scaling_ratio;

            if self.is_visible(pix_pos, config.screen_dims) {
                if let Some(object_pixmap) = self.object_map.get(&object.asset) {
                    let transform = Transform::from_row(
                        scale.x / 16.0,
                        0.0,
                        0.0,
                        scale.y / 16.0,
                        pix_pos.x,
                        pix_pos.y,
                    );
                    canvas.draw_pixmap(0, 0, object_pixmap.as_ref(), &paint, transform, None);
                }
            }
        }

        Ok(())
    }
}

impl Default for TerrainRenderer {
    fn default() -> Self {
        Self::new()
    }
}
