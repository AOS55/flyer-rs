use bevy::prelude::*;

#[derive(Event)]
pub struct StepCommand {
    pub steps: u32,
}

#[derive(Resource, Debug)]
pub struct UpdateControl {
    pub remaining_steps: u32,
    pub mode: UpdateMode,
}

#[derive(Debug, Clone, Copy)]
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
    pub fn new(mode: UpdateMode) -> Self {
        Self {
            remaining_steps: 0,
            mode,
        }
    }

    pub fn should_update(&self) -> bool {
        match self.mode {
            UpdateMode::SITL => true,
            UpdateMode::Gym => self.remaining_steps > 0,
        }
    }

    pub fn consume_step(&mut self) -> bool {
        if self.remaining_steps > 0 {
            self.remaining_steps -= 1;
            true
        } else {
            false
        }
    }

    pub fn set_steps(&mut self, steps: u32) {
        self.remaining_steps = steps;
    }
}

fn handle_step_commands(
    mut update_control: ResMut<UpdateControl>,
    mut step_commands: EventReader<StepCommand>,
) {
    for step_command in step_commands.read() {
        update_control.set_steps(step_command.steps);
    }
}

pub fn step_condition(step_control: Res<UpdateControl>) -> bool {
    step_control.should_update()
}

fn consume_step(mut update_control: ResMut<UpdateControl>) {
    let before = update_control.remaining_steps;
    let consumed = update_control.consume_step();
}

pub struct UpdateControlPlugin;

impl Plugin for UpdateControlPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<UpdateControl>()
            .add_event::<StepCommand>()
            .add_systems(Update, (handle_step_commands, consume_step));
    }
}
