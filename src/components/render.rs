use bevy::prelude::*;

#[derive(Component, Debug, Clone, Reflect)]
#[reflect(Component)]
pub struct RenderProperties {
    pub layer: i32,
    pub tint: Color,
    pub flip_x: bool,
    pub flip_y: bool,
}

impl Default for RenderProperties {
    fn default() -> Self {
        Self {
            layer: 0,
            tint: Color::WHITE,
            flip_x: false,
            flip_y: false,
        }
    }
}

#[derive(Component, Debug, Clone, Reflect)]
#[reflect(Component)]
pub struct SpriteAnimation {
    pub timer: Timer,
    pub frames: Vec<usize>,
    pub current_frame: usize,
    pub row: usize,
    pub frame_size: Vec2,
}

#[derive(Bundle)]
pub struct FlightSpriteBundle {
    pub sprite: Sprite,
    pub render_properties: RenderProperties,
    pub global_transform: GlobalTransform,
    pub inherited_visibility: InheritedVisibility,
    pub view_visibility: ViewVisibility,
}

impl FlightSpriteBundle {
    pub fn new(
        _texture: Handle<Image>,
        _atlas_layout: Handle<TextureAtlasLayout>,
        _animation_frame: usize,
        _transform: Transform,
        properties: RenderProperties,
    ) -> Self {
        Self {
            sprite: Sprite {
                color: properties.tint,
                custom_size: Some(Vec2::new(32.0, 32.0)), // Example size
                flip_x: properties.flip_x,
                flip_y: properties.flip_y,
                ..Default::default()
            },
            render_properties: properties,
            global_transform: GlobalTransform::default(),
            inherited_visibility: InheritedVisibility::VISIBLE,
            view_visibility: ViewVisibility::default(),
        }
    }
}
