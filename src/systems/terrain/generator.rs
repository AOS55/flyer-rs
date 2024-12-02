use crate::systems::terrain::noise::NoiseGenerator;
use glam::{UVec2, Vec2};
use kiddo::distance::squared_euclidean;
use kiddo::KdTree;
use rand::prelude::*;
use std::collections::HashMap;

use crate::components::terrain::*;
use crate::components::CameraComponent;
use crate::ecs::{Result, System, World};

pub struct TerrainGeneratorSystem {
    noise_gen: NoiseGenerator,
    rng: StdRng,
    kd_tree: KdTree<f32, 2>,
}

impl TerrainGeneratorSystem {
    pub fn new(seed: u64) -> Self {
        Self {
            noise_gen: NoiseGenerator::new(seed),
            rng: StdRng::seed_from_u64(seed),
            kd_tree: KdTree::new(),
        }
    }

    fn generate_chunk(&mut self, pos: UVec2, terrain: &TerrainComponent) -> TerrainChunk {
        let mut chunk = TerrainChunk::new(terrain.chunk_size);
        let chunk_pos = Vec2::new(
            pos.x as f32 * terrain.chunk_size as f32,
            pos.y as f32 * terrain.chunk_size as f32,
        );

        // Generate height and moisture maps
        let (height_map, moisture_map) =
            self.noise_gen
                .generate_terrain_maps(chunk_pos, terrain.chunk_size, &terrain.config);

        chunk.height_map = height_map;
        chunk.moisture_map = moisture_map;

        // Rest of chunk generation...
        self.generate_biomes(&mut chunk, pos, terrain);
        self.place_features(&mut chunk, pos, terrain);

        chunk
    }
}

impl TerrainGeneratorSystem {
    fn generate_biomes(
        &mut self,
        chunk: &mut TerrainChunk,
        pos: UVec2,
        terrain: &TerrainComponent,
    ) {
        let size = terrain.chunk_size as usize;
        let chunk_pos = Vec2::new(
            pos.x as f32 * terrain.chunk_size as f32,
            pos.y as f32 * terrain.chunk_size as f32,
        );

        // Generate biome cluster points if not already generated
        self.ensure_biome_clusters(terrain);

        // Process each tile in the chunk
        for y in 0..size {
            for x in 0..size {
                let world_pos = Vec2::new(chunk_pos.x + x as f32, chunk_pos.y + y as f32);

                let height = chunk.height_map[y * size + x];
                let moisture = chunk.moisture_map[y * size + x];

                // Determine biome based on height, moisture, and nearest clusters
                chunk.biome_map[y * size + x] =
                    self.determine_biome(world_pos, height, moisture, terrain);
            }
        }
    }

    fn select_weighted_biome(&mut self, config: &TerrainGenConfig) -> BiomeType {
        let total_weight: f32 = config.biome_weights.values().sum();
        let mut random = self.rng.gen::<f32>() * total_weight;

        for (biome, weight) in &config.biome_weights {
            random -= weight;
            if random <= 0.0 {
                return *biome;
            }
        }

        BiomeType::Grass // Default fallback
    }

    fn biome_to_index(&self, biome: BiomeType) -> usize {
        match biome {
            BiomeType::Grass => 0,
            BiomeType::Forest => 1,
            BiomeType::Crops => 2,
            BiomeType::Orchard => 3,
            BiomeType::Water => 4,
            BiomeType::Sand => 5,
        }
    }

    fn index_to_biome(&self, index: usize) -> BiomeType {
        match index {
            0 => BiomeType::Grass,
            1 => BiomeType::Forest,
            2 => BiomeType::Crops,
            3 => BiomeType::Orchard,
            4 => BiomeType::Water,
            5 => BiomeType::Sand,
            _ => BiomeType::Grass, // Default fallback
        }
    }

    fn ensure_biome_clusters(&mut self, terrain: &TerrainComponent) {
        if self.kd_tree.size() == 0 {
            let world_area = (terrain.world_size.x * terrain.world_size.y) as f32;
            let n_clusters = (world_area * terrain.config.field_density) as usize;

            for _ in 0..n_clusters {
                let x = self.rng.gen_range(0.0..terrain.world_size.x as f32);
                let y = self.rng.gen_range(0.0..terrain.world_size.y as f32);

                let biome = self.select_weighted_biome(&terrain.config);
                // Convert BiomeType to usize for KD-tree storage
                let biome_index = self.biome_to_index(biome);

                self.kd_tree.add(&[x, y], biome_index);
            }
        }
    }

    fn determine_biome(
        &self,
        world_pos: Vec2,
        height: f32,
        moisture: f32,
        terrain: &TerrainComponent,
    ) -> BiomeType {
        // Check for water and beach first based on height
        if height < terrain.config.water_threshold {
            return BiomeType::Water;
        }
        if height < terrain.config.water_threshold + terrain.config.beach_width {
            return BiomeType::Sand;
        }

        // Find nearest biome cluster
        let nearest = self
            .kd_tree
            .nearest_one(&[world_pos.x, world_pos.y], &squared_euclidean);
        let base_biome = self.index_to_biome(nearest.1);

        // Calculate probabilities for nearby biomes
        let mut biome_scores: HashMap<BiomeType, f32> = HashMap::new();

        // Score the base biome
        let base_prob = terrain
            .config
            .get_biome_probability(height, moisture, base_biome);
        biome_scores.insert(base_biome, base_prob);

        // Score potential transitions
        for (biome, _) in &terrain.config.biome_weights {
            if *biome != base_biome {
                let prob = terrain
                    .config
                    .get_biome_probability(height, moisture, *biome);
                biome_scores.insert(*biome, prob * 0.5);
            }
        }

        // Select highest scoring biome
        biome_scores
            .into_iter()
            .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
            .map(|(biome, _)| biome)
            .unwrap_or(base_biome)
    }
}

impl TerrainGeneratorSystem {
    fn place_features(&mut self, chunk: &mut TerrainChunk, pos: UVec2, terrain: &TerrainComponent) {
        let size = terrain.chunk_size as usize;
        let chunk_world_pos = terrain.chunk_to_world_pos(pos);

        for y in 0..size {
            for x in 0..size {
                let local_pos = Vec2::new(x as f32, y as f32) * terrain.scale;
                let world_pos = chunk_world_pos + local_pos;

                let biome = chunk.biome_map[y * size + x];
                let height = chunk.height_map[y * size + x];
                let moisture = chunk.moisture_map[y * size + x];

                // Get variation noise for this position
                let variation = self.noise_gen.get_feature_variation(world_pos, 30.0);

                // Try to place features based on biome and local conditions
                if let Some(feature) =
                    self.generate_feature(biome, world_pos, height, moisture, variation, terrain)
                {
                    chunk.features.push(feature);
                }
            }
        }
    }

    fn generate_feature(
        &mut self,
        biome: BiomeType,
        pos: Vec2,
        height: f32,
        moisture: f32,
        variation: f32,
        terrain: &TerrainComponent,
    ) -> Option<TerrainFeature> {
        match biome {
            BiomeType::Forest => self.generate_forest_feature(pos, variation, terrain),
            BiomeType::Orchard => self.generate_orchard_feature(pos, variation, terrain),
            BiomeType::Crops => self.generate_crop_feature(pos, moisture, variation, terrain),
            BiomeType::Grass => self.generate_grass_feature(pos, moisture, variation, terrain),
            _ => None,
        }
    }

    fn generate_forest_feature(
        &mut self,
        pos: Vec2,
        variation: f32,
        terrain: &TerrainComponent,
    ) -> Option<TerrainFeature> {
        let tree_density = terrain.config.get_feature_density(
            FeatureType::Tree {
                variant: TreeVariant::EvergreenFir,
            },
            BiomeType::Forest,
        );

        if self.rng.gen::<f32>() > tree_density {
            return None;
        }

        // Use variation to determine tree type
        let (variant, asset) = if variation > 0.3 {
            (TreeVariant::EvergreenFir, "evergreen-fur")
        } else if variation > 0.0 {
            (TreeVariant::WiltingFir, "wilting-fur")
        } else {
            (TreeVariant::EvergreenFir, "evergreen-fur")
        };

        Some(TerrainFeature {
            name: "tree".to_string(),
            asset: asset.to_string(),
            feature_type: FeatureType::Tree { variant },
            position: pos,
            rotation: self.noise_gen.get_feature_rotation(pos),
            scale: 0.8 + variation.abs() * 0.4, // Scale variation 0.8-1.2
        })
    }

    fn generate_orchard_feature(
        &mut self,
        pos: Vec2,
        variation: f32,
        terrain: &TerrainComponent,
    ) -> Option<TerrainFeature> {
        let tree_density = terrain.config.get_feature_density(
            FeatureType::Tree {
                variant: TreeVariant::AppleTree,
            },
            BiomeType::Orchard,
        );

        if self.rng.gen::<f32>() > tree_density {
            return None;
        }

        let variant = if variation > 0.0 {
            (TreeVariant::AppleTree, "apple-tree")
        } else {
            (TreeVariant::PrunedTree, "pruned-tree")
        };

        Some(TerrainFeature {
            name: "orchard_tree".to_string(),
            asset: variant.1.to_string(),
            feature_type: FeatureType::Tree { variant: variant.0 },
            position: pos,
            rotation: self.noise_gen.get_feature_rotation(pos),
            scale: 1.0,
        })
    }

    fn generate_crop_feature(
        &mut self,
        pos: Vec2,
        moisture: f32,
        variation: f32,
        terrain: &TerrainComponent,
    ) -> Option<TerrainFeature> {
        let bush_density = terrain.config.get_feature_density(
            FeatureType::Bush {
                variant: BushVariant::GreenBushel,
            },
            BiomeType::Crops,
        );

        if self.rng.gen::<f32>() > bush_density {
            return None;
        }

        // Use moisture and variation to determine bush type
        let variant = if moisture > 0.6 {
            (BushVariant::GreenBushel, "green-bushel")
        } else if variation > 0.0 {
            (BushVariant::RipeBushel, "ripe-bushel")
        } else {
            (BushVariant::DeadBushel, "dead-bushel")
        };

        Some(TerrainFeature {
            name: "bush".to_string(),
            asset: variant.1.to_string(),
            feature_type: FeatureType::Bush { variant: variant.0 },
            position: pos,
            rotation: 0.0,
            scale: 1.0,
        })
    }

    fn generate_grass_feature(
        &mut self,
        pos: Vec2,
        moisture: f32,
        variation: f32,
        terrain: &TerrainComponent,
    ) -> Option<TerrainFeature> {
        let flower_density = terrain.config.get_feature_density(
            FeatureType::Flower {
                variant: FlowerVariant::Single,
            },
            BiomeType::Grass,
        );

        if self.rng.gen::<f32>() > flower_density || moisture < 0.4 {
            return None;
        }

        // Use variation to determine flower type
        let variant = if variation > 0.5 {
            (FlowerVariant::Cluster, "flower-cluster")
        } else if variation > 0.0 {
            (FlowerVariant::Double, "flower-double")
        } else if variation > -0.5 {
            (FlowerVariant::Quad, "flower-quad")
        } else {
            (FlowerVariant::Single, "flower-single")
        };

        Some(TerrainFeature {
            name: "flower".to_string(),
            asset: variant.1.to_string(),
            feature_type: FeatureType::Flower { variant: variant.0 },
            position: pos,
            rotation: self.noise_gen.get_feature_rotation(pos),
            scale: 0.9 + variation.abs() * 0.2,
        })
    }
}

impl System for TerrainGeneratorSystem {
    fn name(&self) -> &str {
        "Terrain Generator System"
    }

    fn run(&mut self, world: &mut World) -> Result<()> {
        // Get visible chunks based on camera position
        let camera = world.get_resource::<CameraComponent>()?;
        let visible_chunks = TerrainComponent::get_visible_chunks(camera);

        for (_, terrain) in world.query_mut::<TerrainComponent>() {
            // Generate missing chunks
            for chunk_pos in &visible_chunks {
                if !terrain.chunks.contains_key(chunk_pos) {
                    let chunk = self.generate_chunk(*chunk_pos, terrain);
                    terrain.chunks.insert(*chunk_pos, chunk);
                }
            }

            // Update active chunks list
            terrain.active_chunks = visible_chunks.clone().into_iter().collect();

            // Clean up far chunks
            terrain.chunks.retain(|pos, _| visible_chunks.contains(pos));
        }

        Ok(())
    }

    fn dependencies(&self) -> Vec<&str> {
        vec!["Camera System"]
    }
}

mod tests {
    use super::*;
    use crate::components::CameraComponent;
    use crate::ecs::EntityId;
    use glam::Vec2;

    fn setup_test_world() -> (World, EntityId) {
        let mut world = World::new();

        // Add camera
        let camera = CameraComponent {
            position: Vec2::ZERO,
            viewport: Vec2::new(800.0, 600.0),
            zoom: 1.0,
            ..Default::default()
        };
        world.add_resource(camera);

        // Add terrain component
        let terrain_entity = world.spawn();
        let terrain = TerrainComponent::new(UVec2::new(1000, 1000), 32, 12345, 1.0);
        world.add_component(terrain_entity, terrain).unwrap();

        (world, terrain_entity)
    }

    #[test]
    fn test_chunk_generation() {
        let (mut world, terrain_entity) = setup_test_world();
        let mut system = TerrainGeneratorSystem::new(12345);

        // Run system
        system.run(&mut world).unwrap();

        // Check if chunks were generated
        let terrain = world
            .get_component::<TerrainComponent>(terrain_entity)
            .unwrap();
        assert!(!terrain.chunks.is_empty());
    }

    #[test]
    fn test_chunk_feature_generation() {
        let (mut world, terrain_entity) = setup_test_world();
        let mut system = TerrainGeneratorSystem::new(12345);

        // Generate a single chunk
        let chunk_pos = UVec2::ZERO;
        let chunk = system.generate_chunk(
            chunk_pos,
            world
                .get_component::<TerrainComponent>(terrain_entity)
                .unwrap(),
        );

        // Verify chunk properties
        assert!(!chunk.height_map.is_empty());
        assert!(!chunk.moisture_map.is_empty());
        assert!(!chunk.biome_map.is_empty());
    }

    #[test]
    fn test_biome_generation() {
        let (mut world, terrain_entity) = setup_test_world();
        let mut system = TerrainGeneratorSystem::new(12345);

        // Generate a chunk
        let chunk_pos = UVec2::ZERO;
        let terrain = world
            .get_component::<TerrainComponent>(terrain_entity)
            .unwrap();
        let chunk = system.generate_chunk(chunk_pos, terrain);

        // Verify biome distribution
        let biome_counts: HashMap<BiomeType, usize> =
            chunk
                .biome_map
                .iter()
                .fold(HashMap::new(), |mut acc, &biome| {
                    *acc.entry(biome).or_insert(0) += 1;
                    acc
                });

        // Check for land biomes
        assert!(
            biome_counts.contains_key(&BiomeType::Grass)
                || biome_counts.contains_key(&BiomeType::Forest)
                || biome_counts.contains_key(&BiomeType::Crops),
            "Should have at least one land biome"
        );

        // Verify total tile count
        let total_tiles: usize = biome_counts.values().sum();
        assert_eq!(
            total_tiles,
            (terrain.chunk_size * terrain.chunk_size) as usize
        );
    }

    #[test]
    fn test_feature_placement() {
        let (mut world, terrain_entity) = setup_test_world();
        let mut system = TerrainGeneratorSystem::new(12345);

        // Generate a chunk
        let chunk_pos = UVec2::ZERO;
        let chunk = system.generate_chunk(
            chunk_pos,
            world
                .get_component::<TerrainComponent>(terrain_entity)
                .unwrap(),
        );

        // Check feature placement
        let forest_tiles = chunk
            .biome_map
            .iter()
            .filter(|&&biome| biome == BiomeType::Forest)
            .count();

        // If we have forest tiles, we should have some trees
        if forest_tiles > 0 {
            let tree_features = chunk
                .features
                .iter()
                .filter(|f| matches!(f.feature_type, FeatureType::Tree { .. }))
                .count();
            assert!(tree_features > 0);
        }
    }

    #[test]
    fn test_terrain_persistence() {
        let (mut world, terrain_entity) = setup_test_world();
        let mut system = TerrainGeneratorSystem::new(12345);

        // Generate initial chunks
        system.run(&mut world).unwrap();

        // Store chunk data
        let initial_chunks: HashMap<UVec2, TerrainChunk> = world
            .get_component::<TerrainComponent>(terrain_entity)
            .unwrap()
            .chunks
            .clone();

        // Run system again
        system.run(&mut world).unwrap();

        // Compare chunks
        let current_chunks = &world
            .get_component::<TerrainComponent>(terrain_entity)
            .unwrap()
            .chunks;

        // Verify chunks haven't changed
        for (pos, chunk) in &initial_chunks {
            assert!(current_chunks.contains_key(pos));
            assert_eq!(
                chunk.height_map,
                current_chunks.get(pos).unwrap().height_map
            );
        }
    }
}
