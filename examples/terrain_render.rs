use flyer::{
    components::{CameraComponent, TerrainComponent},
    ecs::{System, World},
    resources::{RenderConfig, ResourceSystem},
    systems::{
        render::RenderSystem,
        terrain::{TerrainGeneratorSystem, TerrainManagerSystem},
    },
};
use glam::{UVec2, Vec2};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize world and resources
    let mut world = World::new();

    // Set up required resources
    let mut resource_system = ResourceSystem::builder().with_base_path("assets").build()?;

    // Configure render settings
    let render_config = RenderConfig {
        screen_width: 1920,
        screen_height: 1080,
        vsync: true,
        fov: 60.0,
        draw_distance: 1000.0,
    };

    // Create camera
    let camera = CameraComponent {
        position: Vec2::ZERO,
        viewport: Vec2::new(
            render_config.screen_width as f32,
            render_config.screen_height as f32,
        ),
        zoom: 0.5, // Zoomed out to see more terrain
        ..Default::default()
    };

    // Add resources to world
    world.add_resource(resource_system);
    world.add_resource(render_config.clone());
    world.add_resource(camera.clone()); // Add as resource

    // Add camera as an entity with component
    let camera_entity = world.spawn();
    world.add_component(camera_entity, camera)?;

    // Create terrain component
    let terrain = TerrainComponent::new(
        UVec2::new(1, 1), // world size
        32,               // chunk size
        12345,            // seed
        1.0,              // scale
    );

    // Spawn terrain entity
    let terrain_entity = world.spawn();
    world.add_component(terrain_entity, terrain)?;

    // Initialize systems
    let mut terrain_generator = TerrainGeneratorSystem::new(12345);
    let mut terrain_manager = TerrainManagerSystem::new();
    let mut render_system = RenderSystem::new(&render_config);

    // Run systems once to generate and render terrain
    terrain_manager.run(&mut world)?;
    terrain_generator.run(&mut world)?;
    render_system.run(&mut world)?;

    // Get the rendered frame
    let pixmap = render_system.get_render_buffer();

    // Save the rendered image
    pixmap.save_png("terrain_render.png")?;

    println!("Terrain render saved to terrain_render.png");
    Ok(())
}
