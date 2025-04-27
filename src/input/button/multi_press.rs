use std::time::{Duration, Instant};
use crate::{DasherInput, input::{InputFilter, InputDevice, Coordinates, VirtualKey}};
use crate::model::DasherModel;
use crate::input::filter::DasherInputExt;
use crate::view::DasherView;


/// Multi-press mode configuration
#[derive(Debug, Clone)]
pub struct MultiPressConfig {
    /// Maximum time between presses for combination (ms)
    pub max_combo_time: u64,
    /// Time to hold for long press (ms)
    pub long_press_time: u64,
    /// Base movement speed
    pub base_speed: f64,
    /// Speed multiplier for each additional press
    pub combo_multiplier: f64,
}

impl Default for MultiPressConfig {
    fn default() -> Self {
        Self {
            max_combo_time: 500,
            long_press_time: 750,
            base_speed: 1.0,
            combo_multiplier: 1.5,
        }
    }
}

/// Multi-press action type
#[derive(Debug, Clone, Copy, PartialEq)]
enum MultiPressAction {
    /// Single tap - move forward
    SingleTap,
    /// Double tap - move backward
    DoubleTap,
    /// Triple tap - pause/resume
    TripleTap,
    /// Long press - zoom in
    LongPress,
    /// None - no action
    None,
}

/// Multi-press mode state
#[derive(Debug)]
struct MultiPressState {
    /// Press count
    press_count: u32,
    /// First press time
    first_press: Instant,
    /// Last press time
    last_press: Instant,
    /// Current action
    current_action: MultiPressAction,
    /// Is button currently held
    is_held: bool,
}

impl MultiPressState {
    fn new() -> Self {
        Self {
            press_count: 0,
            first_press: Instant::now(),
            last_press: Instant::now(),
            current_action: MultiPressAction::None,
            is_held: false,
        }
    }
}

/// Multi-press mode handler
#[derive(Debug)]
pub struct MultiPressMode {
    /// Configuration
    config: MultiPressConfig,
    /// Current state
    state: MultiPressState,
    /// Current coordinates
    current_coords: Coordinates,
    /// Movement paused
    paused: bool,
}

impl MultiPressMode {
    /// Create a new multi-press mode handler
    pub fn new(config: MultiPressConfig) -> Self {
        Self {
            config,
            state: MultiPressState::new(),
            current_coords: Coordinates::default(),
            paused: false,
        }
    }

    /// Handle button press
    fn handle_press(&mut self, now: Instant) {
        if !self.state.is_held {
            let elapsed = now.duration_since(self.state.last_press);

            if elapsed.as_millis() > self.config.max_combo_time as u128 {
                // Start new combo
                self.state.press_count = 1;
                self.state.first_press = now;
            } else {
                // Continue combo
                self.state.press_count += 1;
            }

            self.state.last_press = now;
            self.state.is_held = true;

            // Determine action based on press count
            self.state.current_action = match self.state.press_count {
                1 => MultiPressAction::SingleTap,
                2 => MultiPressAction::DoubleTap,
                3 => MultiPressAction::TripleTap,
                _ => MultiPressAction::None,
            };
        }
    }

    /// Handle button release
    fn handle_release(&mut self, now: Instant) {
        if self.state.is_held {
            let hold_time = now.duration_since(self.state.last_press);

            if hold_time.as_millis() >= self.config.long_press_time as u128 {
                self.state.current_action = MultiPressAction::LongPress;
            }

            self.state.is_held = false;
        }
    }

    /// Update coordinates based on current action
    fn update_coordinates(&mut self, dt: Duration) {
        let dt_secs = dt.as_secs_f64();
        let base_movement = self.config.base_speed * dt_secs;

        if !self.paused {
            match self.state.current_action {
                MultiPressAction::SingleTap => {
                    // Move forward
                    self.current_coords.x += base_movement * self.state.press_count as f64;
                }
                MultiPressAction::DoubleTap => {
                    // Move backward
                    self.current_coords.x -= base_movement * self.state.press_count as f64;
                }
                MultiPressAction::TripleTap => {
                    // Toggle pause
                    self.paused = !self.paused;
                    self.state.current_action = MultiPressAction::None;
                }
                MultiPressAction::LongPress => {
                    // Zoom effect
                    self.current_coords.y += base_movement;
                }
                MultiPressAction::None => {
                    // Apply inertia/deceleration
                    self.current_coords.x *= 0.95;
                    self.current_coords.y *= 0.95;
                }
            }
        }

        // Apply combo multiplier
        if self.state.press_count > 1 {
            let multiplier = (self.config.combo_multiplier * (self.state.press_count - 1) as f64).min(4.0);
            self.current_coords.x *= multiplier;
            self.current_coords.y *= multiplier;
        }
    }
}

impl InputFilter for MultiPressMode {
    fn process(&mut self, input: &mut dyn DasherInput, time: u64, model: &mut DasherModel, view: &mut dyn DasherView) {
        let now = Instant::now();

        // Handle button state
        if input.is_button_pressed(0) {
            self.handle_press(now);
        } else if self.state.is_held {
            self.handle_release(now);
        }

        // Update coordinates
        let dt = Duration::from_millis(16); // ~60 FPS
        self.update_coordinates(dt);

        // Apply coordinates to model
        model.apply_input_coordinates((self.current_coords.x as i64, self.current_coords.y as i64));
    }

    fn key_down(&mut self, _time: u64, _key: VirtualKey, _model: &mut DasherModel, _view: &mut dyn DasherView) {}

    fn key_up(&mut self, _time: u64, _key: VirtualKey, _model: &mut DasherModel, _view: &mut dyn DasherView) {}

    fn supports_pause(&self) -> bool { true }

    fn pause(&mut self) {
        self.paused = true;
    }

    fn unpause(&mut self) {
        self.paused = false;
    }

    fn is_paused(&self) -> bool {
        self.paused
    }

    fn activate(&mut self) {
        self.reset();
    }

    fn deactivate(&mut self) {
        self.reset();
    }

    fn decorate_view(&mut self, _view: &mut dyn DasherView) -> bool {
        false
    }

    fn reset(&mut self) {
        self.state = MultiPressState::new();
        self.current_coords = Coordinates::default();
        self.paused = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn test_multi_press_basic() {
        let config = MultiPressConfig::default();
        let mut mode = MultiPressMode::new(config);

        // Test initial state
        assert_eq!(mode.state.press_count, 0);
        assert_eq!(mode.state.current_action, MultiPressAction::None);

        // Test single press
        let now = Instant::now();
        mode.handle_press(now);
        assert_eq!(mode.state.press_count, 1);
        assert_eq!(mode.state.current_action, MultiPressAction::SingleTap);

        // Test release
        thread::sleep(Duration::from_millis(100));
        let now = Instant::now();
        mode.handle_release(now);
        assert!(!mode.state.is_held);
    }

    #[test]
    fn test_multi_press_combo() {
        let config = MultiPressConfig::default();
        let mut mode = MultiPressMode::new(config);

        // Test double tap
        let now = Instant::now();
        mode.handle_press(now);
        thread::sleep(Duration::from_millis(100));
        let now = Instant::now();
        mode.handle_press(now);
        assert_eq!(mode.state.press_count, 2);
        assert_eq!(mode.state.current_action, MultiPressAction::DoubleTap);
    }
}
