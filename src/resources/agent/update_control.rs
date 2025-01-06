use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// Event for triggering controlled update steps.
#[derive(Event)]
pub struct StepCommand {
    pub steps: usize,
}

/// Resource for managing update behavior and step control.
#[derive(Resource, Debug)]
pub struct UpdateControl {
    /// Remaining steps to execute in `Gym` mode.
    pub remaining_steps: usize,
    /// Current update mode (e.g., SITL, Gym).
    pub mode: UpdateMode,
}

/// Enum defining different update modes.
#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
pub enum UpdateMode {
    SITL, // Continuous updates
    Gym,  // Controlled by step() calls
}

impl Default for UpdateControl {
    fn default() -> Self {
        Self {
            remaining_steps: 0,
            mode: UpdateMode::SITL,
        }
    }
}

impl UpdateControl {
    /// Creates a new `UpdateControl` with the specified mode.
    pub fn new(mode: UpdateMode) -> Self {
        Self {
            remaining_steps: 0,
            mode,
        }
    }

    /// Determines whether the system should update based on the current mode.
    pub fn should_update(&self) -> bool {
        match self.mode {
            UpdateMode::SITL => true,
            UpdateMode::Gym => self.remaining_steps > 0,
        }
    }

    /// Consumes a single step, decrementing the remaining step count.
    pub fn consume_step(&mut self) -> bool {
        if self.remaining_steps > 0 {
            self.remaining_steps -= 1;
            true
        } else {
            false
        }
    }

    /// Sets the number of steps to execute in `Gym` mode.
    pub fn set_steps(&mut self, steps: usize) {
        self.remaining_steps = steps;
    }
}

/// Handles incoming `StepCommand` events and updates the step count.
fn handle_step_commands(
    mut update_control: ResMut<UpdateControl>,
    mut step_commands: EventReader<StepCommand>,
) {
    for step_command in step_commands.read() {
        update_control.set_steps(step_command.steps);
    }
}

/// Determines whether the system should update based on `UpdateControl`.
pub fn step_condition(step_control: Res<UpdateControl>) -> bool {
    step_control.should_update()
}

/// Consumes a single step after the update stage.
fn consume_step(mut update_control: ResMut<UpdateControl>) {
    update_control.consume_step();
    info!(
        "CONSUME, remaining_steps: {}",
        update_control.remaining_steps
    );
}

/// Plugin for registering the `UpdateControl` systems and resources.
pub struct UpdateControlPlugin;

impl Plugin for UpdateControlPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<UpdateControl>()
            .add_event::<StepCommand>()
            .add_systems(FixedUpdate, (handle_step_commands, consume_step).chain());
    }
}
