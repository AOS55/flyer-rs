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
