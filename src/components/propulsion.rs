use crate::ecs::component::Component;
use serde::{Deserialize, Serialize};
use std::any::Any;

/// Different types of propulsion systems available
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum PropulsionType {
    Piston,
    TurboProp,
    TurboJet,
    Electric,
}

/// Component containing propulsion system state data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PropulsionComponent {
    /// Type of propulsion system
    pub propulsion_type: PropulsionType,

    /// Maximum available power [W]
    pub max_power: f64,

    /// Current throttle setting [0-1]
    pub throttle: f64,

    /// Overall efficiency [0-1]
    pub efficiency: f64,

    /// Current RPM
    pub rpm: f64,

    /// Maximum RPM
    pub max_rpm: f64,

    /// Engine temperature [K]
    pub temperature: f64,

    /// Fuel flow rate [kg/s]
    pub fuel_flow: f64,
}

impl Default for PropulsionComponent {
    fn default() -> Self {
        Self {
            propulsion_type: PropulsionType::TurboProp,
            max_power: 1000.0 * 1000.0, // 1000 kW
            throttle: 0.0,
            efficiency: 0.8,
            rpm: 0.0,
            max_rpm: 2100.0,
            temperature: 288.15, // 15°C in Kelvin
            fuel_flow: 0.0,
        }
    }
}

impl Component for PropulsionComponent {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_propulsion() {
        let prop = PropulsionComponent::default();
        assert_eq!(prop.throttle, 0.0);
        assert_eq!(prop.efficiency, 0.8);
        assert_eq!(prop.rpm, 0.0);
    }

    #[test]
    fn test_component_casting() {
        let prop = PropulsionComponent::default();
        let any_ref = prop.as_any();
        assert!(any_ref.downcast_ref::<PropulsionComponent>().is_some());
    }
}
