use crate::{
    plugins::{
        add_aircraft_plugin, AgentPlugin, CameraPlugin, EnvironmentPlugin, HeadlessPlugin,
        PhysicsPlugin, StartupSequencePlugin, TerrainPlugin, TransformationPlugin,
    },
    resources::{RenderMode, UpdateControlPlugin},
    server::EnvConfig,
};
use bevy::prelude::*;

pub fn setup_app(mut app: App, config: EnvConfig, asset_path: String) -> App {
    app.add_plugins(StartupSequencePlugin);
    app.add_plugins((
        TransformationPlugin::new(1.0),
        AgentPlugin::new(config.agent_config),
        UpdateControlPlugin,
        EnvironmentPlugin::with_config(config.environment_config),
        PhysicsPlugin::with_config(config.physics_config),
    ));

    for aircraft_config in config.aircraft_configs.iter() {
        add_aircraft_plugin(&mut app, aircraft_config.1.clone());
    }

    println!("mode: {:?}", config.agent_config.mode);
    // TODO: sort out camera and render setup
    match config.agent_config.mode {
        RenderMode::Human => {
            println!("Running Human Mode");
            app.add_plugins(
                DefaultPlugins
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
                        file_path: asset_path,
                        ..default()
                    }),
            );
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
