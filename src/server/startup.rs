use crate::{
    plugins::{
        add_aircraft_plugin, AgentPlugin, CameraPlugin, EnvironmentPlugin, HeadlessPlugin,
        PhysicsPlugin, StartupSequencePlugin, TerrainPlugin, TransformationPlugin, TrimPlugin,
    },
    resources::{RenderMode, UpdateControlPlugin},
    server::EnvConfig,
};
use bevy::{log::tracing_subscriber, prelude::*, utils::tracing};
pub fn setup_app(mut app: App, config: EnvConfig, asset_path: String) -> App {
    app.add_plugins(StartupSequencePlugin);
    app.add_plugins((
        TransformationPlugin::new(1.0),
        AgentPlugin::new(config.agent_config),
        UpdateControlPlugin,
        EnvironmentPlugin::with_config(config.environment_config),
        PhysicsPlugin::with_config(config.physics_config),
    ));

    app.add_plugins(TrimPlugin);

    for aircraft_config in config.aircraft_configs.iter() {
        add_aircraft_plugin(&mut app, aircraft_config.1.clone());
    }

    println!("mode: {:?}", config.agent_config.mode);

    // TODO: sort out camera and render setup
    match config.agent_config.mode {
        RenderMode::Human => {
            println!("Running Human Mode");

            // Create a special logger just for human mode
            // This must happen before any other logging setup
            let subscriber = tracing_subscriber::FmtSubscriber::builder()
                .with_max_level(tracing::Level::INFO)
                .with_writer(|| std::io::stderr())
                .finish();

            // Attempt to set the global subscriber
            match tracing::subscriber::set_global_default(subscriber) {
                Ok(_) => println!("Successfully set human mode custom logger"),
                Err(e) => println!("Failed to set custom logger: {}", e),
            }

            // Then add DefaultPlugins without LogPlugin
            let default_plugins = DefaultPlugins
                .build()
                .disable::<bevy::log::LogPlugin>()
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "FlyerEnv".into(),
                        resolution: (
                            config.agent_config.render_width,
                            config.agent_config.render_height,
                        )
                            .into(),
                        ..default()
                    }),
                    ..default()
                })
                .set(AssetPlugin {
                    file_path: asset_path.clone(),
                    ..default()
                });

            app.add_plugins(default_plugins);
            app.add_plugins(CameraPlugin);
            app.insert_resource(Time::<Fixed>::from_seconds(1.0 / 60.0));
        }
        RenderMode::RGBArray => {
            println!("Running RGBArray Mode");
            app.add_plugins(HeadlessPlugin::new(
                config.agent_config.render_width as u32,
                config.agent_config.render_height as u32,
                asset_path,
            ))
            .add_plugins(CameraPlugin);
            app.insert_resource(Time::<Fixed>::from_seconds(1.0 / 10000.0));
            // .add_systems(FixedUpdate, camera_follow_system);
            // .insert_resource(Time::<Fixed>::from_seconds(1.0 / 60.0));
        }
    }

    // app.add_plugins(CameraPlugin);
    app.add_plugins(TerrainPlugin::new());

    app
}
