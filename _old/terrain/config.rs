#[allow(dead_code)]
pub struct TerrainConfig {
    pub name: String,
    pub field_density: f32,
    pub land_types: Vec<String>,
    pub water_cutoff: f32,
    pub beach_thickness: f32,
    pub forest_tree_density: f32,
    pub orchard_tree_density: f32,
    pub orchard_flower_density: f32,
}

impl TerrainConfig {
    #[allow(dead_code)]
    pub fn update_name(&mut self) {
        // Update the name of the TerrainConfig to be a unique string identifier

        // Create a string made up of the first letters of each string
        let land_letters: String = self
            .land_types
            .iter()
            .map(|s| s.chars().next().unwrap_or_default())
            .collect();

        self.name = format!(
            "fd{}lt{}wc{}bt{}ftd{}otd{}ofd{}",
            self.field_density,
            land_letters,
            self.water_cutoff,
            self.beach_thickness,
            self.forest_tree_density,
            self.orchard_tree_density,
            self.orchard_flower_density
        );
    }
}

impl Default for TerrainConfig {
    fn default() -> Self {
        Self {
            name: "default".to_string(),
            field_density: 0.001,
            land_types: ["grass", "forest", "crops", "orchard"]
                .iter()
                .map(|x| x.to_string())
                .collect::<Vec<String>>(),
            water_cutoff: -0.1,
            beach_thickness: 0.04,
            forest_tree_density: 0.6,
            orchard_tree_density: 0.1,
            orchard_flower_density: 0.1,
        }
    }
}
