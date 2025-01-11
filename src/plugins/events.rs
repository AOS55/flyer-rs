use bevy::prelude::*;
use std::collections::HashMap;

#[derive(Event)]
pub struct StepRequestEvent {
    pub actions: HashMap<String, HashMap<String, f64>>,
}

#[derive(Event)]
pub struct StepCompleteEvent {
    pub observations: HashMap<String, HashMap<String, f64>>,
}

#[derive(Event)]
pub struct ResetRequestEvent {
    pub seed: Option<u64>,
}

#[derive(Event)]
pub struct ResetCompleteEvent;
