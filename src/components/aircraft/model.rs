#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhysicsModel {
    Simple,
    Full,
}

impl Default for PhysicsModel {
    fn default() -> Self {
        Self::Simple
    }
}
