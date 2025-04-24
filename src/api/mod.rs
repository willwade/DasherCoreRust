//! # API Module
//!
//! This module contains the main API for the Dasher core.

use std::cell::RefCell;
use std::rc::Rc;

use crate::model::{DasherModel, DasherNode};
use crate::view::{DasherScreen, DasherView, DasherViewSquare, Orientation};
use crate::input::{DasherInput, InputFilter, DefaultFilter, InputManager, VirtualKey};
use crate::settings::{Settings, Parameter};
use crate::Result;

/// The main interface for the Dasher core.
///
/// This is the central class that ties together all the components of Dasher
/// and provides a single interface for the UI to use.
pub struct DasherInterface {
    /// The Dasher model
    model: DasherModel,

    /// The Dasher view
    view: Option<Box<dyn DasherView>>,

    /// The input manager
    input_manager: InputManager,

    /// The settings
    settings: Settings,

    /// Whether Dasher is running
    running: bool,

    /// Whether Dasher is paused
    paused: bool,

    /// The current frame time
    current_time: u64,
}

impl DasherInterface {
    /// Create a new Dasher interface
    pub fn new(settings: Settings) -> Self {
        let mut model = DasherModel::new();

        // Initialize the model
        if let Err(e) = model.initialize() {
            eprintln!("Failed to initialize model: {:?}", e);
        }

        Self {
            model,
            view: None,
            input_manager: InputManager::new(),
            settings,
            running: false,
            paused: false,
            current_time: 0,
        }
    }

    /// Set the screen for rendering
    pub fn change_screen(&mut self, screen: Box<dyn DasherScreen>) -> Result<()> {
        // Create a new view
        let mut view = Box::new(DasherViewSquare::new(screen));

        // Set the orientation based on settings
        let orientation = match self.settings.get_long(Parameter::Orientation).unwrap_or(0) {
            0 => Orientation::LeftToRight,
            1 => Orientation::RightToLeft,
            2 => Orientation::TopToBottom,
            3 => Orientation::BottomToTop,
            _ => Orientation::LeftToRight,
        };
        view.set_orientation(orientation);

        // Store the view
        self.view = Some(view);

        Ok(())
    }

    /// Set the input device
    pub fn set_input(&mut self, input: Box<dyn DasherInput>) {
        // Set the input device in the input manager
        self.input_manager.set_input_device(input);

        // Also set it in the view if available
        if let Some(view) = &mut self.view {
            if let Some(input) = self.input_manager.get_input_device() {
                view.set_input_device(input.box_clone());
            }
        }
    }

    /// Set the input filter
    pub fn set_input_filter(&mut self, filter: Box<dyn InputFilter>) {
        self.input_manager.set_input_filter(filter);
    }

    /// Process a new frame
    pub fn new_frame(&mut self, time_ms: u64) -> bool {
        // Update the current time
        self.current_time = time_ms;

        // If not running, do nothing
        if !self.running {
            return false;
        }

        // If paused, just render
        if self.paused {
            if let Some(view) = &mut self.view {
                return view.render(&mut self.model).is_ok();
            }
            return false;
        }

        // Process input
        if let Some(view) = &mut self.view {
            self.input_manager.process_frame(time_ms, &mut self.model, view.as_mut());

            // Process the next scheduled step in the model
            self.model.next_scheduled_step();

            // Render the view
            return view.render(&mut self.model).is_ok();
        }

        false
    }

    /// Handle a key down event
    pub fn key_down(&mut self, time_ms: u64, key: VirtualKey) {
        // Update the current time
        self.current_time = time_ms;

        // If not running, check for start key
        if !self.running && key == VirtualKey::StartStopKey {
            self.start();
            return;
        }

        // If running, process the key
        if self.running {
            if let Some(view) = &mut self.view {
                self.input_manager.key_down(time_ms, key, &mut self.model, view.as_mut());
            }
        }
    }

    /// Handle a key up event
    pub fn key_up(&mut self, time_ms: u64, key: VirtualKey) {
        // Update the current time
        self.current_time = time_ms;

        // If running, process the key
        if self.running {
            if let Some(view) = &mut self.view {
                self.input_manager.key_up(time_ms, key, &mut self.model, view.as_mut());
            }
        }
    }

    /// Start Dasher
    pub fn start(&mut self) {
        self.running = true;
        self.paused = false;

        // Create a root node if needed
        if self.model.get_node_under_crosshair().is_none() {
            let root = Rc::new(RefCell::new(DasherNode::new(0, Some("Root".to_string()))));
            self.model.set_node(root);
        }

        // Resume input processing
        self.input_manager.resume();
    }

    /// Stop Dasher
    pub fn stop(&mut self) {
        self.running = false;
        self.paused = false;

        // Pause input processing
        self.input_manager.pause();
    }

    /// Pause Dasher
    pub fn pause(&mut self) {
        if self.running {
            self.paused = true;

            // Pause input processing
            self.input_manager.pause();
        }
    }

    /// Resume Dasher
    pub fn resume(&mut self) {
        if self.running && self.paused {
            self.paused = false;

            // Resume input processing
            self.input_manager.resume();
        }
    }

    /// Check if Dasher is running
    pub fn is_running(&self) -> bool {
        self.running
    }

    /// Check if Dasher is paused
    pub fn is_paused(&self) -> bool {
        self.paused
    }

    /// Get a reference to the settings
    pub fn settings(&self) -> &Settings {
        &self.settings
    }

    /// Get a mutable reference to the settings
    pub fn settings_mut(&mut self) -> &mut Settings {
        &mut self.settings
    }

    /// Get a reference to the model
    pub fn model(&self) -> &DasherModel {
        &self.model
    }

    /// Get a mutable reference to the model
    pub fn model_mut(&mut self) -> &mut DasherModel {
        &mut self.model
    }

    /// Get a reference to the view
    pub fn view(&self) -> Option<&dyn DasherView> {
        self.view.as_deref()
    }

    /// Get a mutable reference to the view
    pub fn view_mut(&mut self) -> Option<&mut dyn DasherView> {
        if let Some(view) = &mut self.view {
            Some(view.as_mut())
        } else {
            None
        }
    }

    /// Get the current offset in the text buffer
    pub fn get_offset(&self) -> i32 {
        self.model.get_offset()
    }

    /// Edit the output text
    pub fn edit_output(&mut self, text: &str) {
        // Set the output text in the model
        self.model.set_output_text(text);
    }

    /// Get the current output text
    pub fn get_output_text(&self) -> &str {
        self.model.output_text()
    }

    /// Handle a parameter change
    pub fn handle_parameter_change(&mut self, parameter: Parameter) {
        match parameter {
            Parameter::Orientation => {
                if let Some(view) = &mut self.view {
                    let orientation = match self.settings.get_long(Parameter::Orientation).unwrap_or(0) {
                        0 => Orientation::LeftToRight,
                        1 => Orientation::RightToLeft,
                        2 => Orientation::TopToBottom,
                        3 => Orientation::BottomToTop,
                        _ => Orientation::LeftToRight,
                    };
                    view.set_orientation(orientation);
                }
            }
            _ => {}
        }
    }
}
