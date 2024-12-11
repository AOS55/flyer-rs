use bevy::prelude::*;
use std::collections::HashMap;

use crate::components::aircraft::{AircraftType, Attitude};

#[derive(Resource, Clone)]
pub struct AircraftAssets {
    pub aircraft_textures: HashMap<AircraftType, Handle<Image>>,
    pub aircraft_layouts: HashMap<AircraftType, Handle<TextureAtlasLayout>>,
    pub aircraft_mappings: HashMap<Attitude, usize>,
}

impl AircraftAssets {
    pub fn new() -> Self {
        Self {
            aircraft_textures: HashMap::new(),
            aircraft_layouts: HashMap::new(),
            aircraft_mappings: HashMap::new(),
        }
    }

    pub fn get_aircraft_texture(&self, ac_type: AircraftType) -> &Handle<Image> {
        self.aircraft_textures
            .get(&ac_type)
            .expect("Aircraft texture not found")
    }

    pub fn get_aircraft_layout(&self, ac_type: AircraftType) -> &Handle<TextureAtlasLayout> {
        self.aircraft_layouts
            .get(&ac_type)
            .expect("Aircraft layout not found")
    }
}
