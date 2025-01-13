use bevy::prelude::*;

use crate::resources::{EnvironmentConfig, EnvironmentModel};

pub struct EnvironmentPlugin {
    pub config: Option<EnvironmentConfig>,
}

impl EnvironmentPlugin {
    pub fn new() -> Self {
        Self { config: None }
    }

    pub fn with_config(config: EnvironmentConfig) -> Self {
        Self {
            config: Some(config),
        }
    }

    fn setup_config(mut commands: Commands, config: Option<EnvironmentConfig>) {
        commands.insert_resource(config.unwrap_or_default());
    }

    fn setup_model(mut commands: Commands, config: Option<Res<EnvironmentConfig>>) {
        // If no config is present, use default
        let config = if let Some(cfg) = config {
            cfg.clone()
        } else {
            EnvironmentConfig::default()
        };

        let environment = EnvironmentModel::new(&config);
        commands.insert_resource(environment);
    }

    fn setup_config_with_initial(
        config: Option<EnvironmentConfig>,
    ) -> impl FnMut(Commands) + Send + Sync + 'static {
        move |commands: Commands| {
            Self::setup_config(commands, config.clone());
        }
    }
}

impl Plugin for EnvironmentPlugin {
    fn build(&self, app: &mut App) {
        let config = self.config.clone();
        // Setup configuration
        app.add_systems(
            Startup,
            (Self::setup_config_with_initial(config), Self::setup_model).chain(),
        );
    }
}
