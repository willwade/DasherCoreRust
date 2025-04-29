//! # Input Module
//!
//! This module contains the implementation of input handling for Dasher.

mod filter;
mod device;
mod button;
mod circle_start;
mod frame_rate;
mod dynamic_filter;
mod demo_filter;


use crate::model::DasherModel;
use crate::view::DasherView;

pub use filter::{InputFilter, DefaultFilter};
pub use device::{DasherInput, MouseInput};
pub use button::{ButtonHandler, ButtonConfig, ButtonMode};
pub use circle_start::{CircleStartHandler, CircleStartConfig};
pub use frame_rate::FrameRate;
pub use dynamic_filter::{DynamicFilter, DynamicFilterBase, DynamicFilterConfig};
pub use demo_filter::{DemoDynamicFilter, DemoDynamicFilterConfig};
pub use button::one_button_dynamic_filter::{OneButtonDynamicFilter, OneButtonDynamicFilterConfig};
pub use button::two_button_dynamic_filter::{TwoButtonDynamicFilter, TwoButtonDynamicFilterConfig};

// --- ADDED: Stub update traits for input handlers ---
trait UpdatableInputHandler {
    fn update(&mut self, device: &dyn DasherInput, model: &mut DasherModel, view: &mut dyn DasherView);
}

impl UpdatableInputHandler for Box<dyn InputFilter> {
    fn update(&mut self, device: &dyn DasherInput, model: &mut DasherModel, view: &mut dyn DasherView) {
        // Clone the device to avoid borrowing issues
        let mut device_clone = device.box_clone();
        // Call the process method to handle the input filter logic
        self.as_mut().process(&mut *device_clone, 0, model, view);
    }
}

impl UpdatableInputHandler for crate::input::CircleStartHandler {
    fn update(&mut self, device: &dyn DasherInput, model: &mut DasherModel, view: &mut dyn DasherView) {
        // Clone the device to avoid borrowing issues
        let mut device_clone = device.box_clone();
        // Call the process method to handle the circle start logic
        self.process(&mut *device_clone, 0, model, view);
    }
}

impl UpdatableInputHandler for crate::input::button::ButtonHandler {
    fn update(&mut self, device: &dyn DasherInput, model: &mut DasherModel, view: &mut dyn DasherView) {
        // Process button input
        if let Some(coords) = device.get_screen_coordinates(view) {
            // Update the button state based on the coordinates
            self.update_state(coords.0, coords.1, model);
        }
    }
}
// --- END ADDED ---

/// Virtual key codes for keyboard input
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum VirtualKey {
    /// Primary input key (usually left mouse button)
    PrimaryInput,

    /// Secondary input key (usually right mouse button)
    SecondaryInput,

    /// Tertiary input key (usually middle mouse button)
    TertiaryInput,

    /// Start/stop key (usually space)
    StartStopKey,

    /// Button 1
    Button1,

    /// Button 2
    Button2,

    /// Button 3
    Button3,

    /// Button 4
    Button4,

    /// Button 5
    Button5,

    /// Left arrow key
    Left,

    /// Right arrow key
    Right,

    /// Up arrow key
    Up,

    /// Down arrow key
    Down,

    /// Delete key
    Delete,

    /// Backspace key
    Backspace,

    /// Tab key
    Tab,

    /// Return key
    Return,

    /// Escape key
    Escape,

    /// Space key
    Space,

    /// Any other key
    Other(char),
}

/// Input coordinates
#[derive(Debug, Clone, Copy, Default)]
pub struct Coordinates {
    /// X coordinate
    pub x: f64,
    /// Y coordinate
    pub y: f64,
}

/// Input device trait
pub trait InputDevice {
    /// Get current coordinates
    fn get_coordinates(&self) -> Coordinates;
    /// Check if a button is pressed
    fn is_button_pressed(&self, button: u32) -> bool;
}

/// Input manager that handles input devices and filters
pub struct InputManager {
    /// Current input device
    input_device: Option<Box<dyn DasherInput>>,

    /// Current input filter
    input_filter: Option<Box<dyn InputFilter>>,

    /// Button handler
    button_handler: Option<ButtonHandler>,

    /// Circle start handler
    circle_start: Option<CircleStartHandler>,

    /// Whether the input is paused
    paused: bool,
}

impl Default for InputManager {
    fn default() -> Self {
        Self::new()
    }
}

impl InputManager {
    /// Create a new input manager
    pub fn new() -> Self {
        Self {
            input_device: None,
            input_filter: Some(Box::new(DefaultFilter::new())),
            button_handler: Some(ButtonHandler::new(ButtonConfig::default())),
            circle_start: Some(CircleStartHandler::new(CircleStartConfig::default())),
            paused: false,
        }
    }

    /// Set button mode
    pub fn set_button_mode(&mut self, mode: ButtonMode) {
        if let Some(handler) = &mut self.button_handler {
            handler.set_mode(mode);
        }
    }

    /// Get current button mode
    pub fn button_mode(&self) -> Option<ButtonMode> {
        self.button_handler.as_ref().map(|h| h.mode())
    }

    /// Enable/disable circle start
    pub fn set_circle_start_enabled(&mut self, enabled: bool) {
        if enabled && self.circle_start.is_none() {
            self.circle_start = Some(CircleStartHandler::new(CircleStartConfig::default()));
        } else if !enabled {
            self.circle_start = None;
        }
    }

    /// Check if circle start is enabled
    pub fn is_circle_start_enabled(&self) -> bool {
        self.circle_start.is_some()
    }

    /// Set the input device
    pub fn set_input_device(&mut self, device: Box<dyn DasherInput>) {
        self.input_device = Some(device);
    }

    /// Get a reference to the input device
    pub fn get_input_device(&self) -> Option<&dyn DasherInput> {
        self.input_device.as_deref()
    }

    /// Set the mouse position for the input device
    pub fn set_mouse_position(&mut self, x: i32, y: i32) -> Result<(), crate::DasherError> {
        if let Some(input) = &mut self.input_device {
            // Use a match to avoid borrowing issues
            match input.as_mut() {
                input => {
                    input.set_screen_position(x, y);
                    Ok(())
                }
            }
        } else {
            Err(crate::DasherError::InputError("No input device available".to_string()))
        }
    }

    /// Set the input filter
    pub fn set_input_filter(&mut self, filter: Box<dyn InputFilter>) {
        self.input_filter = Some(filter);
    }

    /// Process input for a frame
    pub fn process_frame(&mut self, _time: u64, model: &mut DasherModel, view: &mut dyn DasherView) {
        if self.paused {
            return;
        }

        // Process circle start first if enabled
        if let Some(circle) = &mut self.circle_start {
            if let Some(device) = &self.input_device {
                circle.update(device.as_ref(), model, view);
            }
        }

        // Process button handler
        if let Some(handler) = &mut self.button_handler {
            if let Some(device) = &self.input_device {
                handler.update(device.as_ref(), model, view);
            }
        }

        // Process main input filter
        if let Some(filter) = &mut self.input_filter {
            if let Some(device) = &self.input_device {
                filter.update(device.as_ref(), model, view);
            }
        }
    }

    /// Handle a key down event
    pub fn key_down(&mut self, time: u64, key: VirtualKey, model: &mut DasherModel, view: &mut dyn DasherView) {
        if self.paused {
            return;
        }

        if let Some(filter) = &mut self.input_filter {
            filter.key_down(time, key, model, view);
        }

        if let Some(input) = &mut self.input_device {
            input.key_down(time, key);
        }
    }

    /// Handle a key up event
    pub fn key_up(&mut self, time: u64, key: VirtualKey, model: &mut DasherModel, view: &mut dyn DasherView) {
        if self.paused {
            return;
        }

        if let Some(filter) = &mut self.input_filter {
            filter.key_up(time, key, model, view);
        }

        if let Some(input) = &mut self.input_device {
            input.key_up(time, key);
        }
    }

    /// Pause input processing
    pub fn pause(&mut self) {
        self.paused = true;

        if let Some(filter) = &mut self.input_filter {
            filter.pause();
        }
    }

    /// Resume input processing
    pub fn resume(&mut self) {
        self.paused = false;

        if let Some(filter) = &mut self.input_filter {
            filter.unpause();
        }
    }

    /// Check if input processing is paused
    pub fn is_paused(&self) -> bool {
        self.paused
    }

    /// Reset the input manager
    pub fn reset(&mut self) {
        // Reset the input filter
        if let Some(filter) = &mut self.input_filter {
            filter.reset();
        }

        // Reset the button handler
        if let Some(handler) = &mut self.button_handler {
            handler.reset();
        }

        // Reset the circle start handler
        if let Some(circle) = &mut self.circle_start {
            circle.reset();
        }
    }
}
