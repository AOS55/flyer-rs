use glam::{Vec2, Vec3};
use tiny_skia::{FillRule, Paint, PathBuilder, Pixmap, Transform as SkiaTransform};

use crate::components::{CameraComponent, RenderComponent, SpatialComponent};
use crate::ecs::{component::ComponentQuery, system::System, world::World};
use crate::resources::{assets::AssetManager, config::RenderConfig};
use crate::utils::errors::SimError;

pub struct RenderSystem {
    canvas: Option<Pixmap>,
    base_paint: Paint,
    position_history: Vec<(f32, f32, f32)>,
}

impl RenderSystem {
    pub fn new() -> Self {
        let mut base_paint = Paint::default();
        base_paint.set_color_rgba8(255, 255, 255, 255);
        base_paint.anti_alias = true;

        Self {
            canvas: None,
            base_paint,
            position_history: Vec::with_capacity(400),
        }
    }

    fn initialize_canvas(&mut self, config: &RenderConfig) -> Result<(), SimError> {
        self.canvas = Some(
            Pixmap::new(config.screen_dims.x as u32, config.screen_dims.y as u32)
                .ok_or_else(|| SimError::RenderError("Failed to create canvas".into()))?,
        );
        Ok(())
    }

    fn render_entity(
        &mut self,
        spatial: &SpatialComponent,
        render: &RenderComponent,
        camera: &CameraComponent,
        config: &RenderConfig,
        assets: &AssetManager,
    ) -> Result<(), SimError> {
        let canvas = self.canvas.as_mut().unwrap();
        let screen_center = Vec2::new(config.screen_dims.x / 2.0, config.screen_dims.y / 2.0);

        let world_pos = Vec3::new(
            spatial.position.x as f32,
            spatial.position.y as f32,
            spatial.position.z as f32,
        );

        let screen_pos = self.world_to_screen(world_pos, camera, screen_center, config.scale);

        if let Some(sprite) = assets.get_sprite(&render.mesh_id) {
            let transform = SkiaTransform::from_row(
                1.0,
                0.0,
                0.0,
                1.0,
                screen_pos.x - sprite.width() as f32 / 2.0,
                screen_pos.y - sprite.height() as f32 / 2.0,
            );

            let (roll, pitch, yaw) = spatial.attitude.euler_angles();
            let transform = transform.post_rotate_at(
                yaw as f32 * 180.0 / std::f32::consts::PI,
                screen_pos.x,
                screen_pos.y,
            );

            canvas.draw_pixmap(0, 0, sprite.as_ref(), &self.base_paint, transform, None);

            self.update_position_history(world_pos);
            self.render_trail(canvas, camera, screen_center, config)?;
        }

        Ok(())
    }

    fn world_to_screen(
        &self,
        world_pos: Vec3,
        camera: &CameraComponent,
        screen_center: Vec2,
        scale: f32,
    ) -> Vec2 {
        Vec2::new(
            (world_pos.x * scale) + screen_center.x,
            (world_pos.y * scale) + screen_center.y,
        )
    }

    fn update_position_history(&mut self, pos: Vec3) {
        self.position_history.push((pos.x, pos.y, pos.z));
        if self.position_history.len() > 400 {
            self.position_history.remove(0);
        }
    }

    fn render_trail(
        &self,
        canvas: &mut Pixmap,
        camera: &CameraComponent,
        screen_center: Vec2,
        config: &RenderConfig,
    ) -> Result<(), SimError> {
        let mut trail_paint = Paint::default();
        trail_paint.set_color_rgba8(255, 0, 0, 200);
        trail_paint.anti_alias = true;

        for pos in &self.position_history {
            let screen_pos = self.world_to_screen(
                Vec3::new(pos.0, pos.1, pos.2),
                camera,
                screen_center,
                config.scale,
            );

            let point = PathBuilder::from_circle(screen_pos.x, screen_pos.y, 3.0).unwrap();
            canvas.fill_path(
                &point,
                &trail_paint,
                FillRule::Winding,
                SkiaTransform::identity(),
                None,
            );
        }

        Ok(())
    }
}

impl System for RenderSystem {
    fn update(&mut self, world: &mut World, _dt: f64) {
        let config = match world.get_resource::<RenderConfig>() {
            Some(config) => config,
            None => return,
        };

        let assets = match world.get_resource::<AssetManager>() {
            Some(assets) => assets,
            None => return,
        };

        if self.canvas.is_none() {
            if let Err(e) = self.initialize_canvas(config) {
                eprintln!("Failed to initialize canvas: {}", e);
                return;
            }
        }

        let camera = match world.query_unique::<&CameraComponent>() {
            Some(camera) => camera,
            None => return,
        };

        let mut query = world.query::<(&SpatialComponent, &RenderComponent)>();
        for (spatial, render) in query.iter() {
            if render.visibility {
                if let Err(e) = self.render_entity(spatial, render, camera, config, assets) {
                    eprintln!("Failed to render entity: {}", e);
                }
            }
        }
    }

    fn cleanup(&mut self) {
        self.canvas = None;
        self.position_history.clear();
    }
}
