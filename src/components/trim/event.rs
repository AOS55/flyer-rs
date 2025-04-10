use bevy::prelude::*;

use crate::components::{TrimCondition, TrimSolver};

#[derive(Component)]
pub struct NeedsTrim {
    pub condition: TrimCondition,
    pub stage: TrimStage,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TrimStage {
    Longitudinal,
    Lateral,
    Complete,
}

#[derive(Event)]
pub struct TrimRequest {
    pub entity: Entity,
    pub condition: TrimCondition,
}
