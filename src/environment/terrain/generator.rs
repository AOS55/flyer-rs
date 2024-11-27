use super::*;
use kiddo::{distance::squared_euclidean, KdTree};
use nalgebra::DMatrix;
use noise::{NoiseFn, OpenSimplex};
use rand::distributions::Distribution;
use rand::SeedableRng;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{Read, Write};
use std::path::PathBuf;
use tiny_skia::*;

#[derive(Serialize, Deserialize)]
struct TerrainData {
    tiles: Vec<Tile>,
    objects: Vec<StaticObject>,
}

pub struct TerrainGenerator {
    seed: u64,
    area: Vec<usize>,
    scaling: f32,
    config: TerrainConfig,
    water_present: bool,
    random_funcs: RandomFuncs,
}

impl TerrainGenerator {
    pub fn new(
        seed: u64,
        area: Vec<usize>,
        scaling: f32,
        config: TerrainConfig,
        water_present: bool,
    ) -> Self {
        Self {
            seed,
            area,
            scaling,
            config,
            water_present,
            random_funcs: RandomFuncs::new(seed as u32),
        }
    }

    pub fn get_name(&mut self) -> String {
        self.config.update_name();
        let config_name = &self.config.name;

        format!(
            "seed{}_area0{}1{}_scaling{}_tconfig{}_wp{}",
            self.seed, self.area[0], self.area[1], self.scaling, config_name, self.water_present
        )
    }

    pub fn generate_or_load_map(
        &self,
        terrain_data_dir: &PathBuf,
    ) -> (Vec<Tile>, Vec<StaticObject>) {
        let name = format!("{}.json", self.get_name());
        let mut config_path = terrain_data_dir.clone();
        config_path.push(name);

        match File::open(&config_path) {
            Ok(mut file) => {
                let mut json_data = String::new();
                file.read_to_string(&mut json_data)
                    .expect("Failed to read terrain data file");
                let t_data: TerrainData =
                    serde_json::from_str(&json_data).expect("Failed to deserialize terrain data");
                (t_data.tiles, t_data.objects)
            }
            Err(_) => {
                let (tiles, objects) = self.generate_map();
                let terrain_data = TerrainData { tiles, objects };

                // Create directory if it doesn't exist
                if !terrain_data_dir.exists() {
                    std::fs::create_dir_all(terrain_data_dir)
                        .expect("Failed to create terrain data directory");
                }

                let serialized =
                    serde_json::to_string(&terrain_data).expect("Failed to serialize terrain data");
                let mut file =
                    File::create(&config_path).expect("Failed to create terrain data file");
                file.write_all(serialized.as_bytes())
                    .expect("Failed to write terrain data");

                (terrain_data.tiles, terrain_data.objects)
            }
        }
    }

    pub fn generate_map(&self) -> (Vec<Tile>, Vec<StaticObject>) {
        let biome_map = self.generate_biome_map();
        let land_map = self.generate_land_map();

        let mut tiles: Vec<Tile> = Vec::new();
        let mut objects: Vec<StaticObject> = Vec::new();

        for idx in 0..biome_map.nrows() {
            for idy in 0..biome_map.ncols() {
                let b_key = biome_map[(idx, idy)];
                let land_name = &land_map[&b_key];
                let position = Vec2::new((idx as f32) * self.scaling, (idy as f32) * self.scaling);

                match land_name.as_str() {
                    "grass" => tiles.push(Tile::grass(position)),
                    "forest" => {
                        let (tile, object) = self.generate_forest(position);
                        tiles.push(tile);
                        if let Some(obj) = object {
                            objects.push(obj);
                        }
                    }
                    "crops" => {
                        let (tile, object) = self.generate_crops(position);
                        tiles.push(tile);
                        if let Some(obj) = object {
                            objects.push(obj);
                        }
                    }
                    "orchard" => {
                        let (tile, object) = self.generate_orchard(position);
                        tiles.push(tile);
                        if let Some(obj) = object {
                            objects.push(obj);
                        }
                    }
                    "water" => tiles.push(Tile::water(position)),
                    "sand" => tiles.push(Tile::sand(position)),
                    _ => println!("{} not recognized", land_name),
                }
            }
        }

        (tiles, objects)
    }

    fn generate_biome_map(&self) -> DMatrix<usize> {
        let mut biome_map = DMatrix::zeros(self.area[0], self.area[1]);
        let n_fields = ((self.area[0] * self.area[1]) as f32 * self.config.field_density) as usize;

        let kd_tree = self.random_kd_tree_clustering(n_fields);

        // Generate biome_map from kdtrees
        for idx in 0..self.area[0] {
            for idy in 0..self.area[1] {
                biome_map[(idx, idy)] = kd_tree
                    .nearest_one(&[idx as f32, idy as f32], &squared_euclidean)
                    .1;
            }
        }

        // Add water and beaches if needed
        if self.water_present {
            for idx in 0..self.area[0] {
                for idy in 0..self.area[1] {
                    let value = self.random_funcs.noise(
                        idx as f64,
                        idy as f64,
                        3.0,
                        Some(HashMap::from([(15, 1), (25, 1)])),
                        Some(true),
                    ) as f32;

                    if value < self.config.water_cutoff {
                        biome_map[(idx, idy)] = self.config.land_types.len() + 1;
                        if value > self.config.water_cutoff - self.config.beach_thickness {
                            biome_map[(idx, idy)] = self.config.land_types.len() + 2;
                        }
                    }
                }
            }
        }

        biome_map
    }

    fn random_kd_tree_clustering(&self, n_points: usize) -> KdTree<f32, 2> {
        let mut rng = rand_chacha::ChaCha8Rng::seed_from_u64(self.seed);
        let x_sampler = rand::distributions::Uniform::new(0, self.area[0]);
        let y_sampler = rand::distributions::Uniform::new(0, self.area[1]);
        let v_sampler = rand::distributions::Uniform::new(0, self.config.land_types.len());

        let mut tree = KdTree::new();

        for _ in 0..n_points {
            let x = x_sampler.sample(&mut rng) as f32;
            let y = y_sampler.sample(&mut rng) as f32;
            let value = v_sampler.sample(&mut rng);
            tree.add(&[x, y], value);
        }
        tree
    }

    fn generate_land_map(&self) -> HashMap<usize, String> {
        let mut land_map: HashMap<usize, String> = HashMap::new();
        for (index, value) in self.config.land_types.iter().enumerate() {
            land_map.insert(index, value.clone());
        }
        land_map.insert(self.config.land_types.len() + 1, "water".to_string());
        land_map.insert(self.config.land_types.len() + 2, "sand".to_string());
        land_map
    }

    fn generate_forest(&self, pos: Vec2) -> (Tile, Option<StaticObject>) {
        let tree_probability = self
            .random_funcs
            .sampler
            .sample(&mut self.random_funcs.rng.clone()) as f32;
        let object_placement = self.random_funcs.noise(
            pos[0] as f64,
            pos[1] as f64,
            6.0,
            Some(HashMap::from([(5, 1), (10, 1)])),
            Some(true),
        ) as f32;

        let (asset, so) = if object_placement < 0.2 {
            if tree_probability < self.config.forest_tree_density {
                (
                    "forest-leaves".to_string(),
                    Some(StaticObject {
                        name: "Evergreen".to_string(),
                        asset: "evergreen-fur".to_string(),
                        pos,
                    }),
                )
            } else {
                ("forest-leaves".to_string(), None)
            }
        } else if object_placement < 0.4 {
            if tree_probability < self.config.forest_tree_density {
                (
                    "forest-leaves".to_string(),
                    Some(StaticObject {
                        name: "Wilting".to_string(),
                        asset: "wilting-fur".to_string(),
                        pos,
                    }),
                )
            } else {
                ("forest-leaves".to_string(), None)
            }
        } else {
            ("forest-leaves".to_string(), None)
        };

        (
            Tile {
                name: "Forest".to_string(),
                asset,
                pos,
            },
            so,
        )
    }

    fn generate_crops(&self, pos: Vec2) -> (Tile, Option<StaticObject>) {
        let object_placement = self.random_funcs.noise(
            pos[0] as f64,
            pos[1] as f64,
            6.0,
            Some(HashMap::from([(5, 1), (10, 1)])),
            Some(true),
        ) as f32;

        let so = if object_placement < -0.25 {
            Some(StaticObject {
                name: "GreenBushel".to_string(),
                asset: "green-bushel".to_string(),
                pos,
            })
        } else if object_placement < 0.0 {
            Some(StaticObject {
                name: "RipeBushel".to_string(),
                asset: "ripe-bushel".to_string(),
                pos,
            })
        } else if object_placement < 0.10 {
            Some(StaticObject {
                name: "DeadBushel".to_string(),
                asset: "dead-bushel".to_string(),
                pos,
            })
        } else {
            None
        };

        let asset = if object_placement < -0.2 {
            "light-mud".to_string()
        } else {
            "mud".to_string()
        };

        (
            Tile {
                name: "Crops".to_string(),
                asset,
                pos,
            },
            so,
        )
    }

    fn generate_orchard(&self, pos: Vec2) -> (Tile, Option<StaticObject>) {
        let object_placement = self
            .random_funcs
            .sampler
            .sample(&mut self.random_funcs.rng.clone()) as f32;

        let so = if object_placement < self.config.orchard_tree_density {
            let tree_type = self
                .random_funcs
                .sampler
                .sample(&mut self.random_funcs.rng.clone()) as f32;
            if tree_type < 0.75 {
                Some(StaticObject {
                    name: "AppleTree".to_string(),
                    asset: "apple-tree".to_string(),
                    pos,
                })
            } else {
                Some(StaticObject {
                    name: "EmptyTree".to_string(),
                    asset: "pruned-tree".to_string(),
                    pos,
                })
            }
        } else {
            None
        };

        let asset = if object_placement
            < (self.config.orchard_flower_density + self.config.orchard_tree_density)
        {
            let flower_type = self
                .random_funcs
                .sampler
                .sample(&mut self.random_funcs.rng.clone()) as f32;
            if flower_type < 0.25 {
                "1-flower".to_string()
            } else if flower_type < 0.5 {
                "2-flowers".to_string()
            } else if flower_type < 0.75 {
                "4-flowers".to_string()
            } else {
                "6-flowers".to_string()
            }
        } else {
            "darker-grass".to_string()
        };

        (
            Tile {
                name: "Orchard".to_string(),
                asset,
                pos,
            },
            so,
        )
    }

    pub fn load_assets(&self, assets: Vec<PathBuf>) -> HashMap<String, Pixmap> {
        let mut asset_map: HashMap<String, Pixmap> = HashMap::new();
        for path in assets {
            let path_str = path.to_str().unwrap_or_default().to_string();
            match Pixmap::load_png(path) {
                Ok(pixmap) => {
                    let name: Vec<&str> = path_str.split('/').collect();
                    let name = name[name.len() - 1].to_string();
                    let name: Vec<&str> = name.split('.').collect();
                    let name = name[0].to_string();
                    asset_map.insert(name, pixmap);
                }
                Err(err) => {
                    eprintln!("Failed to load asset at {}: {}", path_str, err);
                }
            }
        }
        asset_map
    }
}
