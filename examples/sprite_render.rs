use bevy::prelude::*;

use flyer::plugins::terrain::{TerrainPlugin, TerrainPluginConfig};

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(ImagePlugin::default_nearest()),
            TerrainPlugin::new(TerrainPluginConfig {
                world_size: IVec2::new(1000, 1000),
                chunk_size: 32,
                seed: 42,
                scale: 16.0, // Adjusted to match tile scale
                max_concurrent_chunks: 20,
            }),
        ))
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                aircraft_movement,
                animate_aircraft,
                input_aircraft,
                update_camera,
            ),
        )
        .run();
}

#[derive(Component)]
struct MainCamera;

#[derive(Component)]
struct AnimationIndices {
    bank_left: usize,
    bank_right: usize,
    straight: usize,
}

#[derive(Component)]
struct Aircraft {
    heading: f32,
    position: Vec2,
    bank_angle: f32,
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    commands.spawn((Camera2d::default(), MainCamera));

    let texture = asset_server.load("sprites/malibu_sheet.png");
    let layout = TextureAtlasLayout::from_grid(UVec2::splat(128), 3, 3, None, None);
    let texture_atlas_layout = texture_atlas_layouts.add(layout);
    let animation_indices = AnimationIndices {
        bank_left: 1,
        bank_right: 7,
        straight: 4,
    };
    commands.spawn((
        Sprite::from_atlas_image(
            texture,
            TextureAtlas {
                layout: texture_atlas_layout,
                index: animation_indices.straight,
            },
        ),
        Transform {
            translation: Vec3::new(0.0, 0.0, 10.0), // Higher Z value to be above terrain
            rotation: Quat::from_rotation_z(-std::f32::consts::FRAC_PI_2),
            scale: Vec3::splat(0.5),
        },
        animation_indices,
        Aircraft {
            heading: -std::f32::consts::FRAC_PI_2,
            position: Vec2::ZERO,
            bank_angle: 0.0,
        },
    ));
}

fn update_camera(
    aircraft_query: Query<&Transform, With<Aircraft>>,
    mut camera_query: Query<&mut Transform, (With<MainCamera>, Without<Aircraft>)>,
) {
    if let Ok(aircraft_transform) = aircraft_query.get_single() {
        if let Ok(mut camera_transform) = camera_query.get_single_mut() {
            camera_transform.translation.x = aircraft_transform.translation.x;
            camera_transform.translation.y = aircraft_transform.translation.y;
            // Keep camera's z coordinate unchanged
        }
    }
}

fn aircraft_movement(time: Res<Time>, mut query: Query<(&mut Aircraft, &mut Transform)>) {
    const SPEED: f32 = 300.0;
    const WORLD_BOUNDS: f32 = 2000.0; // Increased to match terrain size

    for (mut aircraft, mut transform) in query.iter_mut() {
        let direction = Vec2::new(
            (aircraft.heading + std::f32::consts::FRAC_PI_2).cos(),
            (aircraft.heading + std::f32::consts::FRAC_PI_2).sin(),
        );
        aircraft.position += direction * SPEED * time.delta_secs();

        // Wrap around world boundaries
        if aircraft.position.x > WORLD_BOUNDS {
            aircraft.position.x = -WORLD_BOUNDS;
        } else if aircraft.position.x < -WORLD_BOUNDS {
            aircraft.position.x = WORLD_BOUNDS;
        }

        if aircraft.position.y > WORLD_BOUNDS {
            aircraft.position.y = -WORLD_BOUNDS;
        } else if aircraft.position.y < -WORLD_BOUNDS {
            aircraft.position.y = WORLD_BOUNDS;
        }

        // Update transform position to match aircraft position
        transform.translation = aircraft.position.extend(10.0); // Keep Z at 10.0
    }
}

fn animate_aircraft(mut query: Query<(&AnimationIndices, &Aircraft, &mut Sprite)>) {
    for (indices, aircraft, mut sprite) in &mut query {
        if let Some(atlas) = &mut sprite.texture_atlas {
            // Change sprite based on bank angle
            atlas.index = if aircraft.bank_angle < -1.0 {
                indices.bank_left
            } else if aircraft.bank_angle > 1.0 {
                indices.bank_right
            } else {
                indices.straight
            };
        }
    }
}

fn input_aircraft(
    time: Res<Time>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut query: Single<(&mut Aircraft, &mut Transform)>,
) {
    if keyboard_input.pressed(KeyCode::ArrowLeft) {
        query.0.bank_angle += 1.0;
    }

    if keyboard_input.pressed(KeyCode::ArrowRight) {
        query.0.bank_angle -= 1.0;
    }

    // Optional: Add bank angle limits
    query.0.bank_angle = query.0.bank_angle.clamp(-45.0, 45.0);

    // Optional: Auto-center when no input
    // if !keyboard_input.pressed(KeyCode::ArrowLeft) && !keyboard_input.pressed(KeyCode::ArrowRight) {
    //     query.0.bank_angle *= 0.95; // Gradually return to level flight
    // }

    // Update heading based on bank angle
    let turn_rate = query.0.bank_angle * 0.1; // Adjust multiplier for desired turn rate
    query.0.heading += turn_rate * time.delta_secs();

    // Keep heading between 0 and 2π
    query.0.heading = query.0.heading % (2.0 * std::f32::consts::PI);

    // Update transform rotation to match heading
    query.1.rotation = Quat::from_rotation_z(query.0.heading);
}
