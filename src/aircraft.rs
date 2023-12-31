#![warn(clippy::all)]

use aerso::density_models::ConstantDensity;
use aerso::wind_models::*;
use aerso::*;
use aerso::types::*;
use std::{fs::File, io::Read, f64::consts::PI, collections::HashMap};
use serde::{Deserialize, Serialize};

/// The aerodynamics of the aircraft
pub struct Aerodynamics {
    /// Aircraft name
    pub name: String,
    /// Aircraft mass [Kg] 
    pub mass: f64,
    /// Aircraft inertia array [Kg.m^2]
    pub inertia: Matrix3,
    /// Aircraft wing area [m^2]
    pub wing_area: f64,
    /// Aircraft wing span [m]
    pub wing_span: f64,
    /// Aircraft mean aerodynamic chord [m]
    pub mac: f64,
    /// Aircraft DragData
    pub drag_data: DragData,
    /// Aircraft SideForceData
    pub side_force_data: SideForceData,
    /// Aircraft LiftData
    pub lift_data: LiftData,
    /// Aircraft RollData
    pub roll_data: RollData,
    /// Aircraft PitchData
    pub pitch_data: PitchData,
    /// Aircraft YawData
    pub yaw_data: YawData
}

/// Aircraft Inertia data
#[derive(Debug, Deserialize, Serialize)]
pub struct Inertia {
    /// xx-axis inertia [Kg.m^2]
    ixx: f64,
    /// yy-axis inertia [Kg.m^2]
    iyy: f64,
    /// zz-axis inertia [Kg.m^2]
    izz: f64,
    /// xz-axis inertia [Kg.m^2]
    ixz: f64
}

impl Inertia {

    /// Creates a 3x3 inertia matrix based upon the inertia structure
    pub const fn create_inertia(&self) -> Matrix3 {
        Matrix3::new(
            self.ixx, 0.0, self.ixz,
            0.0, self.iyy, 0.0,
            self.ixz, 0.0, self.izz,
        )
    }
}

/// Aerodynamic drag (D) parameters
#[derive(Debug, Deserialize, Serialize)]
#[allow(non_snake_case)]
pub struct DragData {
    /// 0 alpha drag
    pub c_D_0: f64,
    /// drag due to alpha
    pub c_D_alpha: f64,
    /// drag due to alpha.q
    pub c_D_alpha_q: f64,
    /// drag due to alpha.delta_elevator
    pub c_D_alpha_deltae: f64,
    /// drag due to alpha^2
    pub c_D_alpha2: f64,
    /// drag due to alpha^2.q
    pub c_D_alpha2_q: f64,
    /// drag due to alpha^2.delta_elevator
    pub c_D_alpha2_deltae: f64,
    /// drag due to alpha^3
    pub c_D_alpha3: f64,
    /// drag due to alpha^3.q
    pub c_D_alpha3_q: f64,
    /// drag due to alpha^4
    pub c_D_alpha4: f64,
}

/// Aerodynamic sideforce (Y) parameters
#[derive(Debug, Deserialize, Serialize)]
#[allow(non_snake_case)]
pub struct SideForceData {
    /// sideforce due to beta
    pub c_Y_beta: f64,
    /// sideforce due to p
    pub c_Y_p: f64,
    /// sideforce due to r
    pub c_Y_r: f64,
    /// sideforce due to delta_aileron
    pub c_Y_deltaa: f64,
    /// sideforce due to delta_rudder
    pub c_Y_deltar: f64,
}

/// Aerodynamic lift (L) parameters
#[derive(Debug, Deserialize, Serialize)]
#[allow(non_snake_case)]
pub struct LiftData {
    /// 0 alpha lift
    pub c_L_0: f64,
    /// lift due to angle of attack
    pub c_L_alpha: f64,
    /// lift due to pitch rate
    pub c_L_q: f64,
    /// lift due to delta_e
    pub c_L_deltae: f64,
    /// lift due to alpha.q
    pub c_L_alpha_q: f64,
    /// lift due to alpha^2
    pub c_L_alpha2: f64,
    /// lift due to alpha^3
    pub c_L_alpha3: f64,
    /// lift due to alpha^4
    pub c_L_alpha4: f64,
}

/// Aerodynaic roll moment (l)
#[derive(Debug, Deserialize, Serialize)]
pub struct RollData {
    /// roll moment due to beta
    pub c_l_beta: f64,
    /// roll moment due to p
    pub c_l_p: f64,
    /// roll moment due to r
    pub c_l_r: f64,
    /// roll moment due to delta_aileron
    pub c_l_deltaa: f64,
    /// roll moment due to delta_rudder
    pub c_l_deltar: f64,
}

/// Aerodynamic pitch moment (m)
#[derive(Debug, Deserialize, Serialize)]
pub struct PitchData {
    /// 0 alpha pitch moment
    pub c_m_0: f64,
    /// pitch moment due to alpha
    pub c_m_alpha: f64,
    /// pitch moment due to q
    pub c_m_q: f64,
    /// pitch moment due to delta_e
    pub c_m_deltae: f64,
    /// pitch moment due to alpha.q
    pub c_m_alpha_q: f64,
    /// pitch moment due to alpha^2.q
    pub c_m_alpha2_q: f64,
    /// pitch moment due to alpha^2.delta_e
    pub c_m_alpha2_deltae: f64,
    /// pitch moment due to alpha^3.q
    pub c_m_alpha3_q: f64,
    /// pitch moment due to alpha^3.delta_e
    pub c_m_alpha3_deltae: f64,
    /// pitch moment due to alpha^4
    pub c_m_alpha4: f64,
}

/// Aerodynamic yaw moment
#[derive(Debug, Deserialize, Serialize)]
pub struct YawData {
    /// yaw moment due to beta
    pub c_n_beta: f64,
    /// yaw moment due to p
    pub c_n_p: f64,
    /// yaw moment due to r
    pub c_n_r: f64,
    /// yaw moment due to delta_a
    pub c_n_deltaa: f64,
    /// yaw moment due to delta_r
    pub c_n_deltar: f64,
    /// yaw moment due to beta^2
    pub c_n_beta2: f64,
    /// yaw moment due to beta^3
    pub c_n_beta3: f64,
}

impl Aerodynamics {

    /// Create an aerodynamic class from an aircraft json file
    /// # Arguments
    /// * `aircraft_name` - name of the aircraft used to load the .json file, should be in the form <aircraft_name>.json
    /// * `data_path` - path to directory containing aircraft data, if None defaults to './data/'
    #[inline]
    pub fn from_json(aircraft_name: &str, data_path: Option<&str>) -> Self {
        
        let file_name = match data_path {
                Some(data_path) => [data_path, aircraft_name, ".yaml"].concat(),
                None => ["data/", aircraft_name, ".yaml"].concat()
            };

        let mut file: File = match File::open(file_name.clone()) {
            Ok(file) => {file},
            Err(_e) => {
                eprintln!("No file Found! File is: {}", file_name);
                std::process::exit(1);
            }  
        };
        

        let mut yaml_data = String::new();
        file.read_to_string(&mut yaml_data)
            .expect("Failed to read file");
        
        let inertia_result: Result<Inertia, serde_yaml::Error> = serde_yaml::from_str(&yaml_data);
        let drag_result: Result<DragData, serde_yaml::Error> = serde_yaml::from_str(&yaml_data);
        let side_force_result: Result<SideForceData, serde_yaml::Error> = serde_yaml::from_str(&yaml_data);
        let lift_result: Result<LiftData, serde_yaml::Error> = serde_yaml::from_str(&yaml_data);
        let roll_result: Result<RollData, serde_yaml::Error> = serde_yaml::from_str(&yaml_data);
        let pitch_result: Result<PitchData, serde_yaml::Error> = serde_yaml::from_str(&yaml_data);
        let yaw_result: Result<YawData, serde_yaml::Error> = serde_yaml::from_str(&yaml_data);
        
        let inertia = inertia_result.unwrap();
        let inertia_array = inertia.create_inertia();

        Self {
            name: "TO".to_string(),
            mass: 4874.8,
            inertia: inertia_array,
            wing_area: 39.0,
            wing_span: 19.8,
            mac: 1.98,
            drag_data: drag_result.unwrap(),
            side_force_data: side_force_result.unwrap(),
            lift_data: lift_result.unwrap(),
            roll_data: roll_result.unwrap(),
            pitch_data: pitch_result.unwrap(),
            yaw_data: yaw_result.unwrap()
        }
    }   
}


/// Create the [AeroEffect] for the [Aerodynamics] to generate relevant aero forces and torques
#[allow(non_snake_case)]
impl AeroEffect for Aerodynamics {
    
    fn get_effect(&self, airstate: AirState, _rates: Vector3, input: &Vec<f64>) -> (Force, Torque) {

        let alpha = airstate.alpha.clamp(-4.0 * (PI / 180.0), 30.0 * (PI / 180.0));
        let beta = airstate.beta.clamp(-20.0 * (PI / 180.0), 20.0 * (PI / 180.0));
        let p = _rates[0].clamp(-100.0 * (PI / 180.0), 100.0 * (PI / 180.0));
        let q = _rates[1].clamp(-50.0 * (PI / 180.0), 50.0 * (PI / 180.0));
        let r = _rates[2].clamp(-50.0 * (PI / 180.0), 50.0 * (PI / 180.0));

        let tilde_p = (self.wing_span * p) / (2.0 * airstate.airspeed);
        let tilde_q = (self.wing_area * q) / (2.0 * airstate.airspeed);
        let tilde_r = (self.wing_span * r) / (2.0 * airstate.airspeed);
        
        let c_D = 
            self.drag_data.c_D_0 +
            (self.drag_data.c_D_alpha * alpha) +
            (self.drag_data.c_D_alpha_q * alpha * tilde_q) +
            (self.drag_data.c_D_alpha_deltae * alpha * input[1]) +
            (self.drag_data.c_D_alpha2 * alpha.powf(2.0)) +
            (self.drag_data.c_D_alpha2_q * tilde_q * alpha.powf(2.0)) +
            (self.drag_data.c_D_alpha2_deltae * input[1] * alpha.powf(2.0)) +
            (self.drag_data.c_D_alpha3 * alpha.powf(3.0)) +
            (self.drag_data.c_D_alpha3_q * tilde_q * alpha.powf(3.0)) +
            (self.drag_data.c_D_alpha4 * alpha.powf(4.0));

        let c_Y = 
            (self.side_force_data.c_Y_beta * beta) +
            (self.side_force_data.c_Y_p * tilde_p) +
            (self.side_force_data.c_Y_r * tilde_r) +
            (self.side_force_data.c_Y_deltaa * input[0]) +
            (self.side_force_data.c_Y_deltar * input[3]);

        let c_L = 
            self.lift_data.c_L_0 +
            (self.lift_data.c_L_alpha * alpha) +
            (self.lift_data.c_L_q * tilde_q) +
            (self.lift_data.c_L_deltae * input[1]) +
            (self.lift_data.c_L_alpha_q * alpha * tilde_q) +
            (self.lift_data.c_L_alpha2 * alpha.powf(2.0)) +
            (self.lift_data.c_L_alpha3 * alpha.powf(3.0)) +
            (self.lift_data.c_L_alpha4 * alpha.powf(4.0));

        let c_l = 
            (self.roll_data.c_l_beta * beta) +
            (self.roll_data.c_l_p * tilde_p) +
            (self.roll_data.c_l_r * tilde_r) +
            (self.roll_data.c_l_deltaa * input[0]) +
            (self.roll_data.c_l_deltar * input[3]);

        let c_m = 
             self.pitch_data.c_m_0 +
            (self.pitch_data.c_m_alpha * alpha) +
            (self.pitch_data.c_m_q * tilde_q) +
            (self.pitch_data.c_m_deltae * input[1]) +
            (self.pitch_data.c_m_alpha_q * alpha * tilde_q) +
            (self.pitch_data.c_m_alpha2_q * tilde_q * alpha.powf(2.0)) +
            (self.pitch_data.c_m_alpha2_deltae * input[1] * alpha.powf(2.0)) +
            (self.pitch_data.c_m_alpha3_q * tilde_q * alpha.powf(3.0)) +
            (self.pitch_data.c_m_alpha3_deltae * input[1] * alpha.powf(3.0)) +
            (self.pitch_data.c_m_alpha4 * alpha.powf(4.0));

        let c_n = 
            (self.yaw_data.c_n_beta * beta) +
            (self.yaw_data.c_n_p * tilde_p) +
            (self.yaw_data.c_n_r * tilde_r) +
            (self.yaw_data.c_n_deltaa * input[0]) +
            (self.yaw_data.c_n_deltar * input[3]) +
            (self.yaw_data.c_n_beta2 * beta.powf(2.0)) +
            (self.yaw_data.c_n_beta3 * beta.powf(3.0));

        let drag = airstate.q * self.wing_area * c_D;
        let side_force = airstate.q * self.wing_area * c_Y;
        let lift = airstate.q * self.wing_area * c_L;
        let rolling_moment = airstate.q * self.wing_span * self.wing_area * c_l;
        let pitching_moment = airstate.q * self.mac * self.wing_area * c_m;
        let yawing_moment = airstate.q * self.wing_span * self.wing_area * c_n;

        (
            Force::body(-drag, side_force, -lift),
            Torque::body(rolling_moment, pitching_moment, yawing_moment)
        )

    }
}

/// A turboprop representation of the aircraft's power-plant
#[allow(dead_code)]
struct PowerPlant {
    /// Name of the power-plant/engine
    name: String,
    /// Maximum shaft-power [W]
    shaft_power: f64,
    /// Maximum velocity [m/s]
    v_max: f64,
    /// Maximum efficiency
    efficiency: f64
}

impl PowerPlant {

    /// Create a PT6 powerplant
    fn pt6() -> Self {
        Self {
            name: "PT6".to_string(),
            shaft_power: 2.0 * 1.12e6,
            v_max: 40.0,
            efficiency: 0.6
        }
    }
}

/// Create the AeroEffect for the [PowerPlant] data-class to generate relevant aero forces and torques
impl AeroEffect for PowerPlant {

    fn get_effect(&self, _airstate: AirState, _rates: Vector3, input: &Vec<f64>) -> (Force, Torque) {
        let thrust = ((self.shaft_power * self.efficiency) / self.v_max) * input[2];
        (
            Force::body(thrust, 0.0, 0.0),
            Torque::body(0.0, 0.0, 0.0)
        )
    }
}

/// Represent a fixed-wing aircraft
pub struct Aircraft {
    // Name of the aircraft
    pub name: String,
    // Effected aircraft body
    pub aff_body: AffectedBody<Vec<f64>, f64, ConstantWind<f64>, ConstantDensity>,
    // Aircraft controls
    pub controls: HashMap<String, f64>,
    // Path to the aircraft json directory
    pub data_path: Option<String>
}

impl Aircraft {

    #[inline]
    pub fn new(aircraft_name: &str,
               initial_position: Vector3<f64>,
               initial_velocity: Vector3<f64>,
               initial_attitude: UnitQuaternion<f64>,
               initial_rates: Vector3<f64>,
               controls: Option<HashMap<String, f64>>,
               data_path: Option<String>) -> Self {

        let path = data_path.as_deref();

        let aero = Aerodynamics::from_json(aircraft_name, path);
        let power = PowerPlant::pt6();

        let k_body = Body::new(
            aero.mass,
            aero.inertia,
            initial_position,
            initial_velocity,
            initial_attitude,
            initial_rates
        );

        let a_body = AeroBody::new(k_body);

        let aff_body = AffectedBody {
            body: a_body,
            effectors: vec![Box::new(aero), Box::new(power)],
        };

        let controls = match controls {
            Some(controls) => controls,
            None =>  HashMap::from([
                        ("aileron".to_string(), 0.0),
                        ("elevator".to_string(), 0.0),
                        ("tla".to_string(), 0.0),
                        ("rudder".to_string(), 0.0)
                    ])
        };

        Self {name: aircraft_name.to_string(), aff_body, controls, data_path}
    }

    /// Set the controls
    /// # Arguments
    /// * `controls` - HashMap usually containing ["aileron", "elevator", "tla", "rudder"] 
    #[allow(dead_code)]
    pub fn act(&mut self, controls: HashMap<String, f64>) {
        self.controls = controls;
    }

    /// Step the simulation
    #[allow(dead_code)]
    pub fn step(&mut self, dt: f64) {
        

        // HashMaps aren't ordered so we need to make sure everything comes out in the correct sequence
        let control_keys = vec!["aileron", "elevator", "tla", "rudder"];
        let mut control_in: Vec<f64> = Vec::new();

        for key in control_keys {
            control_in.push(self.controls[key]);
        }

        // let controls: Vec<_> = self.controls.values().cloned().collect();
        self.aff_body.step(dt, &control_in); 
    }

}

impl Clone for Aircraft {
    fn clone(&self) -> Self {

        let name: String = self.name.clone();
        let pos = self.position();
        let vel = self.velocity();
        let att = self.attitude();
        let rates = self.rates();
        let controls = self.controls.clone();
        let data_path = self.data_path.clone();
        let ac = Aircraft::new(&name, pos, vel, att, rates, Some(controls), data_path);

        Self {
            name: ac.name,
            aff_body: ac.aff_body,
            controls: ac.controls,
            data_path: ac.data_path
        }       
    }
}

impl StateView for Aircraft {
    fn position(&self) -> Vector3<f64> {
        self.aff_body.position()
    }

    fn velocity_in_frame(&self, frame: Frame) -> Vector3<f64> {
        self.aff_body.velocity_in_frame(frame)
    }

    fn attitude(&self) -> UnitQuaternion<f64> {
        self.aff_body.attitude()
    }

    fn rates_in_frame(&self, frame: Frame) -> Vector3<f64> {
        self.aff_body.rates_in_frame(frame)
    }

    fn statevector(&self) -> StateVector<f64> {
        self.aff_body.statevector()
    }
}
