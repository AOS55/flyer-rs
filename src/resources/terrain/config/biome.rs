#[derive(Clone, Debug)]
pub struct BiomeConfig {
    pub thresholds: BiomeThresholds,
}

#[derive(Clone, Debug)]
pub struct BiomeThresholds {
    pub water: f32,
    pub mountain_start: f32,
    pub mountain_width: f32,
    pub beach_width: f32,
    pub forest_moisture: f32,
    pub desert_moisture: f32,
    pub field_sizes: [f32; 4],
}

impl Default for BiomeConfig {
    fn default() -> Self {
        Self {
            thresholds: BiomeThresholds::default(),
        }
    }
}

impl Default for BiomeThresholds {
    fn default() -> Self {
        Self {
            water: 0.48,
            mountain_start: 0.75,
            mountain_width: 0.1,
            beach_width: 0.025,
            forest_moisture: 0.95,
            desert_moisture: 0.2,
            field_sizes: [96.0, 128.0, 256.0, 512.0],
        }
    }
}
