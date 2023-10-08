#![warn(clippy::all)]

extern crate nalgebra as na;
use na::DMatrix;

use rand::prelude::*;
use rand::distributions::Distribution;
use rand::distributions::uniform::Uniform;
use rand_chacha::ChaCha8Rng;

use kiddo::KdTree;
use kiddo::distance::squared_euclidean;
use noise::{OpenSimplex, NoiseFn};

use serde::{Serialize, Deserialize};

use std::collections::HashMap;
use std::path::PathBuf;

use glam::Vec2;
use tiny_skia::*;

#[allow(dead_code)] 
pub struct TerrainConfig {
    name: String,
    field_density: f32,
    land_types: Vec<String>,
    water_cutoff: f32,
    beach_thickness: f32,
    forest_tree_density: f32,
    orchard_tree_density: f32,
    orchard_flower_density: f32
}

impl TerrainConfig {

    #[allow(dead_code)]
    pub fn update_name(&mut self){
        // Update the name of the TerrainConfig to be a unique string identifier
        
        // Create a string made up of the first letters of each string
        let land_letters: String = self.land_types
            .iter()
            .map(|s| s.chars().next().unwrap_or_default())
            .collect();

        self.name = format!("fd{}lt{}wc{}bt{}ftd{}otd{}ofd{}",
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
            land_types: ["grass", "forest", "crops", "orchard"].iter().map(|x| x.to_string()).collect::<Vec<String>>(),
            water_cutoff: -0.1,
            beach_thickness: 0.04,
            forest_tree_density: 0.6,
            orchard_tree_density: 0.1,
            orchard_flower_density: 0.1
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct Tile {
    pub name: String,  // name of the tile to use
    pub asset: String,  // name of the asset from the tile_map
    pub pos: Vec2  // position in [m] on the map
}

#[derive(Serialize, Deserialize)]
pub struct StaticObject {
    pub name: String,  // name of the static object
    pub asset: String, // name of the asset from the static object map
    pub pos: Vec2 // position in [m] on the map
}

pub struct RandomFuncs {
    simplex_noise: OpenSimplex,
    sampler: Uniform<f64>,
    rng: ChaCha8Rng
}

impl RandomFuncs {
    pub fn new(seed: u32) -> Self {

        Self {
            simplex_noise: OpenSimplex::new(seed),
            sampler: Uniform::new(0.0, 1.0),
            rng: ChaCha8Rng::seed_from_u64(seed as u64)
        }

    }
}

pub struct Terrain {
    pub seed: u64,
    pub area: Vec<usize>,
    pub scaling: f32,
    pub config: TerrainConfig,
    pub water_present: bool,
    pub random_funcs: RandomFuncs
}

impl Terrain {

    pub fn get_name(&mut self) -> String {
        
        self.config.update_name();
        let config_name = &self.config.name;

        let name = format!("seed{}_area0{}1{}_scaling{}_tconfig{}_wp{}",
            self.seed,
            self.area[0],
            self.area[1],
            self.scaling,
            config_name,
            self.water_present    
        );
        name
    }

    pub fn generate_map(&mut self) -> (Vec<Tile>, Vec<StaticObject>) {

        let biome_map = self.generate_biome_map();
        let land_map = self.generate_land_map();

        let mut tiles: Vec<Tile> = Vec::new();
        let mut objects: Vec<StaticObject> = Vec::new();

        for idx in 0..biome_map.nrows() {
            for idy in 0..biome_map.ncols() {
                let b_key = biome_map[(idx, idy)];
                let land_name = &land_map[&b_key];
                let position = Vec2::new((idx  as f32) * self.scaling, (idy as f32) * self.scaling);
                match land_name.as_str() {
                    "grass" => {
                        let tile = self.grass(position);
                        tiles.push(tile);
                    },
                    "forest" => {
                        let (tile, object) = self.forest(position);
                        tiles.push(tile);
                        if let Some(value) = object {
                            objects.push(value)
                        };
                    },
                    "crops" => {
                        let (tile, object) = self.crops(position);
                        tiles.push(tile);
                        if let Some(value) = object {
                            objects.push(value)
                        };
                    },
                    "orchard" => {
                        let (tile, object) = self.orchard(position);
                        tiles.push(tile);
                        if let Some(value) = object {
                            objects.push(value)
                        };
                    },
                    "water" => tiles.push(self.water(position)),
                    "sand" => tiles.push(self.sand(position)),
                    _ => println!("{}, not recognized", land_name)
                }
            }
        }

        (tiles, objects)
    }

    fn generate_biome_map(&self) -> DMatrix<usize> {

        let mut biome_map = DMatrix::zeros(self.area[0], self.area[1]);

        let n_fields = ((self.area[0] * self.area[1]) as f32 * self.config.field_density) as usize;

        let kd_tree = self.random_kd_tree_clustering(n_fields);
        // let test_point = kd_tree.nearest_one(&[0.0, 0.0], &squared_euclidean);
        
        // Generate biome_map from kdtrees
        // TODO: Find way to speed this up!
        for idx in 0..self.area[0]{
            for idy in 0..self.area[1] {
                biome_map[(idx, idy)] = kd_tree.nearest_one(&[idx as f32, idy as f32], &squared_euclidean).1;
            };
        };

        // Add islands
        if self.water_present{
            for idx in 0..self.area[0] {
                for idy in 0..self.area[1] {
                    let value = self.noise(idx as f64, idy as f64, 3.0, Some(HashMap::from([(15, 1), (25, 1)])), Some(true)) as f32;
                    if value < self.config.water_cutoff {
                        biome_map[(idx, idy)] = self.config.land_types.len()+1;
                        if value > self.config.water_cutoff - self.config.beach_thickness{
                            biome_map[(idx, idy)] = self.config.land_types.len()+2;
                        }
                    }
                }
            }
        }
        
        // println!("{}", biome_map);
        biome_map

    }

    fn random_kd_tree_clustering(&self, n_points: usize) -> KdTree<f32, 2>{
        let mut rng = ChaCha8Rng::seed_from_u64(self.seed);
        let x_sampler = Uniform::new(0, self.area[0]);
        let y_sampler = Uniform::new(0, self.area[1]);
        let v_sampler = Uniform::new(0, self.config.land_types.len());  // TODO: Find a way to get usize of the land_types
        
        let mut tree = KdTree::new();

        for _ in 0..n_points {
            let x = x_sampler.sample(&mut rng) as f32;
            let y = y_sampler.sample(&mut rng) as f32;
            let value = v_sampler.sample(&mut rng);
            tree.add(&[x, y], value);
        };
        tree
    }

    fn generate_land_map(&self) -> HashMap<usize, String> {

        let mut land_map: HashMap<usize, String> = HashMap::new();
        for (index, value) in self.config.land_types.iter().enumerate() {
            land_map.insert(index, value.clone());
        }
        land_map.insert(self.config.land_types.len()+1, "water".to_string());
        land_map.insert(self.config.land_types.len()+2, "sand".to_string());
        land_map
    }

    pub fn load_assets(&self, assets: Vec<PathBuf>) -> HashMap<String, Pixmap> {
        let mut asset_map: HashMap<String, Pixmap> = HashMap::new();
        for path in assets {
            let path_str = path.to_str().unwrap_or_default().to_string();
            match Pixmap::load_png(path) {
                Ok(pixmap) => {
                    let name: Vec<&str> = path_str.split('/').collect();
                    let name = name[name.len()-1].to_string();
                    let name: Vec<&str> = name.split('.').collect();
                    let name = name[0].to_string();
                    // println!("name: {}", name);
                    asset_map.insert(name, pixmap);
                }
                Err(err) => {
                    println!("Path is: {}", path_str);
                    eprintln!("Error {}", err);
                }
            }
        }
        
        asset_map
    }

    fn grass(&self, pos: Vec2) -> Tile {

        Tile {
            name: "Grass".to_string(),
            asset: "grass".to_string(),
            pos
        }

    }

    fn sand(&self, pos: Vec2) -> Tile {
        
        Tile {
            name: "Sand".to_string(),
            asset: "sand".to_string(),
            pos
        }

    }

    fn forest(&mut self, pos: Vec2) -> (Tile, Option<StaticObject>) {
        let tree_probability = self.random_funcs.sampler.sample(&mut self.random_funcs.rng) as f32;
        let object_placement = self.noise(pos[0] as f64, pos[1] as f64, 6.0, Some(HashMap::from([(5, 1), (10, 1)])), Some(true)) as f32;

        let (asset, so) = if object_placement < 0.2 {
            if tree_probability < self.config.forest_tree_density {
                // Add an evergreen fur
                (
                    "forest-leaves".to_string(),
                    Some(StaticObject{
                        name: "Evergreen".to_string(),
                        asset: "evergreen-fur".to_string(),
                        pos
                    })
                )
            } else {
                ("forest-leaves".to_string(), None)
            }
        } else if object_placement < 0.4 {
            if tree_probability < self.config.forest_tree_density {
                // Add a wilting fur
                (
                    "forest-leaves".to_string(),
                    Some(StaticObject{
                        name: "Wilting".to_string(),
                        asset: "wilting-fur".to_string(),
                        pos
                    })
                )
            } else {
                ("forest-leaves".to_string(), None)
            }
        } else {
            ("forest-leaves".to_string(), None)
        };

        let tile = Tile {
            name: "Forest".to_string(),
            asset,
            pos
        };

        (tile, so)

    }

    fn crops(&self, pos: Vec2) -> (Tile, Option<StaticObject>) {
        let object_placement = self.noise(pos[0] as f64, pos[1] as f64, 6.0, Some(HashMap::from([(5, 1), (10, 1)])), Some(true)) as f32;
        
        let so = if object_placement < -0.25 {
            // Add green bushel
            Some(StaticObject {
                name: "GreenBushel".to_string(),
                asset: "green-bushel".to_string(),
                pos
            })
        } else if object_placement < 0.0 {
            // Add ripe bushel
            Some(StaticObject {
                name: "RipeBushel".to_string(),
                asset: "ripe-bushel".to_string(),
                pos
            })
        } else if object_placement < 0.10 {
            // Add dead bushel
            Some(StaticObject {
                name: "DeadBushel".to_string(),
                asset: "dead-bushel".to_string(),
                pos
            })
        } else {
            None
        };

        let asset = if object_placement < -0.2 {
            "light-mud".to_string()
        } else {
            "mud".to_string()
        };

        let tile = Tile {
            name: "Crops".to_string(),
            asset,
            pos
        };

        (tile, so)

    }

    fn orchard(&mut self, pos: Vec2) -> (Tile, Option<StaticObject>) {
        let object_placement = self.random_funcs.sampler.sample(&mut self.random_funcs.rng) as f32;
        // println!("object_placement: {}", object_placement);
        let so = if object_placement < self.config.orchard_tree_density {
            let tree_type = self.random_funcs.sampler.sample(&mut self.random_funcs.rng) as f32;
            if tree_type < 0.75 {
                // Add apple tree
                Some(StaticObject{
                    name: "AppleTree".to_string(),
                    asset: "apple-tree".to_string(),
                    pos
                })
            } else {
                // Add empty tree
                Some(StaticObject{
                    name: "EmptyTree".to_string(),
                    asset: "pruned-tree".to_string(),
                    pos
                })
            }
        } else {
            None
        };

        let asset = if object_placement < (self.config.orchard_flower_density + self.config.orchard_tree_density) {
            let flower_type = self.random_funcs.sampler.sample(&mut self.random_funcs.rng.clone()) as f32;
            // println!("flower_type: {}", flower_type);
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

        let tile = Tile {
            name: "Orchard".to_string(),
            asset,
            pos
        };

        (tile, so)

    }

    fn water(&self, pos: Vec2) -> Tile {
        Tile {
            name: "Water".to_string(),
            asset: "water".to_string(),
            pos
        }
    }
    
    fn noise(&self, x: f64, y: f64, z: f64, sizes: Option<HashMap<i32, i32>>, normalize: Option<bool>) -> f64 {
        let mut value = 0.0;
        
        if let Some(sizes) = &sizes {
            for (size, weight) in sizes.iter() {
                value += (*weight as f64) * (self.random_funcs.simplex_noise.get([x / (*size as f64), y / (*size as f64), z]));
            }
        }

        if normalize.unwrap_or(true) {
            let sum: i32 = sizes.as_ref().unwrap_or(&HashMap::from([(0, 1)]))
                .values()
                .cloned()
                .sum::<i32>();
            value /= sum as f64;
        }

        value
    }

}
