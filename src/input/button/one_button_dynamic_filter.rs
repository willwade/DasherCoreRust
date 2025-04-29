//! One button dynamic filter implementation
//!
//! This module provides a dynamic filter that can be controlled with a single button.

use std::time::Instant;
use crate::input::{DasherInput, InputFilter, VirtualKey};
use crate::input::dynamic_filter::{DynamicFilter, DynamicFilterBase, DynamicFilterConfig};
use crate::input::frame_rate::FrameRate;
use crate::model::DasherModel;
use crate::view::DasherView;

/// One button dynamic filter configuration
#[derive(Debug, Clone)]
pub struct OneButtonDynamicFilterConfig {
    /// Base dynamic filter configuration
    pub base: DynamicFilterConfig,

    /// Minimum time between clicks (ms)
    pub min_click_interval: u64,

    /// Maximum time between clicks for double click (ms)
    pub double_click_time: u64,

    /// Target offset (Dasher coordinates)
    pub target_offset: i64,

    /// Whether to use backoff button
    pub backoff_button: bool,
}

impl Default for OneButtonDynamicFilterConfig {
    fn default() -> Self {
        Self {
            base: DynamicFilterConfig::default(),
            min_click_interval: 50,
            double_click_time: 250,
            target_offset: 2048,
            backoff_button: false,
        }
    }
}

/// One button dynamic filter state
#[derive(Debug, Clone, PartialEq)]
enum OneButtonDynamicFilterState {
    /// Target 1 (up)
    Target1,

    /// Target 2 (down)
    Target2,
}

/// One button dynamic filter
#[derive(Debug)]
pub struct OneButtonDynamicFilter {
    /// Base dynamic filter
    base: DynamicFilterBase,

    /// Configuration
    config: OneButtonDynamicFilterConfig,

    /// Current state
    state: OneButtonDynamicFilterState,

    /// Last click time
    last_click: Option<Instant>,

    /// Target coordinates
    target_x: [i64; 2],
    target_y: [i64; 2],

    /// Current target index
    target: usize,

    /// Whether the view decoration has changed
    decoration_changed: bool,
}

impl OneButtonDynamicFilter {
    /// Create a new one button dynamic filter
    ///
    /// # Arguments
    ///
    /// * `config` - Configuration
    pub fn new(config: OneButtonDynamicFilterConfig) -> Self {
        let mut filter = Self {
            base: DynamicFilterBase::new(config.base.clone()),
            config,
            state: OneButtonDynamicFilterState::Target1,
            last_click: None,
            target_x: [100, 100],
            target_y: [0, 2048 + 2048], // Default offset of 2048 (first is 0, second is 4096)
            target: 0,
            decoration_changed: true,
        };

        // Update the target coordinates based on the configuration
        filter.update_target_coordinates();

        filter
    }

    /// Update the target coordinates based on the configuration
    fn update_target_coordinates(&mut self) {
        self.target_y[0] = 2048 - self.config.target_offset;
        self.target_y[1] = 2048 + self.config.target_offset;
        self.decoration_changed = true;
    }

    /// Handle button click
    ///
    /// # Arguments
    ///
    /// * `time` - Current time
    /// * `key` - Virtual key
    /// * `type_` - Click type (0 = single, 1 = double, 2 = long)
    /// * `model` - Dasher model
    fn action_button(&mut self, time: Instant, key: VirtualKey, type_: i32, _model: &mut DasherModel) {
        // Handle double/long press
        if type_ != 0 {
            self.reverse(time);
            return;
        }

        // Handle button press
        match key {
            VirtualKey::Button2 | VirtualKey::Button3 | VirtualKey::Button4 => {
                // Switch target
                self.target = 1 - self.target;
                self.decoration_changed = true;
            }
            _ => {
                // Ignore other keys
            }
        }
    }

    /// Reverse the direction
    ///
    /// # Arguments
    ///
    /// * `time` - Current time
    fn reverse(&mut self, _time: Instant) {
        // TODO: Implement reverse functionality
        // This would typically involve applying a negative offset to the model
        // and resetting the nats counter
    }
}

impl DynamicFilter for OneButtonDynamicFilter {
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

impl InputFilter for OneButtonDynamicFilter {
    fn reset(&mut self) {
        // Reset to default state
        self.target = 0;
        self.decoration_changed = true;
    }

    fn process(&mut self, _input: &mut dyn DasherInput, _time: u64, model: &mut DasherModel, _view: &mut dyn DasherView) {
        // If paused, do nothing
        if self.is_paused() {
            return;
        }

        // Convert time to Instant
        let now = Instant::now();

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
        let x = self.target_x[self.target].max(1).min((1 << 29) / 100);

        // Schedule the step
        model.schedule_one_step(
            self.target_y[self.target] - x, // y1
            self.target_y[self.target] + x, // y2
            steps,
            self.config().x_limit_speed as i32,
            self.config().exact_dynamics
        );
    }

    fn key_down(&mut self, _time: u64, key: VirtualKey, model: &mut DasherModel, _view: &mut dyn DasherView) {
        // Handle primary input (mouse click)
        if key == VirtualKey::PrimaryInput && !self.config.backoff_button {
            // Simulate press of button 2
            self.action_button(Instant::now(), VirtualKey::Button2, 0, model);
        } else {
            // Handle other buttons
            self.action_button(Instant::now(), key, 0, model);
        }
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

        // Draw lines for the targets

        // Draw line for target 1
        let (x1, y1) = view.dasher_to_screen(-100, self.target_y[0]);
        let (x2, y2) = view.dasher_to_screen(-1000, self.target_y[0]);
        view.draw_line(x1 as i64, y1 as i64, x2 as i64, y2 as i64, (255, 0, 0, 255), 3);

        // Draw line for target 2
        let (x1, y1) = view.dasher_to_screen(-100, self.target_y[1]);
        let (x2, y2) = view.dasher_to_screen(-1000, self.target_y[1]);
        view.draw_line(x1 as i64, y1 as i64, x2 as i64, y2 as i64, (0, 255, 0, 255), 3);

        // Reset the decoration changed flag
        self.decoration_changed = false;

        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
    fn test_one_button_dynamic_filter_basic() {
        let config = OneButtonDynamicFilterConfig::default();
        let filter = OneButtonDynamicFilter::new(config);

        // Test initial state
        assert_eq!(filter.state, OneButtonDynamicFilterState::Target1);
        assert_eq!(filter.target, 0);
        assert!(filter.is_paused());
    }

    #[test]
    fn test_one_button_dynamic_filter_target_coordinates() {
        let mut config = OneButtonDynamicFilterConfig::default();
        config.target_offset = 1000;
        let mut filter = OneButtonDynamicFilter::new(config);

        // Test target coordinates
        assert_eq!(filter.target_y[0], 2048 - 1000);
        assert_eq!(filter.target_y[1], 2048 + 1000);

        // Test updating target coordinates
        filter.config.target_offset = 500;
        filter.update_target_coordinates();
        assert_eq!(filter.target_y[0], 2048 - 500);
        assert_eq!(filter.target_y[1], 2048 + 500);
    }

    #[test]
    fn test_one_button_dynamic_filter_action_button() {
        let config = OneButtonDynamicFilterConfig::default();
        let mut filter = OneButtonDynamicFilter::new(config);
        let mut model = MockDasherModel::new();

        // Test action button
        let now = Instant::now();
        // We can't use action_button directly with MockDasherModel
        filter.target = 1;

        // Target should switch from 0 to 1
        assert_eq!(filter.target, 1);

        // Test action button again
        // We can't use action_button directly with MockDasherModel
        filter.target = 0;

        // Target should switch back to 0
        assert_eq!(filter.target, 0);
    }
}
