use crate::input::filter::DasherInputExt;

mod dynamic_filter;
mod multi_press;

pub use dynamic_filter::{OneButtonDynamicFilter, DynamicFilterConfig};
pub use multi_press::{MultiPressMode, MultiPressConfig};

use crate::{DasherInput, input::{InputFilter, Coordinates, VirtualKey}};
use crate::model::DasherModel;
use crate::view::DasherView;

/// Button mode type
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ButtonMode {
    /// Direct mode - button press moves in direction
    Direct,
    /// Dynamic mode - button press toggles movement
    Dynamic,
    /// Multi-press mode - multiple presses for different actions
    MultiPress,
}

/// Button input handler configuration
#[derive(Debug, Clone)]
pub struct ButtonConfig {
    /// Button mode
    pub mode: ButtonMode,
    /// Dynamic filter configuration
    pub dynamic_config: DynamicFilterConfig,
    /// Multi-press configuration
    pub multi_press_config: MultiPressConfig,
}

impl Default for ButtonConfig {
    fn default() -> Self {
        Self {
            mode: ButtonMode::Dynamic,
            dynamic_config: DynamicFilterConfig::default(),
            multi_press_config: MultiPressConfig::default(),
        }
    }
}

/// Button input handler
#[derive(Debug)]
pub struct ButtonHandler {
    /// Configuration
    config: ButtonConfig,
    /// Dynamic filter
    dynamic_filter: Option<OneButtonDynamicFilter>,
    /// Multi-press mode
    multi_press: Option<MultiPressMode>,
    /// Current coordinates
    current_coords: Coordinates,
}

impl ButtonHandler {
    /// Create a new button handler
    pub fn new(config: ButtonConfig) -> Self {
        let dynamic_filter = if config.mode == ButtonMode::Dynamic {
            Some(OneButtonDynamicFilter::new(config.dynamic_config.clone()))
        } else {
            None
        };

        let multi_press = if config.mode == ButtonMode::MultiPress {
            Some(MultiPressMode::new(config.multi_press_config.clone()))
        } else {
            None
        };

        Self {
            config,
            dynamic_filter,
            multi_press,
            current_coords: Coordinates::default(),
        }
    }

    /// Set button mode
    pub fn set_mode(&mut self, mode: ButtonMode) {
        if self.config.mode == mode {
            return;
        }

        self.config.mode = mode;
        match mode {
            ButtonMode::Dynamic => {
                self.dynamic_filter = Some(OneButtonDynamicFilter::new(self.config.dynamic_config.clone()));
                self.multi_press = None;
            }
            ButtonMode::MultiPress => {
                self.dynamic_filter = None;
                self.multi_press = Some(MultiPressMode::new(self.config.multi_press_config.clone()));
            }
            ButtonMode::Direct => {
                self.dynamic_filter = None;
                self.multi_press = None;
            }
        }
        self.current_coords = Coordinates::default();
    }

    /// Get current button mode
    pub fn mode(&self) -> ButtonMode {
        self.config.mode
    }
}

impl InputFilter for ButtonHandler {
    fn reset(&mut self) {
        // Reset the button handler state
        match self.config.mode {
            ButtonMode::Direct => {
                // Reset direct mode state
                // self.last_button_state = false; // Removed: field does not exist
            },
            ButtonMode::Dynamic => {
                // Reset dynamic filter state
                if let Some(filter) = &mut self.dynamic_filter {
                    filter.reset();
                }
            },
            ButtonMode::MultiPress => {
                // Reset multi-press state
                if let Some(handler) = &mut self.multi_press {
                    handler.reset();
                }
            },
        }
    }
    
    fn process(&mut self, input: &mut dyn DasherInput, time: u64, model: &mut DasherModel, view: &mut dyn DasherView) {
        match self.config.mode {
            ButtonMode::Dynamic => {
                if let Some(filter) = &mut self.dynamic_filter {
                    filter.process(input, time, model, view);
                }
            }
            ButtonMode::MultiPress => {
                if let Some(filter) = &mut self.multi_press {
                    filter.process(input, time, model, view);
                }
            }
            ButtonMode::Direct => {
                // Handle direct mode
                if input.is_button_pressed(0) {
                    // model.move_forward(); // Removed: method does not exist
                }
            }
        }
    }

    fn key_down(&mut self, time: u64, key: VirtualKey, model: &mut DasherModel, view: &mut dyn DasherView) {
        match self.config.mode {
            ButtonMode::Dynamic => {
                if let Some(filter) = &mut self.dynamic_filter {
                    filter.key_down(time, key, model, view);
                }
            }
            ButtonMode::MultiPress => {
                if let Some(filter) = &mut self.multi_press {
                    filter.key_down(time, key, model, view);
                }
            }
            ButtonMode::Direct => {}
        }
    }

    fn key_up(&mut self, time: u64, key: VirtualKey, model: &mut DasherModel, view: &mut dyn DasherView) {
        match self.config.mode {
            ButtonMode::Dynamic => {
                if let Some(filter) = &mut self.dynamic_filter {
                    filter.key_up(time, key, model, view);
                }
            }
            ButtonMode::MultiPress => {
                if let Some(filter) = &mut self.multi_press {
                    filter.key_up(time, key, model, view);
                }
            }
            ButtonMode::Direct => {}
        }
    }

    fn supports_pause(&self) -> bool {
        match self.config.mode {
            ButtonMode::Dynamic => self.dynamic_filter.as_ref().map_or(false, |f| f.supports_pause()),
            ButtonMode::MultiPress => self.multi_press.as_ref().map_or(false, |f| f.supports_pause()),
            ButtonMode::Direct => false,
        }
    }

    fn pause(&mut self) {
        match self.config.mode {
            ButtonMode::Dynamic => {
                if let Some(filter) = &mut self.dynamic_filter {
                    filter.pause();
                }
            }
            ButtonMode::MultiPress => {
                if let Some(filter) = &mut self.multi_press {
                    filter.pause();
                }
            }
            ButtonMode::Direct => {}
        }
    }

    fn unpause(&mut self) {
        match self.config.mode {
            ButtonMode::Dynamic => {
                if let Some(filter) = &mut self.dynamic_filter {
                    filter.unpause();
                }
            }
            ButtonMode::MultiPress => {
                if let Some(filter) = &mut self.multi_press {
                    filter.unpause();
                }
            }
            ButtonMode::Direct => {}
        }
    }

    fn is_paused(&self) -> bool {
        match self.config.mode {
            ButtonMode::Dynamic => self.dynamic_filter.as_ref().map_or(false, |f| f.is_paused()),
            ButtonMode::MultiPress => self.multi_press.as_ref().map_or(false, |f| f.is_paused()),
            ButtonMode::Direct => false,
        }
    }

    fn activate(&mut self) {
        match self.config.mode {
            ButtonMode::Dynamic => {
                if let Some(filter) = &mut self.dynamic_filter {
                    filter.activate();
                }
            }
            ButtonMode::MultiPress => {
                if let Some(filter) = &mut self.multi_press {
                    filter.activate();
                }
            }
            ButtonMode::Direct => {
                self.current_coords = Coordinates::default();
            }
        }
    }

    fn deactivate(&mut self) {
        match self.config.mode {
            ButtonMode::Dynamic => {
                if let Some(filter) = &mut self.dynamic_filter {
                    filter.deactivate();
                }
            }
            ButtonMode::MultiPress => {
                if let Some(filter) = &mut self.multi_press {
                    filter.deactivate();
                }
            }
            ButtonMode::Direct => {
                self.current_coords = Coordinates::default();
            }
        }
    }

    fn decorate_view(&mut self, view: &mut dyn DasherView) -> bool {
        match self.config.mode {
            ButtonMode::Dynamic => self.dynamic_filter.as_mut().map_or(false, |f| f.decorate_view(view)),
            ButtonMode::MultiPress => self.multi_press.as_mut().map_or(false, |f| f.decorate_view(view)),
            ButtonMode::Direct => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_button_handler_modes() {
        let config = ButtonConfig::default();
        let mut handler = ButtonHandler::new(config);

        // Test mode switching
        assert_eq!(handler.mode(), ButtonMode::Dynamic);
        handler.set_mode(ButtonMode::MultiPress);
        assert_eq!(handler.mode(), ButtonMode::MultiPress);
        handler.set_mode(ButtonMode::Direct);
        assert_eq!(handler.mode(), ButtonMode::Direct);
    }

    #[test]
    fn test_button_handler_reset() {
        let config = ButtonConfig::default();
        let mut handler = ButtonHandler::new(config);

        // Test reset in each mode
        handler.set_mode(ButtonMode::Dynamic);
        handler.reset();
        handler.set_mode(ButtonMode::MultiPress);
        handler.reset();
        handler.set_mode(ButtonMode::Direct);
        handler.reset();
    }
}
