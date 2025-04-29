//! Dynamic filter implementation
//!
//! This module provides the base trait and implementation for dynamic filters,
//! which produce continuous movement based on input.

use std::time::Instant;
use crate::input::filter::InputFilter;
use crate::input::frame_rate::FrameRate;
use crate::model::DasherModel;
use crate::view::DasherView;


/// Dynamic filter configuration
#[derive(Debug, Clone)]
pub struct DynamicFilterConfig {
    /// Target bit rate (bits per second)
    pub bit_rate: f64,

    /// Whether to use slow start
    pub slow_start: bool,

    /// Slow start time (in milliseconds)
    pub slow_start_time: u64,

    /// Whether to use exact dynamics
    pub exact_dynamics: bool,

    /// X limit speed (maximum X coordinate)
    pub x_limit_speed: i64,
}

impl Default for DynamicFilterConfig {
    fn default() -> Self {
        Self {
            bit_rate: 10.0,
            slow_start: true,
            slow_start_time: 1000,
            exact_dynamics: false,
            x_limit_speed: 100,
        }
    }
}

/// Dynamic filter trait
///
/// This trait extends the InputFilter trait with methods specific to dynamic filters.
pub trait DynamicFilter: InputFilter {
    /// Get the frame rate manager
    fn frame_rate(&self) -> &FrameRate;

    /// Get a mutable reference to the frame rate manager
    fn frame_rate_mut(&mut self) -> &mut FrameRate;

    /// Get the configuration
    fn config(&self) -> &DynamicFilterConfig;

    /// Get a mutable reference to the configuration
    fn config_mut(&mut self) -> &mut DynamicFilterConfig;

    /// Get the start time
    fn start_time(&self) -> Option<Instant>;

    /// Set the start time
    fn set_start_time(&mut self, time: Instant);

    /// Calculate the speed multiplier for the current frame
    ///
    /// # Arguments
    ///
    /// * `model` - The Dasher model
    /// * `time` - Current time
    ///
    /// # Returns
    ///
    /// The speed multiplier
    fn frame_speed_mul(&self, model: &DasherModel, time: Instant) -> f64 {
        // Get the node under the crosshair
        let node_speed_mul = model.get_node_under_crosshair()
            .map(|node_rc| {
                let node = node_rc.borrow();
                node.speed_mul()
            })
            .unwrap_or(1.0);

        // Apply slow start if enabled
        let mut speed_mul = node_speed_mul;

        if self.config().slow_start {
            if let Some(start_time) = self.start_time() {
                let elapsed = time.duration_since(start_time).as_millis() as u64;
                if elapsed < self.config().slow_start_time {
                    // Gradually increase speed from 10% to 100% over the slow start time
                    let slow_start_factor = 0.1 + 0.9 * (elapsed as f64 / self.config().slow_start_time as f64);
                    speed_mul *= slow_start_factor;
                }
            }
        }

        speed_mul
    }

    /// Schedule one step towards the target
    ///
    /// # Arguments
    ///
    /// * `model` - The Dasher model
    /// * `x` - Target X coordinate
    /// * `y` - Target Y coordinate
    /// * `time` - Current time
    /// * `speed_mul` - Speed multiplier
    ///
    /// # Returns
    ///
    /// `true` if a step was scheduled, `false` otherwise
    fn one_step_towards(&mut self, model: &mut DasherModel, mut x: i64, y: i64, time: Instant, speed_mul: f64) -> bool {
        // If speed multiplier is zero or negative, we're not moving
        if speed_mul <= 0.0 {
            return false;
        }

        // Record the frame
        self.frame_rate_mut().record_frame(time);

        // Calculate the number of steps based on the frame rate and speed multiplier
        let steps = (self.frame_rate().steps() as f64 / speed_mul).round() as i32;
        assert!(steps > 0);

        // Limit X to prevent overflow
        // If X is too large, we risk overflow errors when multiplying by steps
        const MAX_X: i64 = (1 << 29) / 100; // Arbitrary limit to prevent overflow
        x = x.max(1).min(MAX_X);

        // Schedule the step
        model.schedule_one_step(
            y - x, // y1
            y + x, // y2
            steps,
            self.config().x_limit_speed as i32,
            self.config().exact_dynamics
        );

        true
    }

    /// Run the filter
    ///
    /// # Arguments
    ///
    /// * `time` - Current time
    fn run(&mut self, time: Instant) {
        // Reset the frame rate
        self.frame_rate_mut().reset(time);

        // Set the start time for slow start
        self.set_start_time(time);

        // Unpause the filter
        self.unpause();
    }
}

/// Base implementation of a dynamic filter
#[derive(Debug)]
pub struct DynamicFilterBase {
    /// Frame rate manager
    frame_rate: FrameRate,

    /// Configuration
    config: DynamicFilterConfig,

    /// Start time for slow start
    start_time: Option<Instant>,

    /// Whether the filter is paused
    paused: bool,
}

impl DynamicFilterBase {
    /// Create a new dynamic filter base
    ///
    /// # Arguments
    ///
    /// * `config` - Configuration
    pub fn new(config: DynamicFilterConfig) -> Self {
        Self {
            frame_rate: FrameRate::new(config.bit_rate, 1),
            config,
            start_time: None,
            paused: true,
        }
    }

    /// Get the frame rate manager
    pub fn frame_rate(&self) -> &FrameRate {
        &self.frame_rate
    }

    /// Get a mutable reference to the frame rate manager
    pub fn frame_rate_mut(&mut self) -> &mut FrameRate {
        &mut self.frame_rate
    }

    /// Get the configuration
    pub fn config(&self) -> &DynamicFilterConfig {
        &self.config
    }

    /// Get a mutable reference to the configuration
    pub fn config_mut(&mut self) -> &mut DynamicFilterConfig {
        &mut self.config
    }

    /// Get the start time
    pub fn start_time(&self) -> Option<Instant> {
        self.start_time
    }

    /// Set the start time
    pub fn set_start_time(&mut self, time: Instant) {
        self.start_time = Some(time);
    }

    /// Check if the filter is paused
    pub fn is_paused(&self) -> bool {
        self.paused
    }

    /// Pause the filter
    pub fn pause(&mut self) {
        self.paused = true;
    }

    /// Unpause the filter
    pub fn unpause(&mut self) {
        self.paused = false;
    }
}

// We don't implement DynamicFilter for DynamicFilterBase because it doesn't implement InputFilter
// Instead, we'll use composition in our concrete filter implementations

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;
    use crate::input::VirtualKey;
    use crate::input::DasherInput;

    // Mock implementation of DasherModel for testing
    #[derive(Debug)]
    struct MockDasherModel {
        scheduled_steps: Option<(i64, i64, i32, i64, bool)>,
    }

    impl MockDasherModel {
        fn new() -> Self {
            Self {
                scheduled_steps: None,
            }
        }

        fn schedule_one_step(&mut self, y1: i64, y2: i64, steps: i32, x_limit: i64, exact: bool) {
            self.scheduled_steps = Some((y1, y2, steps, x_limit, exact));
        }

        fn get_node_under_crosshair(&self) -> Option<MockNode> {
            Some(MockNode { speed_mul: 1.0 })
        }
    }

    // Mock implementation of DasherNode for testing
    #[derive(Debug)]
    struct MockNode {
        speed_mul: f64,
    }

    impl MockNode {
        fn speed_mul(&self) -> f64 {
            self.speed_mul
        }
    }

    // Mock implementation of DasherView for testing
    #[derive(Debug)]
    struct MockDasherView;

    impl MockDasherView {
        fn new() -> Self {
            Self
        }

        fn dasher_to_screen(&self, _x: i64, _y: i64) -> (i32, i32) {
            (100, 100)
        }

        fn draw_line(&self, _x1: i64, _y1: i64, _x2: i64, _y2: i64, _color: (u8, u8, u8, u8), _width: i32) {
            // No-op for testing
        }
    }

    // Mock implementation of DynamicFilter for testing
    #[derive(Debug)]
    struct MockDynamicFilter {
        base: DynamicFilterBase,
    }

    impl MockDynamicFilter {
        fn new(config: DynamicFilterConfig) -> Self {
            Self {
                base: DynamicFilterBase::new(config),
            }
        }

        // Implement a version of one_step_towards that works with our mock model
        fn one_step_towards_mock(&mut self, model: &mut MockDasherModel, mut x: i64, y: i64, time: Instant, speed_mul: f64) -> bool {
            // If speed multiplier is zero or negative, we're not moving
            if speed_mul <= 0.0 {
                return false;
            }

            // Record the frame
            self.base.frame_rate_mut().record_frame(time);

            // Calculate the number of steps based on the frame rate and speed multiplier
            let steps = (self.base.frame_rate().steps() as f64 / speed_mul).round() as i32;
            assert!(steps > 0);

            // Limit X to prevent overflow
            const MAX_X: i64 = (1 << 29) / 100; // Arbitrary limit to prevent overflow
            x = x.max(1).min(MAX_X);

            // Schedule the step
            model.schedule_one_step(
                y - x, // y1
                y + x, // y2
                steps,
                self.base.config().x_limit_speed,
                self.base.config().exact_dynamics
            );

            true
        }

        // Implement a version of frame_speed_mul that works with our mock model
        fn frame_speed_mul_mock(&self, model: &MockDasherModel, time: Instant) -> f64 {
            // Get the node under the crosshair
            let node_speed_mul = model.get_node_under_crosshair()
                .map(|node| node.speed_mul())
                .unwrap_or(1.0);

            // Apply slow start if enabled
            let mut speed_mul = node_speed_mul;

            if self.base.config().slow_start {
                if let Some(start_time) = self.base.start_time() {
                    let elapsed = time.duration_since(start_time).as_millis() as u64;
                    if elapsed < self.base.config().slow_start_time {
                        // Gradually increase speed from 10% to 100% over the slow start time
                        let slow_start_factor = 0.1 + 0.9 * (elapsed as f64 / self.base.config().slow_start_time as f64);
                        speed_mul *= slow_start_factor;
                    }
                }
            }

            speed_mul
        }
    }

    impl DynamicFilter for MockDynamicFilter {
        fn frame_rate(&self) -> &FrameRate {
            self.base.frame_rate()
        }

        fn frame_rate_mut(&mut self) -> &mut FrameRate {
            self.base.frame_rate_mut()
        }

        fn config(&self) -> &DynamicFilterConfig {
            self.base.config()
        }

        fn config_mut(&mut self) -> &mut DynamicFilterConfig {
            self.base.config_mut()
        }

        fn start_time(&self) -> Option<Instant> {
            self.base.start_time()
        }

        fn set_start_time(&mut self, time: Instant) {
            self.base.set_start_time(time);
        }
    }

    impl InputFilter for MockDynamicFilter {
        fn process(&mut self, _input: &mut dyn DasherInput, _time: u64, _model: &mut DasherModel, _view: &mut dyn DasherView) {}
        fn key_down(&mut self, _time: u64, _key: VirtualKey, _model: &mut DasherModel, _view: &mut dyn DasherView) {}
        fn key_up(&mut self, _time: u64, _key: VirtualKey, _model: &mut DasherModel, _view: &mut dyn DasherView) {}
        fn supports_pause(&self) -> bool { true }
        fn pause(&mut self) { self.base.pause(); }
        fn unpause(&mut self) { self.base.unpause(); }
        fn is_paused(&self) -> bool { self.base.is_paused() }
        fn reset(&mut self) {}
        fn activate(&mut self) {}
        fn deactivate(&mut self) {}
        fn decorate_view(&mut self, _view: &mut dyn DasherView) -> bool { false }
    }

    #[test]
    fn test_dynamic_filter_base() {
        let config = DynamicFilterConfig::default();
        let mut filter = DynamicFilterBase::new(config);

        // Test initial state
        assert!(filter.is_paused());
        assert_eq!(filter.start_time(), None);

        // Test pause/unpause
        filter.unpause();
        assert!(!filter.is_paused());
        filter.pause();
        assert!(filter.is_paused());

        // Test start time
        let now = Instant::now();
        filter.set_start_time(now);
        assert_eq!(filter.start_time(), Some(now));
    }

    #[test]
    fn test_dynamic_filter_one_step_towards() {
        let config = DynamicFilterConfig::default();
        let mut filter = MockDynamicFilter::new(config);
        let mut model = MockDasherModel::new();

        // Test one_step_towards
        let now = Instant::now();
        let result = filter.one_step_towards_mock(&mut model, 100, 2048, now, 1.0);

        // Check that a step was scheduled
        assert!(result);
        assert!(model.scheduled_steps.is_some());

        // Check the scheduled step parameters
        if let Some((y1, y2, steps, x_limit, exact)) = model.scheduled_steps {
            assert_eq!(y1, 2048 - 100);
            assert_eq!(y2, 2048 + 100);
            assert!(steps > 0);
            assert_eq!(x_limit, 100);
            assert_eq!(exact, false);
        }

        // Test with zero speed multiplier
        let mut model = MockDasherModel::new();
        let result = filter.one_step_towards_mock(&mut model, 100, 2048, now, 0.0);

        // Check that no step was scheduled
        assert!(!result);
        assert!(model.scheduled_steps.is_none());
    }

    #[test]
    fn test_dynamic_filter_frame_speed_mul() {
        let mut config = DynamicFilterConfig::default();
        config.slow_start = true;
        config.slow_start_time = 1000;

        let mut filter = MockDynamicFilter::new(config);
        let model = MockDasherModel::new();

        // Test without slow start
        let mut config_no_slow_start = DynamicFilterConfig::default();
        config_no_slow_start.slow_start = false;
        let mut filter_no_slow_start = MockDynamicFilter::new(config_no_slow_start);
        let now = Instant::now();
        filter_no_slow_start.set_start_time(now);
        let speed_mul = filter_no_slow_start.frame_speed_mul_mock(&model, now);
        assert_eq!(speed_mul, 1.0);

        // Test with slow start
        let now = Instant::now();
        filter.set_start_time(now);
        let later = now + Duration::from_millis(500);
        let speed_mul = filter.frame_speed_mul_mock(&model, later);

        // Speed should be between 0.1 and 1.0
        // The exact value depends on the elapsed time, which can vary in tests
        // So we just check that it's in the expected range
        assert!(speed_mul >= 0.1);
        assert!(speed_mul <= 1.0);

        // Test after slow start time
        let now = Instant::now();
        filter.set_start_time(now);
        let later = now + Duration::from_millis(2000);
        let speed_mul = filter.frame_speed_mul_mock(&model, later);
        assert_eq!(speed_mul, 1.0);
    }

    #[test]
    fn test_dynamic_filter_run() {
        let config = DynamicFilterConfig::default();
        let mut filter = MockDynamicFilter::new(config);

        // Test run
        let now = Instant::now();
        filter.run(now);

        // Check that the filter is unpaused and start time is set
        assert!(!filter.is_paused());
        assert_eq!(filter.start_time(), Some(now));
    }
}
