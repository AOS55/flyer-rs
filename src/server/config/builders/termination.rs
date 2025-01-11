use serde::{Deserialize, Serialize};

use crate::{components::TerminalConditions, server::config::errors::ConfigError};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerminalConditionsBuilder;

impl Default for TerminalConditionsBuilder {
    fn default() -> Self {
        TerminalConditionsBuilder
    }
}

impl TerminalConditionsBuilder {
    // pub fn new() -> Self {
    //     Self::default()
    // }

    pub fn build(self) -> Result<TerminalConditions, ConfigError> {
        let conditions = TerminalConditions::default();

        Ok(conditions)
    }
}
