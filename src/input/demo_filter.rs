//! Demo dynamic filter implementation
//!
//! This module provides a dynamic filter that automatically demonstrates Dasher.

use std::time::Instant;
use crate::input::{DasherInput, InputFilter, VirtualKey};
use crate::input::dynamic_filter::{DynamicFilter, DynamicFilterBase, DynamicFilterConfig};
use crate::input::frame_rate::FrameRate;
use crate::model::DasherModel;
use crate::view::DasherView;

/// Demo dynamic filter configuration
#[derive(Debug, Clone)]
pub struct DemoDynamicFilterConfig {
    /// Base dynamic filter configuration
    pub base: DynamicFilterConfig,

    /// Target offset (Dasher coordinates)
    pub target_offset: i64,

    /// Time between target changes (milliseconds)
    pub target_change_interval: u64,

    /// Whether to use random targets
    pub random_targets: bool,
}

impl Default for DemoDynamicFilterConfig {
    fn default() -> Self {
        Self {
            base: DynamicFilterConfig::default(),
            target_offset: 2048,
            target_change_interval: 5000,
            random_targets: true,
        }
    }
}

/// Demo dynamic filter
#[derive(Debug)]
pub struct DemoDynamicFilter {
    /// Base dynamic filter
    base: DynamicFilterBase,

    /// Configuration
    config: DemoDynamicFilterConfig,

    /// Target coordinates
    target_x: i64,
    target_y: i64,

    /// Last target change time
    last_target_change: Option<Instant>,

    /// Whether the view decoration has changed
    decoration_changed: bool,
}

impl DemoDynamicFilter {
    /// Create a new demo dynamic filter
    ///
    /// # Arguments
    ///
    /// * `config` - Configuration
    pub fn new(config: DemoDynamicFilterConfig) -> Self {
        Self {
            base: DynamicFilterBase::new(config.base.clone()),
            config,
            target_x: 100,
            target_y: 2048,
            last_target_change: None,
            decoration_changed: true,
        }
    }

    /// Change the target
    ///
    /// # Arguments
    ///
    /// * `time` - Current time
    fn change_target(&mut self, time: Instant) {
        // Update the last target change time
        self.last_target_change = Some(time);

        // Generate a new target
        if self.config.random_targets {
            // Random target within the offset range
            let offset = rand::random::<f64>() * self.config.target_offset as f64 * 2.0 - self.config.target_offset as f64;
            self.target_y = 2048 + offset as i64;
        } else {
            // Alternate between up and down
            if self.target_y == 2048 + self.config.target_offset {
                self.target_y = 2048 - self.config.target_offset;
            } else {
                self.target_y = 2048 + self.config.target_offset;
            }
        }

        // Mark the decoration as changed
        self.decoration_changed = true;
    }
}

impl DynamicFilter for DemoDynamicFilter {
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

impl InputFilter for DemoDynamicFilter {
    fn reset(&mut self) {
        // Reset to default state
        self.target_y = 2048;
        self.last_target_change = None;
        self.decoration_changed = true;
    }

    fn process(&mut self, _input: &mut dyn DasherInput, _time: u64, model: &mut DasherModel, _view: &mut dyn DasherView) {
        // If paused, do nothing
        if self.is_paused() {
            return;
        }

        // Convert time to Instant
        let now = Instant::now();

        // Check if it's time to change the target
        if let Some(last_change) = self.last_target_change {
            let elapsed = now.duration_since(last_change).as_millis() as u64;
            if elapsed >= self.config.target_change_interval {
                self.change_target(now);
            }
        } else {
            // First run, set the initial target
            self.change_target(now);
        }

        // Move towards the current target
        let speed_mul = self.frame_speed_mul(model, now);

        // If speed multiplier is zero or negative, we're not moving
        if speed_mul <= 0.0 {
            return;
        }

        // Record the frame
        self.frame_rate_mut().record_frame(now);

        // Calculate the number of steps based on the frame rate and speed multiplier
        let steps = (self.frame_rate().steps() as f64 / speed_mul).round() as i32;
        assert!(steps > 0);

        // Limit X to prevent overflow
        let x = self.target_x.max(1).min((1 << 29) / 100);

        // Schedule the step
        model.schedule_one_step(
            self.target_y - x, // y1
            self.target_y + x, // y2
            steps,
            self.config.base.x_limit_speed as i32,
            self.config.base.exact_dynamics
        );
    }

    fn key_down(&mut self, _time: u64, _key: VirtualKey, _model: &mut DasherModel, _view: &mut dyn DasherView) {
        // No action on key down
    }

    fn key_up(&mut self, _time: u64, _key: VirtualKey, _model: &mut DasherModel, _view: &mut dyn DasherView) {
        // No action on key up
    }

    fn supports_pause(&self) -> bool {
        true
    }

    fn pause(&mut self) {
        self.base.pause();
    }

    fn unpause(&mut self) {
        self.base.unpause();
    }

    fn is_paused(&self) -> bool {
        self.base.is_paused()
    }

    fn activate(&mut self) {
        self.reset();
    }

    fn deactivate(&mut self) {
        self.reset();
    }

    fn decorate_view(&mut self, view: &mut dyn DasherView) -> bool {
        // If no decoration change, return false
        if !self.decoration_changed {
            return false;
        }

        // Draw a line for the current target
        let (x1, y1) = view.dasher_to_screen(-100, self.target_y);
        let (x2, y2) = view.dasher_to_screen(-1000, self.target_y);
        view.draw_line(x1 as i64, y1 as i64, x2 as i64, y2 as i64, (255, 0, 0, 255), 3);

        // Reset the decoration changed flag
        self.decoration_changed = false;

        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    // Mock implementation of DasherModel for testing
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
    struct MockNode {
        speed_mul: f64,
    }

    impl MockNode {
        fn speed_mul(&self) -> f64 {
            self.speed_mul
        }
    }

    // Mock implementation of DasherView for testing
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

    #[test]
    fn test_demo_dynamic_filter_basic() {
        let config = DemoDynamicFilterConfig::default();
        let filter = DemoDynamicFilter::new(config);

        // Test initial state
        assert_eq!(filter.target_y, 2048);
        assert_eq!(filter.last_target_change, None);
        assert!(filter.is_paused());
    }

    #[test]
    fn test_demo_dynamic_filter_change_target() {
        // Test with random targets
        let mut config = DemoDynamicFilterConfig::default();
        config.random_targets = true;
        let mut filter = DemoDynamicFilter::new(config);

        // Change target
        let now = Instant::now();
        filter.change_target(now);

        // Check that the target changed
        assert_eq!(filter.last_target_change, Some(now));
        assert!(filter.target_y != 2048);

        // Test with alternating targets
        let mut config = DemoDynamicFilterConfig::default();
        config.random_targets = false;
        let target_offset = config.target_offset;
        let mut filter = DemoDynamicFilter::new(config);

        // Change target
        let now = Instant::now();
        filter.change_target(now);

        // Check that the target changed
        assert_eq!(filter.last_target_change, Some(now));
        assert!(filter.target_y == 2048 + target_offset || filter.target_y == 2048 - target_offset);

        // Change target again
        let later = now + Duration::from_millis(100);
        filter.change_target(later);

        // Check that the target changed to the opposite
        assert_eq!(filter.last_target_change, Some(later));
        if filter.target_y == 2048 + target_offset {
            assert_eq!(filter.target_y, 2048 + target_offset);
        } else {
            assert_eq!(filter.target_y, 2048 - target_offset);
        }
    }
}
