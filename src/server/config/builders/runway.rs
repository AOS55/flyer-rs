use nalgebra::Vector3;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{components::RunwayComponent, server::config::errors::ConfigError};

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct RunwayConfigBuilder {
    pub position: Option<Vector3<f64>>,
    pub heading: Option<f64>,
    pub width: Option<f64>,
    pub length: Option<f64>,
}

impl RunwayConfigBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_json(value: &Value) -> Result<Self, ConfigError> {
        let mut builder = Self::new();

        if let Some(pos_val) = value.get("position") {
            let x = pos_val.get("x").and_then(Value::as_f64).unwrap_or(0.0);
            let y = pos_val.get("y").and_then(Value::as_f64).unwrap_or(0.0);
            let z = pos_val.get("z").and_then(Value::as_f64).unwrap_or(0.0); // Usually 0 for runway threshold
            builder.position = Some(Vector3::new(x, y, z));
        }

        builder.heading = value.get("heading").and_then(Value::as_f64);
        builder.width = value.get("width").and_then(Value::as_f64);
        builder.length = value.get("length").and_then(Value::as_f64);

        Ok(builder)
    }

    pub fn build(self) -> Result<RunwayComponent, ConfigError> {
        Ok(RunwayComponent {
            position: self.position.unwrap_or_else(Vector3::zeros),
            heading: self.heading.unwrap_or(0.0), // Default North
            width: self.width.unwrap_or(30.0),
            length: self.length.unwrap_or(1000.0),
        })
    }
}
