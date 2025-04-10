use crate::components::{NeedsTrim, TrimCondition, TrimRequest, TrimStage};
use bevy::prelude::*;

pub fn handle_trim_requests(mut commands: Commands, mut trim_requests: EventReader<TrimRequest>) {
    for request in trim_requests.read() {
        // Determine initial trim stage based on condition
        let initial_stage = match request.condition {
            TrimCondition::StraightAndLevel { .. } => TrimStage::Longitudinal,
            TrimCondition::SteadyClimb { .. } => TrimStage::Longitudinal,
            TrimCondition::CoordinatedTurn { .. } => TrimStage::Longitudinal, // Still start with longitudinal
        };

        commands.entity(request.entity).insert(NeedsTrim {
            condition: request.condition,
            stage: initial_stage,
        });
    }
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_trim_request_handling() {
        // Test event handling
        // Verify component addition
        // Check condition passing
    }

    #[test]
    fn test_multiple_requests() {
        // Test handling multiple trim requests
        // Verify queue behavior
        // Check entity updates
    }
}
