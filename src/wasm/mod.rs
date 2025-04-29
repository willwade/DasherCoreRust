//! # WebAssembly Module
//!
//! This module contains the WebAssembly bindings for the Dasher core.
//! It provides JavaScript/TypeScript integration for web and Electron applications.

#![cfg(feature = "wasm")]

use wasm_bindgen::prelude::*;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement};
use std::collections::HashMap;

use crate::api::DasherInterface;
use crate::view::{Color, DasherScreen, Label};
use crate::settings::Settings;

/// WebAssembly bindings for the Dasher interface
#[wasm_bindgen]
pub struct DasherInterfaceWasm {
    interface: DasherInterface,
    screen: Option<WebDasherScreen>,
}

#[wasm_bindgen]
impl DasherInterfaceWasm {
    /// Create a new Dasher interface
    #[wasm_bindgen(constructor)]
    pub fn new(settings_json: &str) -> Result<DasherInterfaceWasm, JsValue> {
        // Parse settings from JSON
        let settings = match serde_json::from_str(settings_json) {
            Ok(settings) => settings,
            Err(e) => return Err(JsValue::from_str(&format!("Invalid settings JSON: {}", e))),
        };

        let interface = DasherInterface::new(settings);

        Ok(DasherInterfaceWasm {
            interface,
            screen: None,
        })
    }

    /// Set the canvas for rendering
    #[wasm_bindgen]
    pub fn set_canvas(&mut self, canvas_id: &str) -> Result<(), JsValue> {
        let screen = WebDasherScreen::new(canvas_id)?;
        self.interface.change_screen(Box::new(screen.clone()))
            .map_err(|e| JsValue::from_str(&format!("Failed to change screen: {}", e)))?;
        self.screen = Some(screen);
        Ok(())
    }

    /// Process a new frame
    #[wasm_bindgen]
    pub fn new_frame(&mut self, time_ms: u64) -> bool {
        self.interface.new_frame(time_ms)
    }

    /// Set the mouse position
    #[wasm_bindgen]
    pub fn set_mouse_position(&mut self, x: i32, y: i32) -> Result<(), JsValue> {
        self.interface.set_mouse_position(x, y)
            .map_err(|e| JsValue::from_str(&format!("Failed to set mouse position: {}", e)))
    }

    /// Set the node shape for the Square View
    #[wasm_bindgen]
    pub fn set_node_shape(&mut self, shape: &str) -> Result<(), JsValue> {
        use crate::view::NodeShape;

        let node_shape = match shape {
            "rectangle" => NodeShape::Rectangle,
            "triangle" => NodeShape::Triangle,
            "truncated-triangle" => NodeShape::TruncatedTriangle,
            "circle" => NodeShape::Circle,
            "quadric" => NodeShape::Quadric,
            _ => return Err(JsValue::from_str(&format!("Invalid node shape: {}", shape))),
        };

        self.interface.set_node_shape(node_shape)
            .map_err(|e| JsValue::from_str(&format!("Failed to set node shape: {}", e)))
    }

    /// Enable or disable X nonlinearity
    #[wasm_bindgen]
    pub fn set_x_nonlinear(&mut self, enable: bool) -> Result<(), JsValue> {
        self.interface.set_x_nonlinear(enable)
            .map_err(|e| JsValue::from_str(&format!("Failed to set X nonlinearity: {}", e)))
    }

    /// Enable or disable Y nonlinearity
    #[wasm_bindgen]
    pub fn set_y_nonlinear(&mut self, enable: bool) -> Result<(), JsValue> {
        self.interface.set_y_nonlinear(enable)
            .map_err(|e| JsValue::from_str(&format!("Failed to set Y nonlinearity: {}", e)))
    }

    /// Enable or disable 3D text
    #[wasm_bindgen]
    pub fn set_text_3d(&mut self, enable: bool) -> Result<(), JsValue> {
        self.interface.set_text_3d(enable)
            .map_err(|e| JsValue::from_str(&format!("Failed to set 3D text: {}", e)))
    }

    /// Reset the Dasher interface
    #[wasm_bindgen]
    pub fn reset(&mut self) -> Result<(), JsValue> {
        self.interface.reset()
            .map_err(|e| JsValue::from_str(&format!("Failed to reset: {}", e)))
    }

    /// Handle backspace
    #[wasm_bindgen]
    pub fn backspace(&mut self) -> Result<(), JsValue> {
        self.interface.backspace()
            .map_err(|e| JsValue::from_str(&format!("Failed to handle backspace: {}", e)))
    }
}



/// WebAssembly implementation of DasherScreen using HTML Canvas
#[derive(Clone)]
pub struct WebDasherScreen {
    canvas: HtmlCanvasElement,
    context: CanvasRenderingContext2d,
    width: i32,
    height: i32,
    labels: HashMap<String, WebLabel>,
}

impl WebDasherScreen {
    /// Create a new WebDasherScreen
    pub fn new(canvas_id: &str) -> Result<Self, JsValue> {
        // Get canvas element
        let window = web_sys::window().ok_or_else(|| JsValue::from_str("No window found"))?;
        let document = window.document().ok_or_else(|| JsValue::from_str("No document found"))?;

        let canvas = document
            .get_element_by_id(canvas_id)
            .ok_or_else(|| JsValue::from_str(&format!("Canvas with id '{}' not found", canvas_id)))?
            .dyn_into::<HtmlCanvasElement>()?;

        // Get 2D context
        let context = canvas
            .get_context("2d")?
            .ok_or_else(|| JsValue::from_str("Failed to get 2D context"))?
            .dyn_into::<CanvasRenderingContext2d>()?;

        let width = canvas.width() as i32;
        let height = canvas.height() as i32;

        Ok(WebDasherScreen {
            canvas,
            context,
            width,
            height,
            labels: HashMap::new(),
        })
    }
}

impl DasherScreen for WebDasherScreen {
    fn get_width(&self) -> i32 {
        self.width
    }

    fn get_height(&self) -> i32 {
        self.height
    }

    fn make_label(&self, text: &str, wrap_size: u32) -> Box<dyn Label> {
        Box::new(WebLabel::new(text, wrap_size))
    }

    fn text_size(&self, label: &dyn Label, _font_size: u32) -> (i32, i32) {
        // TODO: Implement proper text measurement
        let text = label.get_text();
        (text.len() as i32 * 8, 16)
    }

    fn draw_string(&mut self, label: &dyn Label, x: i32, y: i32, font_size: u32, color: Color) {
        // TODO: Implement proper text rendering
        let text = label.get_text();
        self.context.set_fill_style(&JsValue::from_str(&color.to_css_string()));
        self.context.set_font(&format!("{}px sans-serif", font_size));
        let _ = self.context.fill_text(text, x as f64, y as f64);
    }

    fn draw_rectangle(&mut self, x1: i32, y1: i32, x2: i32, y2: i32,
                     fill_color: Color, outline_color: Color, line_width: i32) {
        // Use Canvas API to draw rectangle
        if fill_color != crate::view::color_palette::TRANSPARENT {
            self.context.set_fill_style(&JsValue::from_str(&fill_color.to_css_string()));
            self.context.fill_rect(x1 as f64, y1 as f64, (x2 - x1) as f64, (y2 - y1) as f64);
        }

        if outline_color != crate::view::color_palette::TRANSPARENT && line_width > 0 {
            self.context.set_stroke_style(&JsValue::from_str(&outline_color.to_css_string()));
            self.context.set_line_width(line_width as f64);
            self.context.stroke_rect(x1 as f64, y1 as f64, (x2 - x1) as f64, (y2 - y1) as f64);
        }
    }

    fn draw_circle(&mut self, cx: i32, cy: i32, r: i32,
                  fill_color: Color, line_color: Color, line_width: i32) {
        // Use Canvas API to draw circle
        self.context.begin_path();
        let _ = self.context.arc(cx as f64, cy as f64, r as f64, 0.0, 2.0 * std::f64::consts::PI);

        if fill_color != crate::view::color_palette::TRANSPARENT {
            self.context.set_fill_style(&JsValue::from_str(&fill_color.to_css_string()));
            self.context.fill();
        }

        if line_color != crate::view::color_palette::TRANSPARENT && line_width > 0 {
            self.context.set_stroke_style(&JsValue::from_str(&line_color.to_css_string()));
            self.context.set_line_width(line_width as f64);
            self.context.stroke();
        }
    }

    fn draw_line(&mut self, x1: i32, y1: i32, x2: i32, y2: i32, color: Color, line_width: i32) {
        // Use Canvas API to draw line
        self.context.begin_path();
        self.context.move_to(x1 as f64, y1 as f64);
        self.context.line_to(x2 as f64, y2 as f64);
        self.context.set_stroke_style(&JsValue::from_str(&color.to_css_string()));
        self.context.set_line_width(line_width as f64);
        self.context.stroke();
    }

    fn display(&mut self) {
        // Nothing to do - Canvas updates immediately
    }

    fn is_point_visible(&self, _x: i32, _y: i32) -> bool {
        // In a web context, we assume the point is always visible
        true
    }
}

/// WebAssembly implementation of Label
#[derive(Clone)]
pub struct WebLabel {
    text: String,
    wrap_size: u32,
}

impl WebLabel {
    /// Create a new WebLabel
    pub fn new(text: &str, wrap_size: u32) -> Self {
        Self {
            text: text.to_string(),
            wrap_size,
        }
    }
}

impl Label for WebLabel {
    fn get_text(&self) -> &str {
        &self.text
    }

    fn get_wrap_size(&self) -> u32 {
        self.wrap_size
    }
}
