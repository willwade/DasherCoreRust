//! # Input Filter Module
//!
//! This module contains the implementation of input filters for Dasher.

use super::VirtualKey;
use crate::model::DasherModel;
use crate::view::DasherView;
use crate::input::DasherInput;

/// Input filter interface
pub trait InputFilter {
    /// Process input and update the model
    fn process(&mut self, input: &mut dyn DasherInput, time: u64, model: &mut DasherModel, view: &mut dyn DasherView);

    /// Handle key down events
    fn key_down(&mut self, time: u64, key: VirtualKey, model: &mut DasherModel, view: &mut dyn DasherView);

    /// Handle key up events
    fn key_up(&mut self, time: u64, key: VirtualKey, model: &mut DasherModel, view: &mut dyn DasherView);

    /// Check if the filter supports pause
    fn supports_pause(&self) -> bool;

    /// Pause the filter
    fn pause(&mut self);

    /// Unpause the filter
    fn unpause(&mut self);

    /// Check if the filter is paused
    fn is_paused(&self) -> bool;

    /// Activate the filter
    fn activate(&mut self);

    /// Deactivate the filter
    fn deactivate(&mut self);

    /// Decorate the view with filter-specific elements
    fn decorate_view(&mut self, view: &mut dyn DasherView) -> bool;
}

/// Default input filter implementation
pub struct DefaultFilter {
    /// Whether the filter is paused
    paused: bool,

    /// Whether the filter is in turbo mode
    turbo: bool,

    /// Last known Dasher coordinates
    last_x: i64,
    last_y: i64,

    /// Whether we have valid mouse coordinates
    got_mouse_coords: bool,
}

impl DefaultFilter {
    /// Create a new default filter
    pub fn new() -> Self {
        Self {
            paused: false,
            turbo: false,
            last_x: 0,
            last_y: 0,
            got_mouse_coords: false,
        }
    }

    /// Apply a transform to the coordinates
    fn apply_transform(&mut self, _x: &mut i64, _y: &mut i64, _view: &dyn DasherView) {
        // Default implementation does nothing
    }

    /// Execute a movement based on the coordinates
    fn execute_movement(&mut self, time: u64, model: &mut DasherModel, view: &mut dyn DasherView, new_x: i64, new_y: i64) {
        // Store the coordinates
        self.last_x = new_x;
        self.last_y = new_y;
        self.got_mouse_coords = true;

        // Apply any transformations
        let mut x = new_x;
        let mut y = new_y;
        self.apply_transform(&mut x, &mut y, view);

        if !self.is_paused() {
            // Check if we're outside the visible region
            let (min_x, min_y, max_x, max_y) = view.get_visible_region();

            if x > max_x || x < min_x || y > max_y || y < min_y {
                self.pause();
                return;
            }

            // Calculate the speed multiplier
            let speed_mul = if self.turbo { 1.75 } else { 1.0 };

            // Schedule a step towards the target
            self.one_step_towards(model, x, y, time, speed_mul);
        }
    }

    /// Schedule a step towards the target
    fn one_step_towards(&mut self, model: &mut DasherModel, _x: i64, y: i64, _time: u64, _speed_mul: f64) {
        // Calculate the target range
        let y1 = y - 1800;
        let y2 = y + 1800;

        // Schedule a step
        model.schedule_one_step(y1, y2, 1, 100, false);
    }

    /// Stop the filter
    fn stop(&mut self) {
        self.pause();
    }

    /// Run the filter
    fn run(&mut self, _time: u64) {
        self.unpause();
    }
}

impl InputFilter for DefaultFilter {
    fn process(&mut self, input: &mut dyn DasherInput, time: u64, model: &mut DasherModel, view: &mut dyn DasherView) {
        // Get the coordinates from the input device
        if let Some((x, y)) = input.get_dasher_coordinates(view) {
            self.execute_movement(time, model, view, x, y);
        } else {
            self.got_mouse_coords = false;
            self.stop();
        }
    }

    fn key_down(&mut self, time: u64, key: VirtualKey, _model: &mut DasherModel, _view: &mut dyn DasherView) {
        match key {
            VirtualKey::StartStopKey => {
                if self.is_paused() {
                    self.run(time);
                } else {
                    self.stop();
                }
            }
            VirtualKey::SecondaryInput | VirtualKey::TertiaryInput | VirtualKey::Button1 => {
                self.turbo = true;
            }
            _ => {}
        }
    }

    fn key_up(&mut self, _time: u64, key: VirtualKey, _model: &mut DasherModel, _view: &mut dyn DasherView) {
        match key {
            VirtualKey::SecondaryInput | VirtualKey::TertiaryInput | VirtualKey::Button1 => {
                self.turbo = false;
            }
            _ => {}
        }
    }

    fn supports_pause(&self) -> bool {
        true
    }

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
        // Nothing to do
    }

    fn deactivate(&mut self) {
        // Nothing to do
    }

    fn decorate_view(&mut self, view: &mut dyn DasherView) -> bool {
        if !self.got_mouse_coords {
            return false;
        }

        // Draw a crosshair at the current position
        view.draw_line(self.last_x - 100, self.last_y, self.last_x + 100, self.last_y, (255, 0, 0, 255), 1);
        view.draw_line(self.last_x, self.last_y - 100, self.last_x, self.last_y + 100, (255, 0, 0, 255), 1);

        true
    }
}

/// One-dimensional filter implementation
pub struct OneDimensionalFilter {
    /// Base filter
    base: DefaultFilter,

    /// Maximum forward value
    forward_max: i64,
}

impl OneDimensionalFilter {
    /// Create a new one-dimensional filter
    pub fn new() -> Self {
        Self {
            base: DefaultFilter::new(),
            forward_max: 100,
        }
    }

    /// Apply a transform to the coordinates
    fn apply_transform(&mut self, x: &mut i64, y: &mut i64, _view: &dyn DasherView) {
        // In one-dimensional mode, we only use the Y coordinate
        // and calculate X based on a function of Y
        let y_abs = y.abs() as f64;
        let x_max = (y_abs * y_abs).exp() - 1.0;
        let x_max = x_max.min(self.forward_max as f64);

        *x = x_max as i64;
    }
}

impl InputFilter for OneDimensionalFilter {
    fn process(&mut self, input: &mut dyn DasherInput, time: u64, model: &mut DasherModel, view: &mut dyn DasherView) {
        // Get the coordinates from the input device
        if let Some((_, y)) = input.get_dasher_coordinates(view) {
            // In one-dimensional mode, we calculate X based on Y
            let mut x = 0;
            let mut y_copy = y;
            self.apply_transform(&mut x, &mut y_copy, view);

            // Execute the movement
            self.base.execute_movement(time, model, view, x, y);
        } else {
            self.base.got_mouse_coords = false;
            self.base.stop();
        }
    }

    fn key_down(&mut self, time: u64, key: VirtualKey, model: &mut DasherModel, view: &mut dyn DasherView) {
        self.base.key_down(time, key, model, view);
    }

    fn key_up(&mut self, time: u64, key: VirtualKey, model: &mut DasherModel, view: &mut dyn DasherView) {
        self.base.key_up(time, key, model, view);
    }

    fn supports_pause(&self) -> bool {
        self.base.supports_pause()
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
        self.base.activate();
    }

    fn deactivate(&mut self) {
        self.base.deactivate();
    }

    fn decorate_view(&mut self, view: &mut dyn DasherView) -> bool {
        self.base.decorate_view(view)
    }
}

/// Click filter implementation
pub struct ClickFilter {
    /// Whether the filter is paused
    paused: bool,

    /// Target coordinates
    target_x: i64,
    target_y: i64,

    /// Whether a click has been registered
    clicked: bool,
}

impl ClickFilter {
    /// Create a new click filter
    pub fn new() -> Self {
        Self {
            paused: false,
            target_x: 0,
            target_y: 0,
            clicked: false,
        }
    }
}

impl InputFilter for ClickFilter {
    fn process(&mut self, _input: &mut dyn DasherInput, _time: u64, model: &mut DasherModel, _view: &mut dyn DasherView) {
        // In click mode, we only move when a click is registered
        if self.clicked {
            // Schedule a step towards the target
            let y1 = self.target_y - 1800;
            let y2 = self.target_y + 1800;

            model.schedule_one_step(y1, y2, 1, 100, false);

            // Reset the click flag
            self.clicked = false;
        }
    }

    fn key_down(&mut self, _time: u64, key: VirtualKey, _model: &mut DasherModel, view: &mut dyn DasherView) {
        match key {
            VirtualKey::PrimaryInput => {
                // Get the current mouse coordinates
                if let Some(_input) = view.get_input_device() {
                    // We need a mutable reference to get coordinates
                    // This is a design issue - we should refactor to avoid this
                    // For now, we'll just ignore this case
                    // if let Some((x, y)) = input.get_dasher_coordinates(view) {
                    //     self.target_x = x;
                    //     self.target_y = y;
                    //     self.clicked = true;
                    // }
                }
            }
            VirtualKey::StartStopKey => {
                if self.is_paused() {
                    self.unpause();
                } else {
                    self.pause();
                }
            }
            _ => {}
        }
    }

    fn key_up(&mut self, _time: u64, _key: VirtualKey, _model: &mut DasherModel, _view: &mut dyn DasherView) {
        // Nothing to do
    }

    fn supports_pause(&self) -> bool {
        true
    }

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
        // Nothing to do
    }

    fn deactivate(&mut self) {
        // Nothing to do
    }

    fn decorate_view(&mut self, view: &mut dyn DasherView) -> bool {
        // Draw a crosshair at the target position
        if self.clicked {
            view.draw_line(self.target_x - 100, self.target_y, self.target_x + 100, self.target_y, (0, 255, 0, 255), 1);
            view.draw_line(self.target_x, self.target_y - 100, self.target_x, self.target_y + 100, (0, 255, 0, 255), 1);
            return true;
        }

        false
    }
}
