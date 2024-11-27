use glam::Vec2;
use kiddo::distance::squared_euclidean;
use kiddo::KdTree;
use nalgebra::DMatrix;
use noise::{NoiseFn, OpenSimplex};
use rand::distributions::uniform::Uniform;
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;
use std::collections::HashMap;

/// Random number generation and noise utilities for terrain generation
pub struct RandomFuncs {
    pub simplex_noise: OpenSimplex,
    pub sampler: Uniform<f64>,
    pub rng: ChaCha8Rng,
}

impl RandomFuncs {
    /// Create a new RandomFuncs instance with the given seed
    pub fn new(seed: u32) -> Self {
        Self {
            simplex_noise: OpenSimplex::new(seed),
            sampler: Uniform::new(0.0, 1.0),
            rng: ChaCha8Rng::seed_from_u64(seed as u64),
        }
    }

    /// Generate noise value for given coordinates
    ///
    /// # Arguments
    /// * `x` - x coordinate
    /// * `y` - y coordinate
    /// * `z` - z coordinate
    /// * `sizes` - HashMap of octave sizes and weights
    /// * `normalize` - Whether to normalize the output
    pub fn noise(
        &self,
        x: f64,
        y: f64,
        z: f64,
        sizes: Option<HashMap<i32, i32>>,
        normalize: Option<bool>,
    ) -> f64 {
        let mut value = 0.0;

        if let Some(sizes) = &sizes {
            for (size, weight) in sizes.iter() {
                value += (*weight as f64)
                    * (self
                        .simplex_noise
                        .get([x / (*size as f64), y / (*size as f64), z]));
            }
        }

        if normalize.unwrap_or(true) {
            let sum: i32 = sizes
                .as_ref()
                .unwrap_or(&HashMap::from([(0, 1)]))
                .values()
                .cloned()
                .sum();
            value /= sum as f64;
        }

        value
    }
}

/// Generate a random KD-tree clustering for terrain biome generation
pub fn generate_random_kd_tree(
    seed: u64,
    area: &[usize],
    n_points: usize,
    n_biomes: usize,
) -> KdTree<f32, 2> {
    let mut rng = ChaCha8Rng::seed_from_u64(seed);
    let x_sampler = Uniform::new(0, area[0]);
    let y_sampler = Uniform::new(0, area[1]);
    let v_sampler = Uniform::new(0, n_biomes);

    let mut tree = KdTree::new();

    for _ in 0..n_points {
        let x = x_sampler.sample(&mut rng) as f32;
        let y = y_sampler.sample(&mut rng) as f32;
        let value = v_sampler.sample(&mut rng);
        tree.add(&[x, y], value);
    }
    tree
}

/// Generate a biome map using KD-tree clustering
pub fn generate_biome_map(
    tree: &KdTree<f32, 2>,
    area: &[usize],
    water_present: bool,
    random_funcs: &RandomFuncs,
    water_cutoff: f32,
    beach_thickness: f32,
    n_land_types: usize,
) -> DMatrix<usize> {
    let mut biome_map = DMatrix::zeros(area[0], area[1]);

    // Generate initial biome map from kdtree
    for idx in 0..area[0] {
        for idy in 0..area[1] {
            biome_map[(idx, idy)] = tree
                .nearest_one(&[idx as f32, idy as f32], &squared_euclidean)
                .1;
        }
    }

    // Add water and beaches if enabled
    if water_present {
        for idx in 0..area[0] {
            for idy in 0..area[1] {
                let value = random_funcs.noise(
                    idx as f64,
                    idy as f64,
                    3.0,
                    Some(HashMap::from([(15, 1), (25, 1)])),
                    Some(true),
                ) as f32;

                if value < water_cutoff {
                    biome_map[(idx, idy)] = n_land_types + 1;
                    if value > water_cutoff - beach_thickness {
                        biome_map[(idx, idy)] = n_land_types + 2;
                    }
                }
            }
        }
    }

    biome_map
}

/// Check if a point is inside a polygon
pub fn is_point_inside_polygon(point: Vec2, polygon_points: &[Vec2]) -> bool {
    let n = polygon_points.len();
    let mut inside = false;
    let mut idy: usize = n - 1;

    for idx in 0..n {
        if (polygon_points[idx].y < point.y && polygon_points[idy].y >= point.y)
            || (polygon_points[idy].y < point.y && polygon_points[idx].y >= point.y)
        {
            if polygon_points[idx].x
                + (point.y - polygon_points[idx].y)
                    / (polygon_points[idy].y - polygon_points[idx].y)
                    * (polygon_points[idy].x - polygon_points[idx].x)
                < point.x
            {
                inside = !inside;
            }
        }
        idy = idx;
    }

    inside
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_noise_generation() {
        let random_funcs = RandomFuncs::new(42);
        let noise_value = random_funcs.noise(
            1.0,
            1.0,
            1.0,
            Some(HashMap::from([(10, 1), (20, 1)])),
            Some(true),
        );
        assert!(-1.0 <= noise_value && noise_value <= 1.0);
    }

    #[test]
    fn test_point_in_polygon() {
        let polygon = vec![
            Vec2::new(0.0, 0.0),
            Vec2::new(1.0, 0.0),
            Vec2::new(1.0, 1.0),
            Vec2::new(0.0, 1.0),
        ];

        // Test point inside
        assert!(is_point_inside_polygon(Vec2::new(0.5, 0.5), &polygon));

        // Test point outside
        assert!(!is_point_inside_polygon(Vec2::new(2.0, 2.0), &polygon));
    }

    #[test]
    fn test_kd_tree_generation() {
        let area = vec![100, 100];
        let tree = generate_random_kd_tree(42, &area, 10, 4);

        // Test point query
        let (_, value) = tree.nearest_one(&[50.0, 50.0], &squared_euclidean);
        assert!(value < 4); // Should be within number of biomes
    }
}
