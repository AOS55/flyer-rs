use flyer::{
    components::{CameraComponent, TerrainComponent, TerrainGenConfig},
    ecs::{System, World},
    resources::TimeManager,
    resources::{RenderConfig, SimulationConfig},
    systems::render::RenderSystem,
    systems::terrain::TerrainGeneratorSystem,
};
use glam::{UVec2, Vec2};
use std::env;
use std::fs::File;
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize World
    let mut world = World::new();

    // Set up camera
    let camera = CameraComponent {
        position: Vec2::new(0.0, 0.0),
        viewport: Vec2::new(800.0, 600.0),
        zoom: 0.5,
        bounds: None,
        target: None,
        interpolation_factor: 0.1,
    };

    let camera_entity = world.spawn();
    world.add_component(camera_entity, camera)?;

    // Add time manager
    let time_manager = flyer::resources::TimeManager::new();
    world.add_resource(time_manager)?;

    // Add render config
    let render_config = RenderConfig {
        screen_width: 800,
        screen_height: 600,
        vsync: false,
        fov: 60.0,
        draw_distance: 1000.0,
    };
    world.add_resource(render_config.clone())?;

    // Create terrain entity
    let terrain_entity = world.spawn();
    let terrain = TerrainComponent::new(
        UVec2::new(10, 10), // Small world size for example
        32,                 // Chunk size
        12345,              // Random seed
        1.0,                // Scale
    );
    world.add_component(terrain_entity, terrain)?;

    // Create systems
    let mut terrain_system = TerrainGeneratorSystem::new(12345);
    let mut render_system = RenderSystem::new(&render_config);

    // Debug: Print all components
    println!("Listing all entities and their components:");
    for entity in world.entities() {
        println!("Entity {:?}", entity);
        if world.has_component::<CameraComponent>(entity) {
            println!("  Has camera component");
        }
    }

    // Generate and render terrain
    terrain_system.run(&mut world)?;
    render_system.run(&mut world)?;

    // Save output
    let buffer = render_system.get_render_buffer();
    let output_path = PathBuf::from(env::args().nth(1).unwrap_or("terrain.png".to_string()));
    buffer.save_png(&output_path)?;

    Ok(())
}
