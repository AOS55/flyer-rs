use glam::Vec2;
use std::collections::HashMap;
use std::path::Path;
use tiny_skia::*;

use crate::environment::Terrain;
use crate::rendering::types::{RenderConfig, RenderState, RenderType};
use crate::rendering::{TerrainRenderer, VehicleRenderer};
use crate::utils::errors::SimError;
use crate::vehicles::Vehicle;
use crate::world::systems::camera::Camera;

pub struct Renderer {
    config: RenderConfig,
    state: RenderState,
    terrain_renderer: TerrainRenderer,
    vehicle_renderer: VehicleRenderer,
    position_log: Vec<[f32; 3]>, // Store position history for trails
}

impl Renderer {
    pub fn new(config: RenderConfig) -> Result<Self, SimError> {
        let state = RenderState {
            origin: Vec2::new(
                config.scale * (config.screen_dims.x / 2.0),
                config.scale * (config.screen_dims.y / 2.0),
            ),
            canvas: None,
            asset_map: HashMap::new(),
        };

        Ok(Self {
            config,
            state,
            terrain_renderer: TerrainRenderer::new(),
            vehicle_renderer: VehicleRenderer::new(),
            position_log: Vec::with_capacity(400), // Pre-allocate position history
        })
    }

    pub fn load_assets(&mut self, asset_dir: &Path) -> Result<(), SimError> {
        // Ensure asset directory exists
        if !asset_dir.exists() {
            return Err(SimError::AssetError(format!(
                "Asset directory not found: {}",
                asset_dir.display()
            )));
        }

        // Load terrain assets
        let mut terrain_path = asset_dir.to_path_buf();
        terrain_path.push("tiles");
        self.terrain_renderer.load_assets(&terrain_path)?;

        // Load vehicle assets
        let mut vehicle_path = asset_dir.to_path_buf();
        vehicle_path.push("vehicles");
        self.vehicle_renderer.load_assets(&vehicle_path)?;

        Ok(())
    }

    pub fn render(
        &mut self,
        terrain: &Terrain,
        vehicles: &[Box<dyn Vehicle>],
        camera: &Camera,
    ) -> Result<Pixmap, SimError> {
        match self.config.render_type {
            RenderType::World => self.render_world(terrain, vehicles, camera),
            RenderType::Aircraft => self.render_aircraft_view(vehicles, camera),
            RenderType::AircraftFixed => self.render_fixed_aircraft(vehicles, camera),
        }
    }

    fn render_world(
        &mut self,
        terrain: &Terrain,
        vehicles: &[Box<dyn Vehicle>],
        camera: &Camera,
    ) -> Result<Pixmap, SimError> {
        let mut canvas = self.create_canvas()?;
        let paint = PixmapPaint::default();

        // Render terrain first
        self.terrain_renderer
            .render_terrain(&mut canvas, terrain, camera, &self.config)?;

        // Render runway if present
        if let Some(runway) = terrain.get_runway() {
            self.render_runway(&mut canvas, runway, camera)?;
        }

        // Render vehicles
        self.vehicle_renderer
            .render_vehicles(&mut canvas, vehicles, camera, &self.config)?;

        Ok(canvas)
    }

    fn render_aircraft_view(
        &mut self,
        vehicles: &[Box<dyn Vehicle>],
        camera: &Camera,
    ) -> Result<Pixmap, SimError> {
        let mut canvas = self.create_canvas()?;
        let pixmap_paint = PixmapPaint::default();

        // Split screen setup
        let split_fraction = 0.66;
        let split_point = self.config.screen_dims.y * split_fraction;
        self.draw_split_line(&mut canvas, split_point)?;

        // Get screen centers
        let horizontal_center = Vec2::new(
            self.config.screen_dims.x / 2.0,
            (split_fraction / 2.0) * self.config.screen_dims.y,
        );
        let vertical_center = Vec2::new(
            self.config.screen_dims.x / 2.0,
            (1.0 - ((1.0 - split_fraction) / 2.0)) * self.config.screen_dims.y,
        );

        // Update position log
        self.update_position_log(camera);

        // Draw position traces
        self.draw_position_traces(&mut canvas, split_point, horizontal_center, vertical_center)?;

        // Draw aircraft views
        if let Some(vehicle) = vehicles.first() {
            self.draw_aircraft_views(&mut canvas, vehicle, horizontal_center, vertical_center)?;
        }

        Ok(canvas)
    }

    fn render_fixed_aircraft(
        &mut self,
        vehicles: &[Box<dyn Vehicle>],
        camera: &Camera,
    ) -> Result<Pixmap, SimError> {
        let mut canvas = self.create_canvas()?;
        let pixmap_paint = PixmapPaint::default();

        let screen_center = Vec2::new(
            self.config.screen_dims.x / 2.0,
            self.config.screen_dims.y / 2.0,
        );

        // Calculate scaling based on area
        let scale = Vec2::new(
            self.config.screen_dims.x / (self.terrain_renderer.get_area()[0] as f32 * 16.0),
            self.config.screen_dims.y / (self.terrain_renderer.get_area()[1] as f32 * 16.0),
        );

        // Draw vehicles in fixed view
        if let Some(vehicle) = vehicles.first() {
            self.draw_fixed_aircraft(&mut canvas, vehicle, camera, screen_center, scale)?;
        }

        // Draw goal point if present
        if let Some(goal) = self.state.goal {
            self.draw_goal_point(&mut canvas, goal, screen_center, scale)?;
        }

        Ok(canvas)
    }

    // Helper methods
    fn create_canvas(&self) -> Result<Pixmap, SimError> {
        Pixmap::new(
            self.config.screen_dims.x as u32,
            self.config.screen_dims.y as u32,
        )
        .ok_or_else(|| SimError::RenderError("Failed to create canvas".into()))
    }

    fn draw_split_line(&self, canvas: &mut Pixmap, split_point: f32) -> Result<(), SimError> {
        let split_path = {
            let mut pb = PathBuilder::new();
            pb.move_to(0.0, split_point);
            pb.line_to(self.config.screen_dims.x, split_point);
            pb.finish().unwrap()
        };

        let mut stroke = Stroke::default();
        stroke.width = 3.0;
        stroke.line_cap = LineCap::Round;

        let mut split_paint = Paint::default();
        split_paint.set_color_rgba8(255, 255, 255, 200);
        split_paint.anti_alias = true;

        canvas.stroke_path(
            &split_path,
            &split_paint,
            &stroke,
            Transform::identity(),
            None,
        );

        Ok(())
    }

    fn update_position_log(&mut self, camera: &Camera) {
        self.position_log
            .push([camera.x as f32, camera.y as f32, camera.z as f32]);

        // Maintain fixed history size
        if self.position_log.len() > 400 {
            self.position_log.remove(0);
        }
    }

    fn draw_position_traces(
        &self,
        canvas: &mut Pixmap,
        split_point: f32,
        horizontal_center: Vec2,
        vertical_center: Vec2,
    ) -> Result<(), SimError> {
        let mut trace_paint = Paint::default();
        trace_paint.set_color_rgba8(255, 0, 0, 200);
        trace_paint.anti_alias = true;

        for (idx, pos) in self.position_log.iter().enumerate() {
            let idt = idx as f32;

            // Draw horizontal trace
            let horizontal_path_x = horizontal_center.x - (self.camera.y as f32 - pos[1]);
            let horizontal_path_y = horizontal_center.y - (pos[0] - self.camera.x as f32);

            if horizontal_path_y < split_point {
                let horizontal_point =
                    PathBuilder::from_circle(horizontal_path_x, horizontal_path_y, 3.0).unwrap();
                canvas.fill_path(
                    &horizontal_point,
                    &trace_paint,
                    FillRule::Winding,
                    Transform::identity(),
                    None,
                );
            }

            // Draw vertical trace
            let vertical_path_y = (self.camera.z as f32 - pos[2]) + vertical_center.y;
            if vertical_path_y > split_point {
                let vertical_point =
                    PathBuilder::from_circle(vertical_center.x - idt, vertical_path_y, 3.0)
                        .unwrap();
                canvas.fill_path(
                    &vertical_point,
                    &trace_paint,
                    FillRule::Winding,
                    Transform::identity(),
                    None,
                );
            }
        }

        Ok(())
    }

    fn draw_goal_point(
        &self,
        canvas: &mut Pixmap,
        goal: [f32; 3],
        screen_center: Vec2,
        scale: Vec2,
    ) -> Result<(), SimError> {
        let goal_pix_x = (goal[0] * scale.x) + screen_center.x;
        let goal_pix_y = (goal[1] * scale.y) + screen_center.y;

        let mut goal_paint = Paint::default();
        goal_paint.set_color_rgba8(0, 255, 0, 200);
        goal_paint.anti_alias = true;

        let goal_point = PathBuilder::from_circle(goal_pix_x, goal_pix_y, 10.0).unwrap();
        canvas.fill_path(
            &goal_point,
            &goal_paint,
            FillRule::Winding,
            Transform::identity(),
            None,
        );

        Ok(())
    }
}
