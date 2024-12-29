use bevy::prelude::*;
use noise::NoiseFn;
use serde::{Deserialize, Serialize};
use std::ops::Range;

/// Represents a single layer of noise with its parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NoiseLayer {
    pub scale: f32,
    pub amplitude: f32,
    pub octaves: u32,
    pub persistence: f32,
    pub lacunarity: f32,
    pub offset: Vec2, // Allows for shifting different layers
    pub weight: f32,  // Importance of this layer in final result
}

impl NoiseLayer {
    pub fn new(scale: f32, amplitude: f32, octaves: u32) -> Self {
        Self {
            scale,
            amplitude,
            octaves,
            persistence: 0.5,
            lacunarity: 2.0,
            offset: Vec2::ZERO,
            weight: 1.0,
        }
    }

    // Builder pattern for optional parameters
    pub fn with_persistence(mut self, persistence: f32) -> Self {
        self.persistence = persistence;
        self
    }

    pub fn with_lacunarity(mut self, lacunarity: f32) -> Self {
        self.lacunarity = lacunarity;
        self
    }

    pub fn with_offset(mut self, offset: Vec2) -> Self {
        self.offset = offset;
        self
    }

    pub fn with_weight(mut self, weight: f32) -> Self {
        self.weight = weight;
        self
    }
}

/// Manages multiple noise layers for terrain generation
#[derive(Debug, Clone)]
pub struct NoiseGenerator {
    noise_fn: noise::OpenSimplex,
    pub layers: Vec<NoiseLayer>,
    value_range: Range<f32>,
}

impl NoiseGenerator {
    pub fn new(seed: u64) -> Self {
        Self {
            noise_fn: noise::OpenSimplex::new(seed as u32),
            layers: Vec::new(),
            value_range: 0.0..1.0,
        }
    }

    pub fn add_layer(&mut self, layer: NoiseLayer) {
        self.layers.push(layer);
    }

    pub fn set_value_range(&mut self, min: f32, max: f32) {
        self.value_range = min..max;
    }

    /// Generate noise value at a specific position considering all layers
    pub fn get_noise(&self, pos: Vec2) -> f32 {
        let mut total_value = 0.0;
        let mut total_weight = 0.0;

        for layer in &self.layers {
            let noise_value = self.generate_layered_noise(pos, layer);
            total_value += noise_value * layer.weight;
            total_weight += layer.weight;
        }

        // Normalize and map to desired range
        let normalized = if total_weight > 0.0 {
            total_value / total_weight
        } else {
            0.0
        };

        self.map_to_range(normalized)
    }

    /// Generate noise for a specific layer with all its parameters
    fn generate_layered_noise(&self, pos: Vec2, layer: &NoiseLayer) -> f32 {
        let mut value = 0.0;
        let mut amplitude = layer.amplitude;
        let mut frequency = 1.0;

        for _ in 0..layer.octaves {
            let sample_pos = (pos + layer.offset) * frequency / layer.scale;
            let noise_val = self
                .noise_fn
                .get([sample_pos.x as f64, sample_pos.y as f64]) as f32;

            value += noise_val * amplitude;
            amplitude *= layer.persistence;
            frequency *= layer.lacunarity;
        }

        // Normalize to [0, 1] range
        (value + layer.amplitude) / (0.4 + layer.amplitude)
    }

    fn map_to_range(&self, value: f32) -> f32 {
        self.value_range.start + (self.value_range.end - self.value_range.start) * value
    }
}
