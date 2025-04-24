//! # View Module
//!
//! This module contains the implementation of the Dasher view, which is
//! responsible for rendering the Dasher interface.

use std::cell::RefCell;
use std::rc::Rc;

use crate::input::DasherInput;
use crate::model::DasherModel;
use crate::Result;

/// Color representation
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Color {
    /// Create a new color
    pub fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    /// Create a new color from RGB values (alpha = 255)
    pub fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b, a: 255 }
    }

    /// Create a new color from RGBA tuple
    pub fn from_tuple(rgba: (u8, u8, u8, u8)) -> Self {
        Self { r: rgba.0, g: rgba.1, b: rgba.2, a: rgba.3 }
    }

    /// Convert to CSS color string (for web rendering)
    pub fn to_css_string(&self) -> String {
        format!("rgba({}, {}, {}, {})", self.r, self.g, self.b, self.a as f32 / 255.0)
    }
}

/// Color palette with named colors
pub mod color_palette {
    use super::Color;

    pub const BLACK: Color = Color { r: 0, g: 0, b: 0, a: 255 };
    pub const WHITE: Color = Color { r: 255, g: 255, b: 255, a: 255 };
    pub const RED: Color = Color { r: 255, g: 0, b: 0, a: 255 };
    pub const GREEN: Color = Color { r: 0, g: 255, b: 0, a: 255 };
    pub const BLUE: Color = Color { r: 0, g: 0, b: 255, a: 255 };
    pub const YELLOW: Color = Color { r: 255, g: 255, b: 0, a: 255 };
    pub const CYAN: Color = Color { r: 0, g: 255, b: 255, a: 255 };
    pub const MAGENTA: Color = Color { r: 255, g: 0, b: 255, a: 255 };
    pub const GRAY: Color = Color { r: 128, g: 128, b: 128, a: 255 };
    pub const LIGHT_GRAY: Color = Color { r: 192, g: 192, b: 192, a: 255 };
    pub const DARK_GRAY: Color = Color { r: 64, g: 64, b: 64, a: 255 };
    pub const TRANSPARENT: Color = Color { r: 0, g: 0, b: 0, a: 0 };
}

/// Interface for text labels
pub trait Label {
    /// Get the text of the label
    fn get_text(&self) -> &str;

    /// Get the wrap size of the label
    fn get_wrap_size(&self) -> u32;
}

/// Abstract interface for drawing operations, implemented by platform-specific canvases.
pub trait DasherScreen {
    /// Get the width of the screen
    fn get_width(&self) -> i32;

    /// Get the height of the screen
    fn get_height(&self) -> i32;

    /// Create a label for text rendering
    fn make_label(&self, text: &str, wrap_size: u32) -> Box<dyn Label>;

    /// Get the size of a label
    fn text_size(&self, label: &dyn Label, font_size: u32) -> (i32, i32);

    /// Draw a string on the screen
    fn draw_string(&mut self, label: &dyn Label, x: i32, y: i32, font_size: u32, color: Color);

    /// Draw a rectangle on the screen
    fn draw_rectangle(&mut self, x1: i32, y1: i32, x2: i32, y2: i32,
                     fill_color: Color, outline_color: Color, line_width: i32);

    /// Draw a circle on the screen
    fn draw_circle(&mut self, cx: i32, cy: i32, r: i32,
                  fill_color: Color, line_color: Color, line_width: i32);

    /// Draw a line on the screen
    fn draw_line(&mut self, x1: i32, y1: i32, x2: i32, y2: i32, color: Color, line_width: i32);

    /// Signal that a frame is finished - the screen should be updated
    fn display(&mut self);

    /// Returns true if point on screen is not obscured by another window
    fn is_point_visible(&self, x: i32, y: i32) -> bool;
}

/// Orientation of the Dasher view
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Orientation {
    /// Left to right (default)
    LeftToRight,

    /// Right to left
    RightToLeft,

    /// Top to bottom
    TopToBottom,

    /// Bottom to top
    BottomToTop,
}

/// The main Dasher view interface
pub trait DasherView {
    /// Get the dimensions of the view
    fn get_dimensions(&self) -> (i32, i32);

    /// Get the visible region of the view
    fn get_visible_region(&self) -> (i64, i64, i64, i64);

    /// Convert screen coordinates to Dasher coordinates
    fn screen_to_dasher(&self, x: i32, y: i32) -> (i64, i64);

    /// Convert Dasher coordinates to screen coordinates
    fn dasher_to_screen(&self, x: i64, y: i64) -> (i32, i32);

    /// Draw a line in Dasher coordinates
    fn draw_line(&mut self, x1: i64, y1: i64, x2: i64, y2: i64, color: (u8, u8, u8, u8), line_width: i32);

    /// Draw a rectangle in Dasher coordinates
    fn draw_rectangle(&mut self, x1: i64, y1: i64, x2: i64, y2: i64,
                     fill_color: (u8, u8, u8, u8), outline_color: (u8, u8, u8, u8), line_width: i32);

    /// Draw a circle in Dasher coordinates
    fn draw_circle(&mut self, cx: i64, cy: i64, r: i64,
                  fill_color: (u8, u8, u8, u8), line_color: (u8, u8, u8, u8), line_width: i32);

    /// Render the model
    fn render(&mut self, model: &mut DasherModel) -> Result<()>;

    /// Get the input device
    fn get_input_device(&self) -> Option<&dyn DasherInput>;

    /// Set the input device
    fn set_input_device(&mut self, input: Box<dyn DasherInput>);

    /// Get the orientation
    fn get_orientation(&self) -> Orientation;

    /// Set the orientation
    fn set_orientation(&mut self, orientation: Orientation);
}

/// Square Dasher view implementation
pub struct DasherViewSquare {
    /// Screen for rendering
    screen: Box<dyn DasherScreen>,

    /// Orientation of the view
    orientation: Orientation,

    /// Input device
    input_device: Option<Box<dyn DasherInput>>,
}

impl DasherViewSquare {
    /// Create a new square Dasher view
    pub fn new(screen: Box<dyn DasherScreen>) -> Self {
        Self {
            screen,
            orientation: Orientation::LeftToRight,
            input_device: None,
        }
    }
}

impl DasherView for DasherViewSquare {
    fn get_dimensions(&self) -> (i32, i32) {
        (self.screen.get_width(), self.screen.get_height())
    }

    fn get_visible_region(&self) -> (i64, i64, i64, i64) {
        // Return the visible region in Dasher coordinates
        // (min_x, min_y, max_x, max_y)
        match self.orientation {
            Orientation::LeftToRight | Orientation::RightToLeft => {
                (0, 0, DasherModel::MAX_Y, DasherModel::MAX_Y)
            }
            Orientation::TopToBottom | Orientation::BottomToTop => {
                (0, 0, DasherModel::MAX_Y, DasherModel::MAX_Y)
            }
        }
    }

    fn screen_to_dasher(&self, x: i32, y: i32) -> (i64, i64) {
        let (width, height) = self.get_dimensions();

        match self.orientation {
            Orientation::LeftToRight => {
                let dasher_x = (x as f64 / width as f64 * DasherModel::MAX_Y as f64) as i64;
                let dasher_y = (y as f64 / height as f64 * DasherModel::MAX_Y as f64) as i64;
                (dasher_x, dasher_y)
            }
            Orientation::RightToLeft => {
                let dasher_x = ((width - x) as f64 / width as f64 * DasherModel::MAX_Y as f64) as i64;
                let dasher_y = (y as f64 / height as f64 * DasherModel::MAX_Y as f64) as i64;
                (dasher_x, dasher_y)
            }
            Orientation::TopToBottom => {
                let dasher_x = (y as f64 / height as f64 * DasherModel::MAX_Y as f64) as i64;
                let dasher_y = (x as f64 / width as f64 * DasherModel::MAX_Y as f64) as i64;
                (dasher_x, dasher_y)
            }
            Orientation::BottomToTop => {
                let dasher_x = ((height - y) as f64 / height as f64 * DasherModel::MAX_Y as f64) as i64;
                let dasher_y = (x as f64 / width as f64 * DasherModel::MAX_Y as f64) as i64;
                (dasher_x, dasher_y)
            }
        }
    }

    fn dasher_to_screen(&self, x: i64, y: i64) -> (i32, i32) {
        let (width, height) = self.get_dimensions();

        match self.orientation {
            Orientation::LeftToRight => {
                let screen_x = (x as f64 / DasherModel::MAX_Y as f64 * width as f64) as i32;
                let screen_y = (y as f64 / DasherModel::MAX_Y as f64 * height as f64) as i32;
                (screen_x, screen_y)
            }
            Orientation::RightToLeft => {
                let screen_x = width - (x as f64 / DasherModel::MAX_Y as f64 * width as f64) as i32;
                let screen_y = (y as f64 / DasherModel::MAX_Y as f64 * height as f64) as i32;
                (screen_x, screen_y)
            }
            Orientation::TopToBottom => {
                let screen_x = (y as f64 / DasherModel::MAX_Y as f64 * width as f64) as i32;
                let screen_y = (x as f64 / DasherModel::MAX_Y as f64 * height as f64) as i32;
                (screen_x, screen_y)
            }
            Orientation::BottomToTop => {
                let screen_x = (y as f64 / DasherModel::MAX_Y as f64 * width as f64) as i32;
                let screen_y = height - (x as f64 / DasherModel::MAX_Y as f64 * height as f64) as i32;
                (screen_x, screen_y)
            }
        }
    }

    fn draw_line(&mut self, x1: i64, y1: i64, x2: i64, y2: i64, color: (u8, u8, u8, u8), line_width: i32) {
        let (sx1, sy1) = self.dasher_to_screen(x1, y1);
        let (sx2, sy2) = self.dasher_to_screen(x2, y2);

        self.screen.draw_line(sx1, sy1, sx2, sy2, Color::from_tuple(color), line_width);
    }

    fn draw_rectangle(&mut self, x1: i64, y1: i64, x2: i64, y2: i64,
                     fill_color: (u8, u8, u8, u8), outline_color: (u8, u8, u8, u8), line_width: i32) {
        let (sx1, sy1) = self.dasher_to_screen(x1, y1);
        let (sx2, sy2) = self.dasher_to_screen(x2, y2);

        self.screen.draw_rectangle(sx1, sy1, sx2, sy2,
                                  Color::from_tuple(fill_color),
                                  Color::from_tuple(outline_color),
                                  line_width);
    }

    fn draw_circle(&mut self, cx: i64, cy: i64, r: i64,
                  fill_color: (u8, u8, u8, u8), line_color: (u8, u8, u8, u8), line_width: i32) {
        let (sx, sy) = self.dasher_to_screen(cx, cy);

        // Convert radius from Dasher to screen coordinates
        let (width, height) = self.get_dimensions();
        let sr = match self.orientation {
            Orientation::LeftToRight | Orientation::RightToLeft => {
                (r as f64 / DasherModel::MAX_Y as f64 * width as f64) as i32
            }
            Orientation::TopToBottom | Orientation::BottomToTop => {
                (r as f64 / DasherModel::MAX_Y as f64 * height as f64) as i32
            }
        };

        self.screen.draw_circle(sx, sy, sr,
                               Color::from_tuple(fill_color),
                               Color::from_tuple(line_color),
                               line_width);
    }

    fn render(&mut self, model: &mut DasherModel) -> Result<()> {
        // Render the model
        model.render_to_view(&mut *self.screen)
    }

    fn get_input_device(&self) -> Option<&dyn DasherInput> {
        self.input_device.as_deref()
    }

    fn set_input_device(&mut self, input: Box<dyn DasherInput>) {
        self.input_device = Some(input);
    }

    fn get_orientation(&self) -> Orientation {
        self.orientation
    }

    fn set_orientation(&mut self, orientation: Orientation) {
        self.orientation = orientation;
    }
}
