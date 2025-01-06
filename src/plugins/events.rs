use bevy::prelude::*;

#[derive(Event)]
pub struct ResetRequestEvent {
    pub seed: Option<u64>,
}

#[derive(Event)]
pub struct ResetCompleteEvent;
