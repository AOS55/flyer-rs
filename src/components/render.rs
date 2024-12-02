use crate::ecs::component::Component;
use glam::{Vec2, Vec4};
use std::any::Any;

#[derive(Clone, Debug)]
pub struct RenderComponent {
    pub texture_id: String,
    pub position: Vec2,
    pub scale: Vec2,
    pub rotation: f32,
    pub layer: i32,
    pub tint: Vec4,
    pub visible: bool,
    pub src_rect: Option<[f32; 4]>,
    pub flip_x: bool,
    pub flip_y: bool,
}

#[derive(Clone, Debug)]
pub struct SpriteRenderData {
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<u8>,
    pub format: PixelFormat,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PixelFormat {
    RGBA8,
    RGB8,
    Gray8,
}

impl Default for RenderComponent {
    fn default() -> Self {
        Self {
            texture_id: String::new(),
            position: Vec2::ZERO,
            scale: Vec2::ONE,
            rotation: 0.0,
            layer: 0,
            tint: Vec4::ONE,
            visible: true,
            src_rect: None,
            flip_x: false,
            flip_y: false,
        }
    }
}

impl Component for RenderComponent {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
