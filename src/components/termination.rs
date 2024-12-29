use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct TerminalConditions;

impl Default for TerminalConditions {
    fn default() -> Self {
        TerminalConditions
    }
}
