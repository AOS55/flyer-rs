use crate::components::render::*;
use bevy::prelude::*;

pub fn sprite_layer_system(mut sprite_query: Query<(&RenderProperties, &mut Transform)>) {
    for (properties, mut transform) in sprite_query.iter_mut() {
        transform.translation.z = properties.layer as f32;
    }
}

pub fn sprite_animation_system(
    time: Res<Time>,
    mut query: Query<(&mut SpriteAnimation, &mut Sprite)>,
) {
    for (mut animation, mut sprite) in query.iter_mut() {
        animation.timer.tick(time.delta());
        if animation.timer.just_finished() {
            animation.current_frame = (animation.current_frame + 1) % animation.frames.len();
            if let Some(atlas) = &mut sprite.texture_atlas {
                atlas.index = animation.frames[animation.current_frame];
            }
        }
    }
}

pub fn update_sprite_properties(
    mut query: Query<(&RenderProperties, &mut Sprite), Changed<RenderProperties>>,
) {
    for (properties, mut sprite) in query.iter_mut() {
        sprite.color = properties.tint;
        sprite.flip_x = properties.flip_x;
        sprite.flip_y = properties.flip_y;
    }
}

pub fn spawn_animated_sprite(
    commands: &mut Commands,
    texture: Handle<Image>,
    atlas_layout: Handle<TextureAtlasLayout>,
    atlas_sources: &TextureAtlasSources,
    animation: SpriteAnimation,
    transform: Transform,
    properties: RenderProperties,
) -> Entity {
    let sprite = Sprite {
        color: properties.tint,
        flip_x: properties.flip_x,
        flip_y: properties.flip_y,
        image: texture.clone(),
        texture_atlas: Some(TextureAtlas {
            layout: atlas_layout,
            index: animation.frames[0],
        }),
        ..default()
    };

    commands
        .spawn((
            FlightSpriteBundle {
                sprite,
                render_properties: properties,
                global_transform: GlobalTransform::default(),
                inherited_visibility: InheritedVisibility::VISIBLE,
                view_visibility: ViewVisibility::default(),
            },
            animation,
        ))
        .id()
}
