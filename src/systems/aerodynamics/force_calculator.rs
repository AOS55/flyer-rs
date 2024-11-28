use super::aerso_adapter::AersoAdapter;
use crate::components::{
    AerodynamicsComponent, Force, ForceCategory, Moment, PhysicsComponent, ReferenceFrame,
    SpatialComponent,
};
use crate::ecs::error::Result;
use crate::ecs::{System, World};
use aerso::{AeroEffect, AirState};

const MIN_AIRSPEED_THRESHOLD: f64 = 1e-6;

pub struct AeroForceSystem {
    adapter: AersoAdapter,
}

impl AeroForceSystem {
    pub fn new(aero: AerodynamicsComponent) -> Self {
        Self {
            adapter: AersoAdapter::new(aero.geometry, aero.coefficients),
        }
    }

    fn calculate_aero_forces(
        &self,
        spatial: &SpatialComponent,
        aero: &AerodynamicsComponent,
        physics: &mut PhysicsComponent,
    ) {
        if aero.air_data.true_airspeed < MIN_AIRSPEED_THRESHOLD {
            return;
        }

        let air_state = AirState {
            alpha: aero.air_data.alpha,
            beta: aero.air_data.beta,
            airspeed: aero.air_data.true_airspeed,
            q: aero.air_data.dynamic_pressure,
        };

        let input = vec![
            aero.control_surfaces.aileron,
            aero.control_surfaces.elevator,
            aero.control_surfaces.rudder,
            aero.control_surfaces.flaps,
        ];

        let (aero_force, aero_torque) =
            self.adapter
                .get_effect(air_state, spatial.angular_velocity, &input);

        let force_vector = match aero_force.frame {
            aerso::types::Frame::Body => aero_force.force,
            aerso::types::Frame::World => spatial.attitude.inverse() * aero_force.force,
        };

        physics.add_force(Force {
            vector: force_vector,
            point: None,
            frame: ReferenceFrame::Body,
            category: ForceCategory::Aerodynamic,
        });

        physics.add_moment(Moment {
            vector: aero_torque.torque,
            frame: ReferenceFrame::Body,
            category: ForceCategory::Aerodynamic,
        });

        // Calculate net forces and moments in body frame
        physics.net_force = force_vector;
        physics.net_moment = aero_torque.torque;
    }
}

impl System for AeroForceSystem {
    fn name(&self) -> &str {
        "Aerodynamic Force Calculator"
    }

    fn run(&self, world: &mut World) -> Result<()> {
        let mut entity_data = Vec::new();

        // First collect all required components
        for (entity, spatial) in world.query::<SpatialComponent>() {
            if let (Ok(aero), Ok(physics)) = (
                world.get_component::<AerodynamicsComponent>(entity),
                world.get_component::<PhysicsComponent>(entity),
            ) {
                entity_data.push((entity, spatial.clone(), aero.clone(), physics.clone()));
            }
        }

        // Then update physics components
        for (entity, spatial, aero, _) in entity_data {
            if let Ok(physics) = world.get_component_mut::<PhysicsComponent>(entity) {
                self.calculate_aero_forces(&spatial, &aero, physics);
            }
        }

        Ok(())
    }

    fn dependencies(&self) -> Vec<&str> {
        vec!["Air Data System"]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::{AircraftGeometry, ControlSurfaces};
    use crate::resources::{
        AtmosphereConfig, AtmosphereType, EnvironmentConfig, EnvironmentResource, WindModelConfig,
    };
    use approx::assert_relative_eq;
    use nalgebra::{Matrix3, UnitQuaternion, Vector3};
    use std::f64::consts::PI;

    fn setup_test_world() -> World {
        let mut world = World::new();

        // Create environment config with zero wind for simpler testing
        let env_config = EnvironmentConfig {
            wind_model_config: WindModelConfig::Constant {
                velocity: Vector3::zeros(),
            },
            atmosphere_config: AtmosphereConfig {
                model_type: AtmosphereType::Standard,
                sea_level_density: 1.225,
                sea_level_temperature: 288.15,
            },
        };

        // Add environment resource to world
        let env = EnvironmentResource::new(&env_config);
        world.add_resource(env).unwrap();

        world
    }

    fn create_test_components() -> (SpatialComponent, AerodynamicsComponent, PhysicsComponent) {
        let spatial = SpatialComponent {
            position: Vector3::new(0.0, 0.0, 1000.0),
            velocity: Vector3::new(50.0, 0.0, 0.0), // 50 m/s forward flight
            attitude: UnitQuaternion::identity(),
            angular_velocity: Vector3::zeros(),
        };

        let mut aero = AerodynamicsComponent::default();
        aero.geometry = AircraftGeometry {
            wing_area: 16.0,
            wing_span: 10.0,
            mean_aerodynamic_chord: 1.6,
        };

        // Set some realistic aerodynamic coefficients
        aero.coefficients.lift.c_l_alpha = 5.0;
        aero.coefficients.drag.c_d_0 = 0.025;
        aero.coefficients.pitch.c_m_deltae = -1.5;

        println!("Test components:");
        println!("Aero coefficients: {:?}", aero.coefficients);
        println!("Geometry: {:?}", aero.geometry);

        // Initialize air data
        aero.air_data.true_airspeed = 50.0;
        aero.air_data.density = 1.225;
        aero.air_data.dynamic_pressure = 0.5 * 1.225 * 50.0 * 50.0;

        let physics = PhysicsComponent::new(
            1000.0,                       // 1000 kg mass
            Matrix3::identity() * 1000.0, // Simple inertia tensor
        );

        (spatial, aero, physics)
    }

    #[test]
    fn test_basic_force_calculation() {
        let mut world = setup_test_world();
        let (spatial, mut aero, physics) = create_test_components();

        aero.air_data.true_airspeed = 50.0;
        aero.air_data.alpha = 0.05; // Small positive angle of attack
        aero.air_data.beta = 0.0; // No sideslip
        aero.air_data.dynamic_pressure = 0.5 * 1.225 * 50.0 * 50.0;
        aero.air_data.density = 1.225;

        let entity = world.spawn();
        world.add_component(entity, spatial.clone()).unwrap();
        world.add_component(entity, aero.clone()).unwrap();
        world.add_component(entity, physics).unwrap();

        let system = AeroForceSystem::new(aero.clone());
        system.run(&mut world).unwrap();

        let physics = world.get_component::<PhysicsComponent>(entity).unwrap();

        println!("Forces in physics component:");
        for force in &physics.forces {
            println!("  Force: {:?}", force);
        }
        println!("Net force: {:?}", physics.net_force);
        println!("Net moment: {:?}", physics.net_moment);

        assert!(
            physics.net_force.norm() > 0.0,
            "Expected non-zero net force"
        );
    }

    #[test]
    fn test_zero_airspeed_condition() {
        let mut world = setup_test_world();
        let (mut spatial, mut aero, physics) = create_test_components();

        // Set zero velocity
        spatial.velocity = Vector3::zeros();

        // Update air data to match zero velocity condition
        aero.air_data.true_airspeed = 0.0;
        aero.air_data.dynamic_pressure = 0.0; // q = 0.5 * rho * V^2 = 0 when V = 0
        aero.air_data.alpha = 0.0;
        aero.air_data.beta = 0.0;

        let entity = world.spawn();
        world.add_component(entity, spatial).unwrap();
        world.add_component(entity, aero.clone()).unwrap();
        world.add_component(entity, physics).unwrap();

        let system = AeroForceSystem::new(aero);
        system.run(&mut world).unwrap();

        let physics = world.get_component::<PhysicsComponent>(entity).unwrap();

        assert_relative_eq!(physics.net_force.norm(), 0.0, epsilon = 1e-6);
    }

    #[test]
    fn test_control_surface_moments() {
        let mut world = setup_test_world();
        let (spatial, mut aero, physics) = create_test_components();

        // Set elevator deflection
        aero.control_surfaces = ControlSurfaces {
            elevator: 0.1, // 10% deflection
            ..Default::default()
        };

        // Initialize air data
        aero.air_data.true_airspeed = 50.0;
        aero.air_data.alpha = 0.0;
        aero.air_data.beta = 0.0;
        aero.air_data.dynamic_pressure = 0.5 * 1.225 * 50.0 * 50.0;
        aero.air_data.density = 1.225;

        let entity = world.spawn();
        world.add_component(entity, spatial).unwrap();
        world.add_component(entity, aero.clone()).unwrap();
        world.add_component(entity, physics).unwrap();

        let system = AeroForceSystem::new(aero);
        system.run(&mut world).unwrap();

        let physics = world.get_component::<PhysicsComponent>(entity).unwrap();
        assert!(
            physics.net_moment.y != 0.0,
            "Expected pitch moment from elevator"
        );
    }

    #[test]
    fn test_force_frame_transformation() {
        let mut world = setup_test_world();
        let (mut spatial, aero, physics) = create_test_components();

        // Set aircraft attitude to 45 degrees pitch up
        spatial.attitude = UnitQuaternion::from_axis_angle(&Vector3::y_axis(), PI / 4.0);

        let entity = world.spawn();
        world.add_component(entity, spatial).unwrap();
        world.add_component(entity, aero.clone()).unwrap();
        world.add_component(entity, physics).unwrap();

        let system = AeroForceSystem::new(aero);
        system.run(&mut world).unwrap();

        let physics = world.get_component::<PhysicsComponent>(entity).unwrap();

        // Forces are in body frame
        assert!(
            physics.net_force.norm() > 1e-6,
            "Expected non-zero net force"
        );

        // Since we're in body frame, with pitch up we expect:
        assert!(
            physics.net_force.x < 0.0,
            "Expected negative x force (drag)"
        );
        assert!(
            physics.net_force.z < 0.0,
            "Expected negative z force (lift)"
        );
    }

    #[test]
    fn test_high_angle_of_attack() {
        let mut world = setup_test_world();
        let (mut spatial, mut aero, physics) = create_test_components();

        // Initialize air data
        aero.air_data.true_airspeed = 50.0;
        aero.air_data.alpha = 15.0 * PI / 180.0;
        aero.air_data.beta = 0.0;
        aero.air_data.dynamic_pressure = 0.5 * 1.225 * 50.0 * 50.0;
        aero.air_data.density = 1.225;

        // Set high angle of attack
        spatial.attitude = UnitQuaternion::from_axis_angle(&Vector3::y_axis(), aero.air_data.alpha);

        let entity = world.spawn();
        world.add_component(entity, spatial).unwrap();
        world.add_component(entity, aero.clone()).unwrap();
        world.add_component(entity, physics).unwrap();

        let system = AeroForceSystem::new(aero);
        system.run(&mut world).unwrap();

        let physics = world.get_component::<PhysicsComponent>(entity).unwrap();
        assert!(
            physics.net_force.norm() > 0.0,
            "Expected increased forces at high AoA"
        );
    }
}
