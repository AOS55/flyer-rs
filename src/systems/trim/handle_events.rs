use bevy::prelude::*;

use crate::components::{NeedsTrim, TrimRequest};

pub fn handle_trim_requests(mut commands: Commands, mut trim_requests: EventReader<TrimRequest>) {
    for request in trim_requests.read() {
        commands.entity(request.entity).insert(NeedsTrim {
            condition: request.condition,
            solver: None,
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
