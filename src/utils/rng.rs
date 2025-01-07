use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use serde::{Deserialize, Serialize};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

/// A simplified RNG manager that provides deterministic seeding
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RngManager {
    master_seed: u64,
}

impl RngManager {
    pub fn new(seed: u64) -> Self {
        Self { master_seed: seed }
    }

    pub fn master_seed(&self) -> u64 {
        self.master_seed
    }

    // Get a new RNG for a component by hashing its name with master seed
    pub fn get_rng(&self, name: &str) -> ChaCha8Rng {
        let mut hasher = DefaultHasher::new();
        self.master_seed.hash(&mut hasher);
        name.hash(&mut hasher);
        ChaCha8Rng::seed_from_u64(hasher.finish())
    }
}

// Simple trait for adding RNG to components
pub trait WithRng {
    fn with_rng(self, rng: ChaCha8Rng) -> Self;
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::Rng;

    #[test]
    fn test_rng_manager_consistency() {
        let rng_manager = RngManager::new(42);
        let component_name = "test_component";

        // First sequence
        let mut first_sequence = Vec::new();
        {
            let mut rng1 = rng_manager.get_rng(component_name);
            for _ in 0..5 {
                first_sequence.push(rng1.gen::<f64>());
            }
        }

        // Second sequence - should be different with current implementation
        let mut second_sequence = Vec::new();
        {
            let mut rng2 = rng_manager.get_rng(component_name);
            for _ in 0..5 {
                second_sequence.push(rng2.gen::<f64>());
            }
        }

        println!("First sequence:  {:?}", first_sequence);
        println!("Second sequence: {:?}", second_sequence);

        // This assertion will fail with current implementation
        assert_eq!(
            first_sequence, second_sequence,
            "RNG sequences should be identical for same seed and component name"
        );
    }

    #[test]
    fn test_rng_manager_different_components() {
        let rng_manager = RngManager::new(42);

        // Get sequences for two different component names
        let mut sequence1 = Vec::new();
        let mut sequence2 = Vec::new();

        {
            let mut rng1 = rng_manager.get_rng("component1");
            let mut rng2 = rng_manager.get_rng("component2");

            for _ in 0..5 {
                sequence1.push(rng1.gen::<f64>());
                sequence2.push(rng2.gen::<f64>());
            }
        }

        println!("Component1 sequence: {:?}", sequence1);
        println!("Component2 sequence: {:?}", sequence2);

        // Different components should get different sequences
        assert_ne!(
            sequence1, sequence2,
            "Different components should get different RNG sequences"
        );
    }

    #[test]
    fn test_rng_manager_reset_scenario() {
        let rng_manager = RngManager::new(42);
        let component_name = "aircraft_0";

        // First reset
        let mut state1 = {
            let mut rng = rng_manager.get_rng(component_name);
            let u1: f64 = rng.gen();
            let u2: f64 = rng.gen();
            (u1, u2)
        };

        // Second reset with same seed
        let mut state2 = {
            let mut rng = rng_manager.get_rng(component_name);
            let u1: f64 = rng.gen();
            let u2: f64 = rng.gen();
            (u1, u2)
        };

        println!("First reset state:  {:?}", state1);
        println!("Second reset state: {:?}", state2);

        assert_eq!(
            state1, state2,
            "Reset states should be identical for same seed"
        );
    }

    #[test]
    fn test_rng_manager_multiple_calls() {
        let rng_manager = RngManager::new(42);
        let component_name = "aircraft_0";
        let mut rng = rng_manager.get_rng(component_name);

        // First set of calls
        let state1 = (rng.gen::<f64>(), rng.gen::<f64>());

        // Next set of calls to same RNG
        let state2 = (rng.gen::<f64>(), rng.gen::<f64>());

        println!("First two values:  {:?}", state1);
        println!("Next two values: {:?}", state2);

        // These should be different as we're advancing the RNG
        assert_ne!(
            state1, state2,
            "Subsequent calls to same RNG should produce different values"
        );
    }
}
