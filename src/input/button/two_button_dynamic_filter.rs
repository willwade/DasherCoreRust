//! Two button dynamic filter implementation
//!
//! This module provides a dynamic filter that can be controlled with two buttons.

use std::time::Instant;
use crate::input::{DasherInput, InputFilter, VirtualKey};
use crate::input::dynamic_filter::{DynamicFilter, DynamicFilterBase, DynamicFilterConfig};
use crate::input::filter::DasherInputExt;
use crate::input::frame_rate::FrameRate;
use crate::model::DasherModel;
use crate::view::DasherView;

/// Two button dynamic filter configuration
#[derive(Debug, Clone)]
pub struct TwoButtonDynamicFilterConfig {
    /// Base dynamic filter configuration
    pub base: DynamicFilterConfig,

    /// Target offset (Dasher coordinates)
    pub target_offset: i64,

    /// Whether to use backoff button
    pub backoff_button: bool,
}

impl Default for TwoButtonDynamicFilterConfig {
    fn default() -> Self {
        Self {
            base: DynamicFilterConfig::default(),
            target_offset: 2048,
            backoff_button: false,
        }
    }
}

/// Two button dynamic filter state
#[derive(Debug, Clone, PartialEq)]
enum TwoButtonDynamicFilterState {
    /// Idle
    Idle,

    /// Moving up
    Up,

    /// Moving down
    Down,
}

/// Two button dynamic filter
#[derive(Debug)]
pub struct TwoButtonDynamicFilter {
    /// Base dynamic filter
    base: DynamicFilterBase,

    /// Configuration
    config: TwoButtonDynamicFilterConfig,

    /// Current state
    state: TwoButtonDynamicFilterState,

    /// Target coordinates
    target_x: i64,
    target_y: [i64; 3], // Idle, Up, Down

    /// Whether the view decoration has changed
    decoration_changed: bool,

    /// Whether button 1 is pressed
    button1_pressed: bool,

    /// Whether button 2 is pressed
    button2_pressed: bool,
}

impl TwoButtonDynamicFilter {
    /// Create a new two button dynamic filter
    ///
    /// # Arguments
    ///
    /// * `config` - Configuration
    pub fn new(config: TwoButtonDynamicFilterConfig) -> Self {
        let mut filter = Self {
            base: DynamicFilterBase::new(config.base.clone()),
            config,
            state: TwoButtonDynamicFilterState::Idle,
            target_x: 100,
            target_y: [2048, 0, 2048 + 2048], // Default offset of 2048 (center, top, bottom)
            decoration_changed: true,
            button1_pressed: false,
            button2_pressed: false,
        };

        // Update the target coordinates based on the configuration
        filter.update_target_coordinates();

        filter
    }

    /// Update the target coordinates based on the configuration
    fn update_target_coordinates(&mut self) {
        self.target_y[1] = 2048 - self.config.target_offset;
        self.target_y[2] = 2048 + self.config.target_offset;
        self.decoration_changed = true;
    }

    /// Update the state based on button presses
    fn update_state(&mut self) {
        if self.button1_pressed && !self.button2_pressed {
            self.state = TwoButtonDynamicFilterState::Up;
        } else if !self.button1_pressed && self.button2_pressed {
            self.state = TwoButtonDynamicFilterState::Down;
        } else {
            self.state = TwoButtonDynamicFilterState::Idle;
        }
    }

    /// Get the current target Y coordinate
    fn current_target_y(&self) -> i64 {
        match self.state {
            TwoButtonDynamicFilterState::Idle => self.target_y[0],
            TwoButtonDynamicFilterState::Up => self.target_y[1],
            TwoButtonDynamicFilterState::Down => self.target_y[2],
        }
    }
}

impl DynamicFilter for TwoButtonDynamicFilter {
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

impl InputFilter for TwoButtonDynamicFilter {
    fn reset(&mut self) {
        // Reset to default state
        self.state = TwoButtonDynamicFilterState::Idle;
        self.button1_pressed = false;
        self.button2_pressed = false;
        self.decoration_changed = true;
    }

    fn process(&mut self, input: &mut dyn DasherInput, _time: u64, model: &mut DasherModel, _view: &mut dyn DasherView) {
        // If paused, do nothing
        if self.is_paused() {
            return;
        }

        // Convert time to Instant
        let now = Instant::now();

        // Update button states
        self.button1_pressed = input.is_button_pressed(1);
        self.button2_pressed = input.is_button_pressed(2);

        // Update state
        self.update_state();

        // If idle, do nothing
        if self.state == TwoButtonDynamicFilterState::Idle {
            return;
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
            self.current_target_y() - x, // y1
            self.current_target_y() + x, // y2
            steps,
            self.config().x_limit_speed as i32,
            self.config().exact_dynamics
        );
    }

    fn key_down(&mut self, _time: u64, key: VirtualKey, _model: &mut DasherModel, _view: &mut dyn DasherView) {
        // Handle key down
        match key {
            VirtualKey::Button1 => {
                self.button1_pressed = true;
                self.update_state();
            }
            VirtualKey::Button2 => {
                self.button2_pressed = true;
                self.update_state();
            }
            _ => {
                // Ignore other keys
            }
        }
    }

    fn key_up(&mut self, _time: u64, key: VirtualKey, _model: &mut DasherModel, _view: &mut dyn DasherView) {
        // Handle key up
        match key {
            VirtualKey::Button1 => {
                self.button1_pressed = false;
                self.update_state();
            }
            VirtualKey::Button2 => {
                self.button2_pressed = false;
                self.update_state();
            }
            _ => {
                // Ignore other keys
            }
        }
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

        // Draw line for up target
        let (x1, y1) = view.dasher_to_screen(-100, self.target_y[1]);
        let (x2, y2) = view.dasher_to_screen(-1000, self.target_y[1]);
        view.draw_line(x1 as i64, y1 as i64, x2 as i64, y2 as i64, (255, 0, 0, 255), 3);

        // Draw line for down target
        let (x1, y1) = view.dasher_to_screen(-100, self.target_y[2]);
        let (x2, y2) = view.dasher_to_screen(-1000, self.target_y[2]);
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
    fn test_two_button_dynamic_filter_basic() {
        let config = TwoButtonDynamicFilterConfig::default();
        let filter = TwoButtonDynamicFilter::new(config);

        // Test initial state
        assert_eq!(filter.state, TwoButtonDynamicFilterState::Idle);
        assert!(!filter.button1_pressed);
        assert!(!filter.button2_pressed);
        assert!(filter.is_paused());
    }

    #[test]
    fn test_two_button_dynamic_filter_target_coordinates() {
        let mut config = TwoButtonDynamicFilterConfig::default();
        config.target_offset = 1000;
        let mut filter = TwoButtonDynamicFilter::new(config);

        // Test target coordinates
        assert_eq!(filter.target_y[1], 2048 - 1000);
        assert_eq!(filter.target_y[2], 2048 + 1000);

        // Test updating target coordinates
        filter.config.target_offset = 500;
        filter.update_target_coordinates();
        assert_eq!(filter.target_y[1], 2048 - 500);
        assert_eq!(filter.target_y[2], 2048 + 500);
    }

    #[test]
    fn test_two_button_dynamic_filter_update_state() {
        let config = TwoButtonDynamicFilterConfig::default();
        let mut filter = TwoButtonDynamicFilter::new(config);

        // Test idle state
        filter.button1_pressed = false;
        filter.button2_pressed = false;
        filter.update_state();
        assert_eq!(filter.state, TwoButtonDynamicFilterState::Idle);

        // Test up state
        filter.button1_pressed = true;
        filter.button2_pressed = false;
        filter.update_state();
        assert_eq!(filter.state, TwoButtonDynamicFilterState::Up);

        // Test down state
        filter.button1_pressed = false;
        filter.button2_pressed = true;
        filter.update_state();
        assert_eq!(filter.state, TwoButtonDynamicFilterState::Down);

        // Test both buttons pressed (should be idle)
        filter.button1_pressed = true;
        filter.button2_pressed = true;
        filter.update_state();
        assert_eq!(filter.state, TwoButtonDynamicFilterState::Idle);
    }

    #[test]
    fn test_two_button_dynamic_filter_current_target_y() {
        let config = TwoButtonDynamicFilterConfig::default();
        let mut filter = TwoButtonDynamicFilter::new(config);

        // Test idle target
        filter.state = TwoButtonDynamicFilterState::Idle;
        assert_eq!(filter.current_target_y(), filter.target_y[0]);

        // Test up target
        filter.state = TwoButtonDynamicFilterState::Up;
        assert_eq!(filter.current_target_y(), filter.target_y[1]);

        // Test down target
        filter.state = TwoButtonDynamicFilterState::Down;
        assert_eq!(filter.current_target_y(), filter.target_y[2]);
    }
}
