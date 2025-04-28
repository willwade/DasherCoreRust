//! # Input Device Module
//!
//! This module contains the implementation of input devices for Dasher.

use super::VirtualKey;
use crate::view::DasherView;

/// Interface for input devices
pub trait DasherInput {
    /// Get the coordinates from the input device in Dasher coordinates
    fn get_dasher_coordinates(&mut self, view: &dyn DasherView) -> Option<(i64, i64)>;

    /// Get the coordinates from the input device in screen coordinates
    fn get_screen_coordinates(&mut self, view: &dyn DasherView) -> Option<(i32, i32)>;

    /// Get the name of the input device
    fn get_name(&self) -> &str;

    /// Check if the input device supports pause
    fn supports_pause(&self) -> bool;

    /// Activate the device
    fn activate(&mut self);

    /// Deactivate the device
    fn deactivate(&mut self);

    /// Handle key down events
    fn key_down(&mut self, time: u64, key: VirtualKey);

    /// Handle key up events
    fn key_up(&mut self, time: u64, key: VirtualKey);

    /// Clone the input device into a Box
    fn box_clone(&self) -> Box<dyn DasherInput>;
}

/// Mouse input implementation
#[derive(Clone)]
pub struct MouseInput {
    /// Name of the input device
    name: String,

    /// X coordinate in screen space
    x: i32,

    /// Y coordinate in screen space
    y: i32,

    /// Whether the device is active
    active: bool,
}

impl Default for MouseInput {
    fn default() -> Self {
        Self::new()
    }
}

impl MouseInput {
    /// Create a new mouse input
    pub fn new() -> Self {
        Self {
            name: "Mouse".to_string(),
            x: 0,
            y: 0,
            active: false,
        }
    }

    /// Set the coordinates
    pub fn set_coordinates(&mut self, x: i32, y: i32) {
        self.x = x;
        self.y = y;
    }
}

impl DasherInput for MouseInput {
    fn get_dasher_coordinates(&mut self, view: &dyn DasherView) -> Option<(i64, i64)> {
        if !self.active {
            return None;
        }

        let (x, y) = view.screen_to_dasher(self.x, self.y);
        Some((x, y))
    }

    fn get_screen_coordinates(&mut self, _view: &dyn DasherView) -> Option<(i32, i32)> {
        if !self.active {
            return None;
        }

        Some((self.x, self.y))
    }

    fn get_name(&self) -> &str {
        &self.name
    }

    fn supports_pause(&self) -> bool {
        true
    }

    fn activate(&mut self) {
        self.active = true;
    }

    fn deactivate(&mut self) {
        self.active = false;
    }

    fn key_down(&mut self, _time: u64, key: VirtualKey) {
        // Handle mouse button presses
        match key {
            VirtualKey::PrimaryInput => {
                // Left mouse button
            }
            VirtualKey::SecondaryInput => {
                // Right mouse button
            }
            VirtualKey::TertiaryInput => {
                // Middle mouse button
            }
            _ => {}
        }
    }

    fn key_up(&mut self, _time: u64, key: VirtualKey) {
        // Handle mouse button releases
        match key {
            VirtualKey::PrimaryInput => {
                // Left mouse button
            }
            VirtualKey::SecondaryInput => {
                // Right mouse button
            }
            VirtualKey::TertiaryInput => {
                // Middle mouse button
            }
            _ => {}
        }
    }

    fn box_clone(&self) -> Box<dyn DasherInput> {
        Box::new(self.clone())
    }
}

/// One-dimensional input implementation
#[derive(Clone)]
pub struct OneDimensionalInput {
    /// Name of the input device
    name: String,

    /// Y coordinate in screen space
    y: i32,

    /// Whether the device is active
    active: bool,
}

impl OneDimensionalInput {
    /// Create a new one-dimensional input
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self {
            name: "One-Dimensional".to_string(),
            y: 0,
            active: false,
        }
    }

    /// Set the Y coordinate
    #[allow(dead_code)]
    pub fn set_y_coordinate(&mut self, y: i32) {
        self.y = y;
    }
}

impl DasherInput for OneDimensionalInput {
    fn get_dasher_coordinates(&mut self, view: &dyn DasherView) -> Option<(i64, i64)> {
        if !self.active {
            return None;
        }

        // In one-dimensional mode, the X coordinate is always 0
        let (_, y) = view.screen_to_dasher(0, self.y);
        Some((0, y))
    }

    fn get_screen_coordinates(&mut self, view: &dyn DasherView) -> Option<(i32, i32)> {
        if !self.active {
            return None;
        }

        // In one-dimensional mode, the X coordinate depends on the orientation
        let (width, _) = view.get_dimensions();
        Some((width / 2, self.y))
    }

    fn get_name(&self) -> &str {
        &self.name
    }

    fn supports_pause(&self) -> bool {
        true
    }

    fn activate(&mut self) {
        self.active = true;
    }

    fn deactivate(&mut self) {
        self.active = false;
    }

    fn key_down(&mut self, _time: u64, _key: VirtualKey) {
        // Nothing to do
    }

    fn key_up(&mut self, _time: u64, _key: VirtualKey) {
        // Nothing to do
    }

    fn box_clone(&self) -> Box<dyn DasherInput> {
        Box::new(self.clone())
    }
}

/// Eye tracker input implementation
#[derive(Clone)]
pub struct EyeTrackerInput {
    /// Name of the input device
    name: String,

    /// X coordinate in screen space
    x: i32,

    /// Y coordinate in screen space
    y: i32,

    /// Whether the device is active
    active: bool,

    /// Smoothing factor for eye tracking
    #[allow(dead_code)]
    smoothing_factor: f64,

    /// Previous X coordinate
    #[allow(dead_code)]
    prev_x: i32,

    /// Previous Y coordinate
    #[allow(dead_code)]
    prev_y: i32,
}

impl EyeTrackerInput {
    /// Create a new eye tracker input
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self {
            name: "Eye Tracker".to_string(),
            x: 0,
            y: 0,
            active: false,
            smoothing_factor: 0.8,
            prev_x: 0,
            prev_y: 0,
        }
    }

    /// Set the coordinates
    #[allow(dead_code)]
    pub fn set_coordinates(&mut self, x: i32, y: i32) {
        // Apply smoothing
        self.x = ((self.prev_x as f64 * self.smoothing_factor) + (x as f64 * (1.0 - self.smoothing_factor))) as i32;
        self.y = ((self.prev_y as f64 * self.smoothing_factor) + (y as f64 * (1.0 - self.smoothing_factor))) as i32;

        self.prev_x = self.x;
        self.prev_y = self.y;
    }

    /// Set the smoothing factor
    #[allow(dead_code)]
    pub fn set_smoothing_factor(&mut self, factor: f64) {
        self.smoothing_factor = factor.max(0.0).min(1.0);
    }
}

impl DasherInput for EyeTrackerInput {
    fn get_dasher_coordinates(&mut self, view: &dyn DasherView) -> Option<(i64, i64)> {
        if !self.active {
            return None;
        }

        let (x, y) = view.screen_to_dasher(self.x, self.y);
        Some((x, y))
    }

    fn get_screen_coordinates(&mut self, _view: &dyn DasherView) -> Option<(i32, i32)> {
        if !self.active {
            return None;
        }

        Some((self.x, self.y))
    }

    fn get_name(&self) -> &str {
        &self.name
    }

    fn supports_pause(&self) -> bool {
        true
    }

    fn activate(&mut self) {
        self.active = true;
    }

    fn deactivate(&mut self) {
        self.active = false;
    }

    fn key_down(&mut self, _time: u64, _key: VirtualKey) {
        // Nothing to do
    }

    fn key_up(&mut self, _time: u64, _key: VirtualKey) {
        // Nothing to do
    }

    fn box_clone(&self) -> Box<dyn DasherInput> {
        Box::new(self.clone())
    }
}
