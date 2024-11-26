pub struct Settings {
    pub simulation_frequency: f64, // frequency of simulation update [Hz]
    pub policy_frequency: f64,     // frequency of policy update
    pub render_frequency: f64,     // frequency of render update
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            simulation_frequency: 120.0,
            policy_frequency: 1.0,
            render_frequency: 0.01,
        }
    }
}

impl Settings {
    pub fn new(
        simulation_frequency: Option<f64>,
        policy_frequency: Option<f64>,
        render_frequency: Option<f64>,
    ) -> Self {
        let simulation_frequency = if let Some(frequency) = simulation_frequency {
            frequency
        } else {
            120.0
        };

        let policy_frequency = if let Some(frequency) = policy_frequency {
            frequency
        } else {
            1.0
        };

        let render_frequency = if let Some(frequency) = render_frequency {
            frequency
        } else {
            0.01
        };

        Self {
            simulation_frequency,
            policy_frequency,
            render_frequency,
        }
    }
}
