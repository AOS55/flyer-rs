use std::time::{Duration, Instant};

pub struct TimeManager {
    start_time: Instant,
    last_update: Instant,
    delta_time: Duration,
    elapsed_time: Duration,
    time_scale: f64,
    frame_count: u64,
}

impl TimeManager {
    pub fn new() -> Self {
        let now = Instant::now();
        Self {
            start_time: now,
            last_update: now,
            delta_time: Duration::ZERO,
            elapsed_time: Duration::ZERO,
            time_scale: 1.0,
            frame_count: 0,
        }
    }

    pub fn update(&mut self) {
        let now = Instant::now();
        self.delta_time = now - self.last_update;
        self.elapsed_time = now - self.start_time;
        self.last_update = now;
        self.frame_count += 1;
    }

    pub fn delta_seconds(&self) -> f64 {
        self.delta_time.as_secs_f64() * self.time_scale
    }

    pub fn elapsed_seconds(&self) -> f64 {
        self.elapsed_time.as_secs_f64()
    }

    pub fn set_time_scale(&mut self, scale: f64) {
        self.time_scale = scale.max(0.0);
    }

    pub fn fps(&self) -> f64 {
        if self.elapsed_time.as_secs_f64() > 0.0 {
            self.frame_count as f64 / self.elapsed_time.as_secs_f64()
        } else {
            0.0
        }
    }

    pub fn reset(&mut self) {
        let now = Instant::now();
        self.start_time = now;
        self.last_update = now;
        self.delta_time = Duration::ZERO;
        self.elapsed_time = Duration::ZERO;
        self.frame_count = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_time_manager_initialization() {
        let time_manager = TimeManager::new();
        assert_eq!(time_manager.delta_seconds(), 0.0);
        assert_eq!(time_manager.frame_count, 0);
    }

    #[test]
    fn test_time_manager_update() {
        let mut time_manager = TimeManager::new();
        thread::sleep(Duration::from_millis(16)); // Simulate one frame
        time_manager.update();

        assert!(time_manager.delta_seconds() > 0.0);
        assert_eq!(time_manager.frame_count, 1);
    }

    #[test]
    fn test_time_scale() {
        let mut time_manager = TimeManager::new();
        time_manager.set_time_scale(2.0);
        thread::sleep(Duration::from_millis(16));
        time_manager.update();

        let normal_delta = time_manager.delta_time.as_secs_f64();
        let scaled_delta = time_manager.delta_seconds();
        assert!(scaled_delta > normal_delta);
    }

    #[test]
    fn test_reset() {
        let mut time_manager = TimeManager::new();
        thread::sleep(Duration::from_millis(16));
        time_manager.update();
        time_manager.reset();

        assert_eq!(time_manager.frame_count, 0);
        assert_eq!(time_manager.delta_seconds(), 0.0);
    }
}
