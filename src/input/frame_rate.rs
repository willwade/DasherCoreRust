//! Frame rate management for dynamic filters
//!
//! This module provides a frame rate manager that tracks frame times and calculates
//! the number of steps needed to maintain a consistent bit rate.

use std::time::Instant;

/// Frame rate manager for dynamic filters
#[derive(Debug)]
pub struct FrameRate {
    /// Last frame time
    last_frame_time: Option<Instant>,

    /// Average time between frames (in seconds)
    average_frame_time: f64,

    /// Target bit rate (bits per second)
    target_bit_rate: f64,

    /// Minimum number of steps
    min_steps: i32,
}

impl Default for FrameRate {
    fn default() -> Self {
        Self::new(10.0, 1)
    }
}

impl FrameRate {
    /// Create a new frame rate manager
    ///
    /// # Arguments
    ///
    /// * `target_bit_rate` - Target bit rate in bits per second
    /// * `min_steps` - Minimum number of steps
    pub fn new(target_bit_rate: f64, min_steps: i32) -> Self {
        Self {
            last_frame_time: None,
            average_frame_time: 0.016, // Default to 60 FPS
            target_bit_rate,
            min_steps,
        }
    }

    /// Reset the frame rate manager
    ///
    /// # Arguments
    ///
    /// * `time` - Current time
    pub fn reset(&mut self, time: Instant) {
        self.last_frame_time = Some(time);
        self.average_frame_time = 0.016; // Default to 60 FPS
    }

    /// Record a frame and update the average frame time
    ///
    /// # Arguments
    ///
    /// * `time` - Current time
    pub fn record_frame(&mut self, time: Instant) {
        if let Some(last_time) = self.last_frame_time {
            let frame_time = time.duration_since(last_time).as_secs_f64();

            // Update the average frame time with a simple exponential moving average
            // This gives more weight to recent frames while still smoothing out variations
            const ALPHA: f64 = 0.1; // Smoothing factor
            self.average_frame_time = (1.0 - ALPHA) * self.average_frame_time + ALPHA * frame_time;
        }

        self.last_frame_time = Some(time);
    }

    /// Calculate the number of steps needed to maintain the target bit rate
    ///
    /// # Returns
    ///
    /// The number of steps
    pub fn steps(&self) -> i32 {
        // Calculate the number of steps based on the target bit rate and average frame time
        // steps = bit_rate * frame_time
        let steps = (self.target_bit_rate * self.average_frame_time).round() as i32;

        // Ensure we have at least the minimum number of steps
        steps.max(self.min_steps)
    }

    /// Get the average frame time
    ///
    /// # Returns
    ///
    /// The average frame time in seconds
    pub fn average_frame_time(&self) -> f64 {
        self.average_frame_time
    }

    /// Set the target bit rate
    ///
    /// # Arguments
    ///
    /// * `bit_rate` - Target bit rate in bits per second
    pub fn set_target_bit_rate(&mut self, bit_rate: f64) {
        self.target_bit_rate = bit_rate;
    }

    /// Get the target bit rate
    ///
    /// # Returns
    ///
    /// The target bit rate in bits per second
    pub fn target_bit_rate(&self) -> f64 {
        self.target_bit_rate
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_frame_rate_basic() {
        let mut frame_rate = FrameRate::new(10.0, 1);

        // Test initial state
        assert_eq!(frame_rate.steps(), 1);

        // Test reset
        let now = Instant::now();
        frame_rate.reset(now);
        assert_eq!(frame_rate.last_frame_time, Some(now));

        // Test record_frame
        thread::sleep(Duration::from_millis(16));
        let now = Instant::now();
        frame_rate.record_frame(now);

        // Test steps calculation
        assert!(frame_rate.steps() >= 1);
    }

    #[test]
    fn test_frame_rate_steps() {
        let mut frame_rate = FrameRate::new(10.0, 1);

        // Test with different frame times
        frame_rate.average_frame_time = 0.1; // 100ms per frame
        assert_eq!(frame_rate.steps(), 1); // 10 * 0.1 = 1

        frame_rate.average_frame_time = 0.5; // 500ms per frame
        assert_eq!(frame_rate.steps(), 5); // 10 * 0.5 = 5

        // Test with different bit rates
        frame_rate.set_target_bit_rate(20.0);
        assert_eq!(frame_rate.steps(), 10); // 20 * 0.5 = 10

        // Test with minimum steps
        let mut frame_rate = FrameRate::new(10.0, 5);
        frame_rate.average_frame_time = 0.1; // 100ms per frame
        assert_eq!(frame_rate.steps(), 5); // 10 * 0.1 = 1, but min is 5
    }
}
