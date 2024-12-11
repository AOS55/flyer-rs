use bevy::prelude::*;

#[derive(Component, Debug, Default)]
pub struct PlayerController {
    pub active: bool,
}

impl PlayerController {
    pub fn new() -> Self {
        Self { active: true }
    }

    pub fn disabled() -> Self {
        Self { active: false }
    }

    pub fn enable(&mut self) {
        self.active = true;
    }

    pub fn disable(&mut self) {
        self.active = false;
    }
}
