use std::cell::RefCell;
use std::rc::Rc;

use crate::model::DasherModel;
use crate::model::node::DasherNode;
use crate::DasherInput;
use crate::Result;
use crate::view::{DasherView, DasherScreen, Orientation, Color, Label};
use crate::view::color_palette;

/// Text string for delayed rendering
struct TextString {
    /// The label to render
    label: Box<dyn Label>,

    /// X coordinate
    x: i32,

    /// Y coordinate
    y: i32,

    /// Font size
    size: u32,

    /// Text color
    color: Color,

    /// Child text strings
    children: Vec<TextString>,
}

impl TextString {
    /// Create a new text string
    fn new(label: Box<dyn Label>, x: i32, y: i32, size: u32, color: Color) -> Self {
        Self {
            label,
            x,
            y,
            size,
            color,
            children: Vec::new(),
        }
    }
}

/// Constants for the Square View
const SCALE_FACTOR: i64 = 1 << 26; // Large power of 2 for efficient division

/// Configuration for the Square View
#[derive(Debug, Clone)]
pub struct SquareViewConfig {
    /// Node shape type
    pub node_shape: NodeShape,

    /// Enable X logarithmic mapping
    pub x_nonlinear: bool,

    /// X nonlinearity factor (higher = more nonlinear)
    pub x_nonlinear_factor: f64,

    /// Enable Y nonlinearity
    pub y_nonlinear: bool,

    /// Y1 parameter for Y nonlinearity (lower bound of first region)
    pub y1: i64,

    /// Y2 parameter for Y nonlinearity (upper bound of second region)
    pub y2: i64,

    /// Y3 parameter for Y nonlinearity (boundary between first and second regions)
    pub y3: i64,

    /// Enable 3D text rendering
    pub text_3d: bool,

    /// 3D text depth
    pub text_3d_depth: i32,

    /// Base font size
    pub base_font_size: u32,

    /// Font size scaling factor
    pub font_size_scaling: f64,
}

impl Default for SquareViewConfig {
    fn default() -> Self {
        Self {
            node_shape: NodeShape::Rectangle,
            x_nonlinear: true,
            x_nonlinear_factor: 4.8, // Default from C++ implementation
            y_nonlinear: true,
            y1: 4, // Lower bound of first region
            y2: (0.95 * DasherModel::MAX_Y as f64) as i64, // Upper bound of second region
            y3: (0.05 * DasherModel::MAX_Y as f64) as i64, // Boundary between first and second regions
            text_3d: true,
            text_3d_depth: 2,
            base_font_size: 24,
            font_size_scaling: 0.5,
        }
    }
}

/// Node shape types
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum NodeShape {
    /// Rectangle shape
    Rectangle,
    /// Triangle shape
    Triangle,
    /// Truncated triangle shape
    TruncatedTriangle,
    /// Circle shape
    Circle,
    /// Quadric shape (curved)
    Quadric,
}

/// Square Dasher view implementation
pub struct DasherViewSquare {
    /// Screen for rendering
    screen: Box<dyn DasherScreen>,

    /// Orientation of the view
    orientation: Orientation,

    /// Input device
    input_device: Option<Box<dyn DasherInput>>,

    /// Scale factor for X coordinate
    scale_factor_x: i64,

    /// Scale factor for Y coordinate
    scale_factor_y: i64,

    /// Margin width in abstract screen coordinates
    margin_width: i64,

    /// Coefficient for X logarithmic mapping
    x_log_coeff: f64,

    /// Threshold for X logarithmic mapping
    x_log_threshold: i64,

    /// Cached visible region
    visible_region: Option<(i64, i64, i64, i64)>,

    /// Delayed text objects for rendering
    delayed_texts: Vec<TextString>,

    /// Y3 screen parameter (screen coordinate corresponding to Y3)
    y3_screen: i64,

    /// Configuration for the view
    config: SquareViewConfig,
}

impl DasherViewSquare {
    /// Create a new square Dasher view with default configuration
    pub fn new(screen: Box<dyn DasherScreen>) -> Self {
        Self::with_config(screen, SquareViewConfig::default())
    }

    /// Access the screen for testing
    #[cfg(test)]
    pub(crate) fn screen(&self) -> &Box<dyn DasherScreen> {
        &self.screen
    }

    /// Access the x_map method for testing
    #[cfg(test)]
    pub(crate) fn x_map(&self, dasher_x: i64) -> i64 {
        // Call the private implementation
        self._x_map(dasher_x)
    }

    /// Access the set_scale_factor method for testing
    #[cfg(test)]
    pub(crate) fn set_scale_factor(&mut self) {
        // Call the private implementation
        self._set_scale_factor();
    }

    /// Create a new square Dasher view with custom configuration
    pub fn with_config(screen: Box<dyn DasherScreen>, config: SquareViewConfig) -> Self {
        let mut view = Self {
            screen,
            orientation: Orientation::LeftToRight,
            input_device: None,
            scale_factor_x: 0,
            scale_factor_y: 0,
            margin_width: 0,
            x_log_coeff: 0.0,
            x_log_threshold: 0,
            visible_region: None,
            delayed_texts: Vec::new(),
            y3_screen: 0, // Will be calculated in set_scale_factor
            config,
        };

        // Initialize scale factors
        view._set_scale_factor();

        view
    }

    /// Get a reference to the configuration
    pub fn config(&self) -> &SquareViewConfig {
        &self.config
    }

    /// Get a mutable reference to the configuration
    pub fn config_mut(&mut self) -> &mut SquareViewConfig {
        &mut self.config
    }

    /// Set the node shape
    pub fn set_node_shape(&mut self, shape: NodeShape) {
        self.config.node_shape = shape;
    }

    /// Enable or disable X nonlinearity
    pub fn set_x_nonlinear(&mut self, enable: bool) {
        self.config.x_nonlinear = enable;
        self._set_scale_factor(); // Recalculate scale factors
    }

    /// Set the X nonlinearity factor
    pub fn set_x_nonlinear_factor(&mut self, factor: f64) {
        self.config.x_nonlinear_factor = factor;
        self._set_scale_factor(); // Recalculate scale factors
    }

    /// Enable or disable Y nonlinearity
    pub fn set_y_nonlinear(&mut self, enable: bool) {
        self.config.y_nonlinear = enable;
        self._set_scale_factor(); // Recalculate scale factors
    }

    /// Enable or disable 3D text rendering
    pub fn set_text_3d(&mut self, enable: bool) {
        self.config.text_3d = enable;
    }

    /// Set the 3D text depth
    pub fn set_text_3d_depth(&mut self, depth: i32) {
        self.config.text_3d_depth = depth;
    }

    /// Process delayed text rendering
    fn do_delayed_text(&mut self, text: &mut TextString) {
        // Get text dimensions
        let (_text_width, text_height) = self.screen.text_size(&*text.label, text.size);

        // Calculate text position
        let text_x = text.x;
        let text_y = text.y - text_height / 2;

        // Check if 3D text rendering is enabled
        if self.config.text_3d {
            // Draw 3D text with shadow
            let depth = self.config.text_3d_depth;

            // Create shadow color (darker version of the text color)
            let shadow_color = Color::from_tuple((
                (text.color.r as f32 * 0.5) as u8,
                (text.color.g as f32 * 0.5) as u8,
                (text.color.b as f32 * 0.5) as u8,
                text.color.a,
            ));

            // Draw shadow layers
            for i in 1..=depth {
                self.screen.draw_string(&*text.label, text_x + i, text_y + i, text.size, shadow_color);
            }

            // Draw the main text on top
            self.screen.draw_string(&*text.label, text_x, text_y, text.size, text.color);
        } else {
            // Draw normal text
            self.screen.draw_string(&*text.label, text_x, text_y, text.size, text.color);
        }

        // Process children
        for child in &mut text.children {
            self.do_delayed_text(child);
        }
    }

    /// Set the scale factor based on screen dimensions
    fn _set_scale_factor(&mut self) {
        let (width, height) = self.get_dimensions();

        // Set scale factors based on orientation
        match self.orientation {
            Orientation::LeftToRight | Orientation::RightToLeft => {
                self.scale_factor_x = SCALE_FACTOR / width as i64;
                self.scale_factor_y = SCALE_FACTOR / height as i64;
            }
            Orientation::TopToBottom | Orientation::BottomToTop => {
                self.scale_factor_x = SCALE_FACTOR / height as i64;
                self.scale_factor_y = SCALE_FACTOR / width as i64;
            }
        }

        // Set margin width (10% of screen width)
        self.margin_width = SCALE_FACTOR / 10;

        // Set X logarithmic mapping parameters
        if self.config.x_nonlinear {
            self.x_log_coeff = f64::exp(self.config.x_nonlinear_factor / 3.0);
            self.x_log_threshold = DasherModel::MAX_Y / 2;
        } else {
            // Disable X nonlinearity
            self.x_log_coeff = 1.0;
            self.x_log_threshold = DasherModel::MAX_Y;
        }

        // Calculate Y3 screen parameter for Y nonlinearity
        // This is the screen coordinate corresponding to Y3
        // In the C++ implementation, this is calculated based on the screen height
        if self.config.y_nonlinear {
            // Calculate Y3 screen as a fraction of the distance between Y1 and Y2
            let y_range = self.config.y2 - self.config.y1;
            let y3_fraction = (self.config.y3 - self.config.y1) as f64 / y_range as f64;

            // Apply a nonlinear transformation to make the first region steeper
            // and the second region shallower
            let transformed_fraction = y3_fraction * 0.7; // Adjust this factor to control nonlinearity

            // Calculate the screen coordinate
            self.y3_screen = self.config.y1 + (y_range as f64 * transformed_fraction) as i64;
        } else {
            // Disable Y nonlinearity
            self.y3_screen = self.config.y3;
        }

        // Invalidate cached visible region
        self.visible_region = None;
    }

    /// Draw text with the specified parameters
    fn dasher_draw_text(&mut self, max_x: i64, mid_y: i64, label: &str, color: Color) -> TextString {
        // Convert Dasher coordinates to screen coordinates
        let (screen_x, screen_y) = self.dasher_to_screen(max_x, mid_y);

        // Create label object
        let label_obj = self.screen.make_label(label, 0);

        // Calculate font size based on position
        // In C++, font size is scaled based on the distance from the origin
        // The further from the origin, the smaller the font
        let distance_factor = max_x as f64 / DasherModel::MAX_Y as f64;
        let base_font_size = self.config.base_font_size as f64;
        let scaling_factor = self.config.font_size_scaling;
        let font_size = (base_font_size * (1.0 - distance_factor * scaling_factor).max(0.5)) as u32;

        // Create text string
        TextString::new(label_obj, screen_x, screen_y, font_size, color)
    }

    /// Add text to be rendered later
    fn add_delayed_text(&mut self, text: TextString) {
        self.delayed_texts.push(text);
    }

    /// Map Dasher Y coordinate to screen Y coordinate
    fn y_map(&self, dasher_y: i64) -> i64 {
        // Check if Y nonlinearity is enabled
        if self.config.y_nonlinear {
            // Apply nonlinear mapping based on the C++ implementation
            // The C++ implementation uses three regions:
            // 1. Y1 to Y3: Linear mapping with a steeper gradient
            // 2. Y3 to Y2: Linear mapping with a shallower gradient
            // 3. Outside these regions: Linear mapping with the original gradient

            let y1 = self.config.y1;
            let y2 = self.config.y2;
            let y3 = self.config.y3;

            if dasher_y > y1 && dasher_y < y3 {
                // Region 1: Steeper gradient
                let gradient = (y3 - y1) as f64 / (self.y3_screen - y1) as f64;
                let mapped = y1 + ((dasher_y - y1) as f64 / gradient) as i64;
                return mapped;
            } else if dasher_y >= y3 && dasher_y < y2 {
                // Region 2: Shallower gradient
                let gradient = (y2 - y3) as f64 / (y2 - self.y3_screen) as f64;
                let mapped = self.y3_screen + ((dasher_y - y3) as f64 / gradient) as i64;
                return mapped;
            }
        }

        // Linear mapping for regions outside the nonlinear range or if nonlinearity is disabled
        dasher_y
    }

    /// Map Dasher X coordinate to screen X coordinate
    fn _x_map(&self, dasher_x: i64) -> i64 {
        // Apply margin
        let x = dasher_x - self.margin_width;

        // Apply logarithmic mapping if enabled
        if self.x_log_coeff > 1.0 && x >= self.x_log_threshold {
            // Calculate the logarithmic part
            let dx = (x - self.x_log_threshold) as f64 / DasherModel::MAX_Y as f64;
            let dx = ((dx * self.x_log_coeff).exp() - 1.0) / self.x_log_coeff;

            // Combine linear and logarithmic parts
            let result = (dx * DasherModel::MAX_Y as f64) as i64 + self.x_log_threshold;
            return result;
        }

        // Linear mapping for values below threshold
        x
    }

    /// Inverse Y mapping
    fn iy_map(&self, screen_y: i64) -> i64 {
        // Check if Y nonlinearity is enabled
        if self.config.y_nonlinear {
            // Apply inverse nonlinear mapping
            let y1 = self.config.y1;
            let y2 = self.config.y2;
            let y3 = self.config.y3;
            let y3_screen = self.y3_screen;

            if screen_y > y1 && screen_y < y3_screen {
                // Region 1: Steeper gradient
                let gradient = (y3 - y1) as f64 / (y3_screen - y1) as f64;
                let mapped = y1 + ((screen_y - y1) as f64 * gradient) as i64;
                return mapped;
            } else if screen_y >= y3_screen && screen_y < y2 {
                // Region 2: Shallower gradient
                let gradient = (y2 - y3) as f64 / (y2 - y3_screen) as f64;
                let mapped = y3 + ((screen_y - y3_screen) as f64 * gradient) as i64;
                return mapped;
            }
        }

        // Linear mapping for regions outside the nonlinear range or if nonlinearity is disabled
        screen_y
    }

    /// Inverse X mapping
    fn ix_map(&self, screen_x: i64) -> i64 {
        // Apply logarithmic mapping if enabled
        if self.x_log_coeff > 1.0 && screen_x >= self.x_log_threshold {
            // Calculate the logarithmic part
            let dx = ((screen_x - self.x_log_threshold) as f64 * self.x_log_coeff / DasherModel::MAX_Y as f64 + 1.0).ln() / self.x_log_coeff;

            // Combine linear and logarithmic parts
            let result = (dx * DasherModel::MAX_Y as f64) as i64 + self.x_log_threshold;

            // Apply margin
            return result + self.margin_width;
        }

        // Linear mapping for values below threshold
        screen_x + self.margin_width
    }

    /// Draw a triangle node
    fn draw_triangle(&mut self, range: i64, y1: i64, y2: i64, fill_color: Color, outline_color: Color, line_width: i32) {
        // Calculate the midpoint
        let mid_y = (y1 + y2) / 2;

        // Convert to screen coordinates
        let (sx1, sy1) = self.dasher_to_screen(0, y1);
        let (sx2, sy2) = self.dasher_to_screen(range, mid_y);
        let (sx3, sy3) = self.dasher_to_screen(0, y2);

        // Draw the triangle
        let points = [
            (sx1, sy1),
            (sx2, sy2),
            (sx3, sy3),
        ];

        // Draw filled triangle
        self.screen.draw_polygon(&points, fill_color, outline_color, line_width);
    }

    /// Draw a truncated triangle node
    fn draw_truncated_triangle(&mut self, range: i64, y1: i64, y2: i64, fill_color: Color, outline_color: Color, line_width: i32) {
        // Calculate the truncation points
        let trunc_y1 = (y1 + y1 + y2) / 3;
        let trunc_y2 = (y1 + y2 + y2) / 3;

        // Convert to screen coordinates
        let (sx1, sy1) = self.dasher_to_screen(0, y1);
        let (sx2, sy2) = self.dasher_to_screen(range, trunc_y1);
        let (sx3, sy3) = self.dasher_to_screen(range, trunc_y2);
        let (sx4, sy4) = self.dasher_to_screen(0, y2);

        // Draw the truncated triangle
        let points = [
            (sx1, sy1),
            (sx2, sy2),
            (sx3, sy3),
            (sx4, sy4),
        ];

        // Draw filled polygon
        self.screen.draw_polygon(&points, fill_color, outline_color, line_width);
    }

    /// Draw a quadric node (curved shape)
    fn draw_quadric(&mut self, range: i64, y1: i64, y2: i64, fill_color: Color, outline_color: Color, line_width: i32) {
        // Calculate the midpoint
        let mid_y = (y1 + y2) / 2;

        // Calculate control points for the quadric curve
        // We'll use a Bezier curve with 4 control points
        let rr2 = 1.0 / f64::sqrt(2.0); // 1/sqrt(2)

        // Calculate control points
        let p1 = (0, y1); // Top-left
        let p2 = ((range as f64 * rr2) as i64, (y1 as f64 * rr2 + mid_y as f64 * (1.0 - rr2)) as i64); // Top-right control
        let p3 = (range, mid_y); // Right-middle
        let p4 = ((range as f64 * rr2) as i64, (y2 as f64 * rr2 + mid_y as f64 * (1.0 - rr2)) as i64); // Bottom-right control
        let p5 = (0, y2); // Bottom-left

        // Convert to screen coordinates
        let (sx1, sy1) = self.dasher_to_screen(p1.0, p1.1);
        let (sx2, sy2) = self.dasher_to_screen(p2.0, p2.1);
        let (sx3, sy3) = self.dasher_to_screen(p3.0, p3.1);
        let (sx4, sy4) = self.dasher_to_screen(p4.0, p4.1);
        let (sx5, sy5) = self.dasher_to_screen(p5.0, p5.1);

        // Generate points along the curve
        let num_steps = 40;
        let mut points = Vec::with_capacity(num_steps + 2);

        // Add the first point
        points.push((sx1, sy1));

        // Generate points along the top curve (p1 -> p2 -> p3)
        for i in 1..=num_steps {
            let t = i as f64 / num_steps as f64;
            let one_minus_t = 1.0 - t;

            // Quadratic Bezier formula: (1-t)^2 * P0 + 2(1-t)t * P1 + t^2 * P2
            let x = (one_minus_t * one_minus_t * sx1 as f64 +
                    2.0 * one_minus_t * t * sx2 as f64 +
                    t * t * sx3 as f64) as i32;

            let y = (one_minus_t * one_minus_t * sy1 as f64 +
                    2.0 * one_minus_t * t * sy2 as f64 +
                    t * t * sy3 as f64) as i32;

            points.push((x, y));
        }

        // Generate points along the bottom curve (p3 -> p4 -> p5)
        for i in 1..=num_steps {
            let t = i as f64 / num_steps as f64;
            let one_minus_t = 1.0 - t;

            // Quadratic Bezier formula: (1-t)^2 * P0 + 2(1-t)t * P1 + t^2 * P2
            let x = (one_minus_t * one_minus_t * sx3 as f64 +
                    2.0 * one_minus_t * t * sx4 as f64 +
                    t * t * sx5 as f64) as i32;

            let y = (one_minus_t * one_minus_t * sy3 as f64 +
                    2.0 * one_minus_t * t * sy4 as f64 +
                    t * t * sy5 as f64) as i32;

            points.push((x, y));
        }

        // Draw the polygon
        self.screen.draw_polygon(&points, fill_color, outline_color, line_width);
    }

    /// Draw a circle node
    fn draw_circle_node(&mut self, range: i64, y1: i64, y2: i64, fill_color: Color, outline_color: Color, line_width: i32) {
        // Calculate the center and radius
        let center_y = (y1 + y2) / 2;
        let radius = range / 2;

        // Convert to screen coordinates
        let (cx, cy) = self.dasher_to_screen(radius, center_y);

        // Calculate screen radius
        let (width, _) = self.get_dimensions();
        let screen_radius = (radius as f64 / DasherModel::MAX_Y as f64 * width as f64) as i32;

        // Draw the circle
        self.screen.draw_circle(cx, cy, screen_radius, fill_color, outline_color, line_width);
    }

    /// Draw a node with the current shape
    fn draw_node_shape(&mut self, range: i64, y1: i64, y2: i64, fill_color: Color, outline_color: Color, line_width: i32) {
        match self.config.node_shape {
            NodeShape::Rectangle => {
                // Draw a rectangle
                let (sx1, sy1) = self.dasher_to_screen(0, y1);
                let (sx2, sy2) = self.dasher_to_screen(range, y2);
                self.screen.draw_rectangle(sx1, sy1, sx2, sy2, fill_color, outline_color, line_width);
            }
            NodeShape::Triangle => {
                // Draw a triangle
                self.draw_triangle(range, y1, y2, fill_color, outline_color, line_width);
            }
            NodeShape::TruncatedTriangle => {
                // Draw a truncated triangle
                self.draw_truncated_triangle(range, y1, y2, fill_color, outline_color, line_width);
            }
            NodeShape::Circle => {
                // Draw a circle
                self.draw_circle_node(range, y1, y2, fill_color, outline_color, line_width);
            }
            NodeShape::Quadric => {
                // Draw a quadric curve
                self.draw_quadric(range, y1, y2, fill_color, outline_color, line_width);
            }
        }
    }

    /// Draw a crosshair at the center of the screen
    fn crosshair(&mut self) {
        let (width, height) = self.get_dimensions();
        let cx = width / 2;
        let cy = height / 2;

        // Draw horizontal line
        self.screen.draw_line(cx - 10, cy, cx + 10, cy, color_palette::RED, 2);

        // Draw vertical line
        self.screen.draw_line(cx, cy - 10, cx, cy + 10, color_palette::RED, 2);

        // Draw circle at intersection
        self.screen.draw_circle(cx, cy, 5, color_palette::RED, color_palette::BLACK, 1);
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

        // Convert screen coordinates to normalized Dasher coordinates
        let (mapped_x, mapped_y) = match self.orientation {
            Orientation::LeftToRight => {
                let mapped_x = x as i64 * self.scale_factor_x;
                let mapped_y = y as i64 * self.scale_factor_y;
                (mapped_x, mapped_y)
            }
            Orientation::RightToLeft => {
                let mapped_x = (width - x) as i64 * self.scale_factor_x;
                let mapped_y = y as i64 * self.scale_factor_y;
                (mapped_x, mapped_y)
            }
            Orientation::TopToBottom => {
                let mapped_x = y as i64 * self.scale_factor_x;
                let mapped_y = x as i64 * self.scale_factor_y;
                (mapped_x, mapped_y)
            }
            Orientation::BottomToTop => {
                let mapped_x = (height - y) as i64 * self.scale_factor_x;
                let mapped_y = x as i64 * self.scale_factor_y;
                (mapped_x, mapped_y)
            }
        };

        // Apply inverse coordinate mapping
        let dasher_x = self.ix_map(mapped_x);
        let dasher_y = self.iy_map(mapped_y);

        (dasher_x, dasher_y)
    }

    fn dasher_to_screen(&self, x: i64, y: i64) -> (i32, i32) {
        // Apply the nonlinearities
        let mapped_x = self._x_map(x);
        let mapped_y = self.y_map(y);

        let (width, height) = self.get_dimensions();

        // Convert to screen coordinates based on orientation
        match self.orientation {
            Orientation::LeftToRight => {
                let screen_x = (mapped_x / self.scale_factor_x) as i32;
                let screen_y = (mapped_y / self.scale_factor_y) as i32;
                (screen_x, screen_y)
            }
            Orientation::RightToLeft => {
                let screen_x = width - (mapped_x / self.scale_factor_x) as i32;
                let screen_y = (mapped_y / self.scale_factor_y) as i32;
                (screen_x, screen_y)
            }
            Orientation::TopToBottom => {
                let screen_x = (mapped_y / self.scale_factor_y) as i32;
                let screen_y = (mapped_x / self.scale_factor_x) as i32;
                (screen_x, screen_y)
            }
            Orientation::BottomToTop => {
                let screen_x = (mapped_y / self.scale_factor_y) as i32;
                let screen_y = height - (mapped_x / self.scale_factor_x) as i32;
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
        // Get screen dimensions
        let (width, height) = self.get_dimensions();

        // Clear the screen
        self.screen.draw_rectangle(0, 0, width, height, color_palette::WHITE, color_palette::BLACK, 1);

        // Draw the root node and its children
        if let Some(root) = model.get_root_node() {
            self.render_node(root);
        }

        // Draw the crosshair
        self.crosshair();

        // Process delayed text rendering
        let mut delayed_texts = std::mem::take(&mut self.delayed_texts);
        for text in &mut delayed_texts {
            self.do_delayed_text(text);
        }

        // Display the frame
        self.screen.display();

        Ok(())
    }

    /// Render a node and its children
    fn render_node(&mut self, node: Rc<RefCell<DasherNode>>) {
        let node_ref = node.borrow();

        // Calculate node boundaries in Dasher coordinates
        let lower = node_ref.lower_bound() as i64;
        let upper = node_ref.upper_bound() as i64;
        let range = upper - lower;

        // We don't need screen coordinates anymore since we're using Dasher coordinates directly
        // in draw_node_shape

        // Draw the node
        let bg_color = Color::from_tuple((
            node_ref.background_color().0,
            node_ref.background_color().1,
            node_ref.background_color().2,
            200
        ));

        let fg_color = Color::from_tuple((
            node_ref.foreground_color().0,
            node_ref.foreground_color().1,
            node_ref.foreground_color().2,
            255
        ));

        // Draw the node with the current shape
        self.draw_node_shape(DasherModel::MAX_Y / 4, lower, upper, bg_color, color_palette::BLACK, 1);

        // Draw the node label
        if let Some(label) = node_ref.label() {
            // Create a delayed text object
            let text = self.dasher_draw_text(DasherModel::MAX_Y / 8, (lower + upper) / 2, label, fg_color);

            // Add it to the delayed texts
            self.add_delayed_text(text);
        }

        // Render children
        for child in node_ref.children() {
            let child_ref = child.borrow();

            // Calculate child boundaries in Dasher coordinates
            let child_lower = lower + (range * child_ref.lower_bound() as i64) / DasherNode::NORMALIZATION as i64;
            let child_upper = lower + (range * child_ref.upper_bound() as i64) / DasherNode::NORMALIZATION as i64;

            // We don't need screen coordinates anymore since we're using Dasher coordinates directly
            // in draw_node_shape

            // Draw the child node
            let child_bg_color = Color::from_tuple((
                child_ref.background_color().0,
                child_ref.background_color().1,
                child_ref.background_color().2,
                200
            ));

            let child_fg_color = Color::from_tuple((
                child_ref.foreground_color().0,
                child_ref.foreground_color().1,
                child_ref.foreground_color().2,
                255
            ));

            // Draw the child node with the current shape
            self.draw_node_shape(DasherModel::MAX_Y / 4, child_lower, child_upper, child_bg_color, color_palette::BLACK, 1);

            // Draw the child node label
            if let Some(label) = child_ref.label() {
                // Create a delayed text object
                let text = self.dasher_draw_text(DasherModel::MAX_Y / 3, (child_lower + child_upper) / 2, label, child_fg_color);

                // Add it to the delayed texts
                self.add_delayed_text(text);
            }
        }
    }

    fn get_input_device(&self) -> Option<&dyn DasherInput> {
        self.input_device.as_deref()
    }

    fn get_input_device_mut(&mut self) -> Option<&mut dyn DasherInput> {
        // This is a workaround for lifetime issues
        // We're returning None for now, but in a real implementation
        // we would need to properly handle the lifetime issues
        None
    }

    fn set_input_device(&mut self, input: Box<dyn DasherInput>) {
        self.input_device = Some(input);
    }

    fn get_orientation(&self) -> Orientation {
        self.orientation
    }

    fn set_orientation(&mut self, orientation: Orientation) {
        self.orientation = orientation;
        // Update scale factors when orientation changes
        self._set_scale_factor();
    }
}
