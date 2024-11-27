use glam::Vec2;
use nalgebra::Vector3;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tiny_skia::Transform as SkiaTransform;
use tiny_skia::*;

use crate::rendering::types::{RenderConfig, RenderType};
use crate::utils::errors::SimError;
use crate::vehicles::Vehicle;
use crate::world::systems::camera::Camera;

/// Handles rendering of all vehicle types in different view modes
pub struct VehicleRenderer {
    /// Map of vehicle assets by type and variant
    vehicle_assets: HashMap<String, Pixmap>,
    /// Vehicle position history for trails
    position_history: HashMap<usize, VehicleTrail>,
}

/// Stores position history for rendering vehicle trails
struct VehicleTrail {
    positions: Vec<(f32, f32, f32)>, // x, y, z positions
    max_length: usize,
}

impl VehicleTrail {
    fn new(max_length: usize) -> Self {
        Self {
            positions: Vec::with_capacity(max_length),
            max_length,
        }
    }

    fn add_position(&mut self, x: f32, y: f32, z: f32) {
        self.positions.push((x, y, z));
        if self.positions.len() > self.max_length {
            self.positions.remove(0);
        }
    }
}

impl VehicleRenderer {
    pub fn new() -> Self {
        Self {
            vehicle_assets: HashMap::new(),
            position_history: HashMap::new(),
        }
    }

    pub fn load_assets(&mut self, asset_path: &Path) -> Result<(), SimError> {
        // Load standard vehicle assets
        let asset_types = ["t67h", "t67v"]; // Horizontal and vertical views

        for asset_type in asset_types {
            let mut path = PathBuf::from(asset_path);
            path.push(format!("{}.png", asset_type));

            match Pixmap::load_png(&path) {
                Ok(pixmap) => {
                    self.vehicle_assets.insert(asset_type.to_string(), pixmap);
                }
                Err(err) => {
                    return Err(SimError::AssetError(format!(
                        "Failed to load vehicle asset {}: {}",
                        path.display(),
                        err
                    )));
                }
            }
        }

        Ok(())
    }

    pub fn render_vehicles(
        &mut self,
        canvas: &mut Pixmap,
        vehicles: &[Box<dyn Vehicle>],
        camera: &Camera,
        config: &RenderConfig,
    ) -> Result<(), SimError> {
        match config.render_type {
            RenderType::World => self.render_world_view(canvas, vehicles, camera, config),
            RenderType::Aircraft => self.render_aircraft_view(canvas, vehicles, camera, config),
            RenderType::AircraftFixed => {
                self.render_fixed_aircraft_view(canvas, vehicles, camera, config)
            }
        }
    }

    fn render_world_view(
        &mut self,
        canvas: &mut Pixmap,
        vehicles: &[Box<dyn Vehicle>],
        camera: &Camera,
        config: &RenderConfig,
    ) -> Result<(), SimError> {
        let paint = PixmapPaint::default();
        let screen_center = Vec2::new(config.screen_dims.x / 2.0, config.screen_dims.y / 2.0);

        for (idx, vehicle) in vehicles.iter().enumerate() {
            let state = vehicle.get_state();

            // Calculate screen position
            let pos = Vector3::new(state.position.x, state.position.y, state.position.z);
            let screen_pos = self.world_to_screen_coords(pos, camera, screen_center, config.scale);

            // Get vehicle heading
            let (_, _, yaw) = state.attitude.euler_angles();

            // Draw vehicle sprite
            if let Some(sprite) = self.vehicle_assets.get("t67h") {
                let transform = SkiaTransform::from_row(
                    1.0,
                    0.0,
                    0.0,
                    1.0,
                    screen_pos.x - sprite.width() as f32 / 2.0,
                    screen_pos.y - sprite.height() as f32 / 2.0,
                );

                let transform = transform.post_rotate_at(
                    yaw as f32 * 180.0 / std::f32::consts::PI,
                    screen_pos.x,
                    screen_pos.y,
                );

                canvas.draw_pixmap(0, 0, sprite.as_ref(), &paint, transform, None);
            }

            // Update and render trail
            self.update_trail(
                idx,
                state.position.x as f32,
                state.position.y as f32,
                state.position.z as f32,
            );
            self.render_trail(canvas, idx, camera, screen_center, config);
        }

        Ok(())
    }

    fn render_aircraft_view(
        &mut self,
        canvas: &mut Pixmap,
        vehicles: &[Box<dyn Vehicle>],
        camera: &Camera,
        config: &RenderConfig,
    ) -> Result<(), SimError> {
        let paint = PixmapPaint::default();
        let screen_center = Vec2::new(config.screen_dims.x / 2.0, config.screen_dims.y / 2.0);
        let split_height = config.screen_dims.y * 0.66; // 2/3 of screen height

        // Draw split line
        let mut stroke = Stroke::default();
        stroke.width = 3.0;
        stroke.line_cap = LineCap::Round;

        let split_path = {
            let mut pb = PathBuilder::new();
            pb.move_to(0.0, split_height);
            pb.line_to(config.screen_dims.x, split_height);
            pb.finish().unwrap()
        };

        let mut split_paint = Paint::default();
        split_paint.set_color_rgba8(255, 255, 255, 200);
        split_paint.anti_alias = true;

        canvas.stroke_path(
            &split_path,
            &split_paint,
            &stroke,
            SkiaTransform::identity(),
            None,
        );

        // Render vehicle views
        for (idx, vehicle) in vehicles.iter().enumerate() {
            let state = vehicle.get_state();
            let (roll, pitch, yaw) = state.attitude.euler_angles();

            // Top view (horizontal)
            if let Some(sprite) = self.vehicle_assets.get("t67h") {
                let top_center = Vec2::new(screen_center.x, split_height * 0.5);

                let transform = SkiaTransform::from_row(
                    1.0,
                    0.0,
                    0.0,
                    1.0,
                    top_center.x - sprite.width() as f32 / 2.0,
                    top_center.y - sprite.height() as f32 / 2.0,
                );

                let transform = transform.post_rotate_at(
                    yaw as f32 * 180.0 / std::f32::consts::PI,
                    top_center.x,
                    top_center.y,
                );

                canvas.draw_pixmap(0, 0, sprite.as_ref(), &paint, transform, None);
            }

            // Bottom view (vertical)
            if let Some(sprite) = self.vehicle_assets.get("t67v") {
                let bottom_center = Vec2::new(
                    screen_center.x,
                    split_height + (config.screen_dims.y - split_height) * 0.5,
                );

                let transform = SkiaTransform::from_row(
                    1.0,
                    0.0,
                    0.0,
                    1.0,
                    bottom_center.x - sprite.width() as f32 / 2.0,
                    bottom_center.y - sprite.height() as f32 / 2.0,
                );

                let transform = transform.post_rotate_at(
                    -pitch as f32 * 180.0 / std::f32::consts::PI,
                    bottom_center.x,
                    bottom_center.y,
                );

                canvas.draw_pixmap(0, 0, sprite.as_ref(), &paint, transform, None);
            }

            // Update and render position trail
            self.update_trail(
                idx,
                state.position.x as f32,
                state.position.y as f32,
                state.position.z as f32,
            );
            self.render_split_trail(canvas, idx, camera, screen_center, split_height, config);
        }

        Ok(())
    }

    fn render_fixed_aircraft_view(
        &mut self,
        canvas: &mut Pixmap,
        vehicles: &[Box<dyn Vehicle>],
        camera: &Camera,
        config: &RenderConfig,
    ) -> Result<(), SimError> {
        let paint = PixmapPaint::default();
        let screen_center = Vec2::new(config.screen_dims.x / 2.0, config.screen_dims.y / 2.0);

        for vehicle in vehicles {
            let state = vehicle.get_state();
            let (_, _, yaw) = state.attitude.euler_angles();

            if let Some(sprite) = self.vehicle_assets.get("t67h") {
                let transform = SkiaTransform::from_row(
                    1.0,
                    0.0,
                    0.0,
                    1.0,
                    screen_center.x - sprite.width() as f32 / 2.0,
                    screen_center.y - sprite.height() as f32 / 2.0,
                );

                let transform = transform.post_rotate_at(
                    (yaw as f32 * 180.0 / std::f32::consts::PI) + 90.0,
                    screen_center.x,
                    screen_center.y,
                );

                canvas.draw_pixmap(0, 0, sprite.as_ref(), &paint, transform, None);
            }
        }

        Ok(())
    }

    fn update_trail(&mut self, vehicle_id: usize, x: f32, y: f32, z: f32) {
        let trail = self
            .position_history
            .entry(vehicle_id)
            .or_insert_with(|| VehicleTrail::new(400));
        trail.add_position(x, y, z);
    }

    fn render_trail(
        &self,
        canvas: &mut Pixmap,
        vehicle_id: usize,
        camera: &Camera,
        screen_center: Vec2,
        config: &RenderConfig,
    ) {
        if let Some(trail) = self.position_history.get(&vehicle_id) {
            let mut trace_paint = Paint::default();
            trace_paint.set_color_rgba8(255, 0, 0, 200);
            trace_paint.anti_alias = true;

            for pos in &trail.positions {
                let screen_pos = self.world_to_screen_coords(
                    Vector3::new(pos.0 as f64, pos.1 as f64, pos.2 as f64),
                    camera,
                    screen_center,
                    config.scale,
                );

                let point = PathBuilder::from_circle(screen_pos.x, screen_pos.y, 3.0).unwrap();
                canvas.fill_path(
                    &point,
                    &trace_paint,
                    FillRule::Winding,
                    SkiaTransform::identity(),
                    None,
                );
            }
        }
    }

    fn render_split_trail(
        &self,
        canvas: &mut Pixmap,
        vehicle_id: usize,
        camera: &Camera,
        screen_center: Vec2,
        split_height: f32,
        config: &RenderConfig,
    ) {
        if let Some(trail) = self.position_history.get(&vehicle_id) {
            let mut trace_paint = Paint::default();
            trace_paint.set_color_rgba8(255, 0, 0, 200);
            trace_paint.anti_alias = true;

            for (i, pos) in trail.positions.iter().enumerate() {
                // Top view (horizontal)
                let top_pos = Vec2::new(
                    screen_center.x - (camera.y as f32 - pos.1),
                    (split_height * 0.5) - (pos.0 - camera.x as f32),
                );

                if top_pos.y < split_height {
                    let point = PathBuilder::from_circle(top_pos.x, top_pos.y, 3.0).unwrap();
                    canvas.fill_path(
                        &point,
                        &trace_paint,
                        FillRule::Winding,
                        SkiaTransform::identity(),
                        None,
                    );
                }

                // Bottom view (vertical)
                let vertical_pos = Vec2::new(
                    screen_center.x - (i as f32),
                    (camera.z as f32 - pos.2)
                        + split_height
                        + ((config.screen_dims.y - split_height) * 0.5),
                );

                if vertical_pos.y > split_height {
                    let point =
                        PathBuilder::from_circle(vertical_pos.x, vertical_pos.y, 3.0).unwrap();
                    canvas.fill_path(
                        &point,
                        &trace_paint,
                        FillRule::Winding,
                        SkiaTransform::identity(),
                        None,
                    );
                }
            }
        }
    }

    fn world_to_screen_coords(
        &self,
        world_pos: Vector3<f64>,
        camera: &Camera,
        screen_center: Vec2,
        scale: f32,
    ) -> Vec2 {
        Vec2::new(
            (world_pos.x as f32 * scale) + screen_center.x,
            (world_pos.y as f32 * scale) + screen_center.y,
        )
    }
}
