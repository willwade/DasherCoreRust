//! # Input Module
//!
//! This module contains the implementation of input handling for Dasher.

mod filter;
mod device;

use std::cell::RefCell;
use std::rc::Rc;

use crate::model::DasherModel;
use crate::view::DasherView;

pub use filter::{InputFilter, DefaultFilter};
pub use device::{DasherInput, MouseInput};

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

/// Input manager that handles input devices and filters
pub struct InputManager {
    /// Current input device
    input_device: Option<Box<dyn DasherInput>>,
    
    /// Current input filter
    input_filter: Option<Box<dyn InputFilter>>,
    
    /// Whether the input is paused
    paused: bool,
}

impl InputManager {
    /// Create a new input manager
    pub fn new() -> Self {
        Self {
            input_device: None,
            input_filter: Some(Box::new(DefaultFilter::new())),
            paused: false,
        }
    }
    
    /// Set the input device
    pub fn set_input_device(&mut self, device: Box<dyn DasherInput>) {
        self.input_device = Some(device);
    }
    
    /// Get a reference to the input device
    pub fn get_input_device(&self) -> Option<&dyn DasherInput> {
        self.input_device.as_deref()
    }
    
    /// Set the input filter
    pub fn set_input_filter(&mut self, filter: Box<dyn InputFilter>) {
        self.input_filter = Some(filter);
    }
    
    /// Process input for a frame
    pub fn process_frame(&mut self, time: u64, model: &mut DasherModel, view: &mut dyn DasherView) {
        if self.paused {
            return;
        }
        
        if let (Some(input), Some(filter)) = (&mut self.input_device, &mut self.input_filter) {
            filter.process(input.as_mut(), time, model, view);
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
}
