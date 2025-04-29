//! WASM API for DasherCore: C++-style persistent model and context.
//! Exposes functions for frontend to interact with the model, similar to C++ API.

use wasm_bindgen::prelude::*;
use wasm_bindgen::JsValue;
use web_sys::{console, HtmlCanvasElement, CanvasRenderingContext2d};
use js_sys::{Function, Object};

use crate::api::DasherInterface;
use crate::settings::Settings;
use crate::view::{NodeShape, DasherScreen};
use crate::input::MouseInput;
use crate::model::DasherModel;
use crate::alphabet::Alphabet;

// Create a global static to hold the Dasher interface
static mut DASHER_INTERFACE: Option<DasherInterface> = None;

/// Initialize the Dasher interface with default settings
#[wasm_bindgen]
pub fn init_dasher() -> Result<(), JsValue> {
    console::log_1(&JsValue::from_str("[WASM] Initializing Dasher"));

    // Create default settings
    let settings = Settings::new();

    // Create the interface
    let interface = DasherInterface::new(settings);

    // Store the interface in the global static
    unsafe {
        DASHER_INTERFACE = Some(interface);
    }

    Ok(())
}

/// Set the canvas for rendering
#[wasm_bindgen]
pub fn set_canvas(canvas_id: &str) -> Result<(), JsValue> {
    console::log_1(&JsValue::from_str(&format!("[WASM] Setting canvas: {}", canvas_id)));

    // Get the canvas element
    let window = web_sys::window().ok_or_else(|| JsValue::from_str("No window found"))?;
    let document = window.document().ok_or_else(|| JsValue::from_str("No document found"))?;
    let canvas = document.get_element_by_id(canvas_id)
        .ok_or_else(|| JsValue::from_str(&format!("No canvas found with id: {}", canvas_id)))?;
    let canvas: HtmlCanvasElement = canvas.dyn_into::<HtmlCanvasElement>()?;

    // Get the 2D context
    let context = canvas.get_context("2d")?
        .ok_or_else(|| JsValue::from_str("Failed to get 2D context"))?
        .dyn_into::<CanvasRenderingContext2d>()?;

    // Get the dimensions
    let width = canvas.width() as i32;
    let height = canvas.height() as i32;

    // Create a WebScreen
    let screen = WebScreen {
        canvas: canvas.clone(),
        context,
        width,
        height,
    };

    // Create a mouse input
    let mut mouse_input = MouseInput::new();
    mouse_input.activate();

    // Set the screen and input in the interface
    unsafe {
        if let Some(interface) = &mut DASHER_INTERFACE {
            interface.set_screen(Box::new(screen))?;
            interface.set_input(Box::new(mouse_input))?;
        } else {
            return Err(JsValue::from_str("Dasher interface not initialized"));
        }
    }

    // Set up mouse events
    let mouse_move_callback = Closure::wrap(Box::new(move |event: web_sys::MouseEvent| {
        let x = event.client_x();
        let y = event.client_y();

        // Get the canvas position
        let rect = canvas.get_bounding_client_rect();
        let canvas_x = x - rect.left() as i32;
        let canvas_y = y - rect.top() as i32;

        // Update the mouse position
        unsafe {
            if let Some(interface) = &mut DASHER_INTERFACE {
                let _ = interface.set_mouse_position(canvas_x, canvas_y);
            }
        }
    }) as Box<dyn FnMut(_)>);

    // Add the mouse move event listener
    canvas.add_event_listener_with_callback("mousemove", mouse_move_callback.as_ref().unchecked_ref())?;

    // Prevent the callback from being garbage collected
    mouse_move_callback.forget();

    Ok(())
}

/// Process a new frame
#[wasm_bindgen]
pub fn new_frame(timestamp: f64) -> Result<(), JsValue> {
    unsafe {
        if let Some(interface) = &mut DASHER_INTERFACE {
            match interface.new_frame(timestamp as u64) {
                Ok(_) => Ok(()),
                Err(e) => Err(JsValue::from_str(&e.to_string())),
            }
        } else {
            Err(JsValue::from_str("Dasher interface not initialized"))
        }
    }
}

/// Start the interface
#[wasm_bindgen]
pub fn start() -> Result<(), JsValue> {
    unsafe {
        if let Some(interface) = &mut DASHER_INTERFACE {
            match interface.start() {
                Ok(_) => Ok(()),
                Err(e) => Err(JsValue::from_str(&e.to_string())),
            }
        } else {
            Err(JsValue::from_str("Dasher interface not initialized"))
        }
    }
}

/// Stop the interface
#[wasm_bindgen]
pub fn stop() -> Result<(), JsValue> {
    unsafe {
        if let Some(interface) = &mut DASHER_INTERFACE {
            match interface.stop() {
                Ok(_) => Ok(()),
                Err(e) => Err(JsValue::from_str(&e.to_string())),
            }
        } else {
            Err(JsValue::from_str("Dasher interface not initialized"))
        }
    }
}

/// Pause the interface
#[wasm_bindgen]
pub fn pause() -> Result<(), JsValue> {
    unsafe {
        if let Some(interface) = &mut DASHER_INTERFACE {
            match interface.pause() {
                Ok(_) => Ok(()),
                Err(e) => Err(JsValue::from_str(&e.to_string())),
            }
        } else {
            Err(JsValue::from_str("Dasher interface not initialized"))
        }
    }
}

/// Resume the interface
#[wasm_bindgen]
pub fn resume() -> Result<(), JsValue> {
    unsafe {
        if let Some(interface) = &mut DASHER_INTERFACE {
            match interface.resume() {
                Ok(_) => Ok(()),
                Err(e) => Err(JsValue::from_str(&e.to_string())),
            }
        } else {
            Err(JsValue::from_str("Dasher interface not initialized"))
        }
    }
}

/// Reset the interface
#[wasm_bindgen]
pub fn reset() -> Result<(), JsValue> {
    unsafe {
        if let Some(interface) = &mut DASHER_INTERFACE {
            match interface.reset() {
                Ok(_) => Ok(()),
                Err(e) => Err(JsValue::from_str(&e.to_string())),
            }
        } else {
            Err(JsValue::from_str("Dasher interface not initialized"))
        }
    }
}

/// Handle backspace
#[wasm_bindgen]
pub fn backspace() -> Result<(), JsValue> {
    unsafe {
        if let Some(interface) = &mut DASHER_INTERFACE {
            match interface.backspace() {
                Ok(_) => Ok(()),
                Err(e) => Err(JsValue::from_str(&e.to_string())),
            }
        } else {
            Err(JsValue::from_str("Dasher interface not initialized"))
        }
    }
}

/// Get the output text
#[wasm_bindgen]
pub fn get_output_text() -> String {
    unsafe {
        if let Some(interface) = &DASHER_INTERFACE {
            interface.get_output_text().to_string()
        } else {
            "".to_string()
        }
    }
}

/// Set the node shape
#[wasm_bindgen]
pub fn set_node_shape(shape: &str) -> Result<(), JsValue> {
    let node_shape = match shape {
        "rectangle" => NodeShape::Rectangle,
        "triangle" => NodeShape::Triangle,
        "truncated-triangle" => NodeShape::TruncatedTriangle,
        "circle" => NodeShape::Circle,
        "quadric" => NodeShape::Quadric,
        _ => NodeShape::Rectangle,
    };

    unsafe {
        if let Some(interface) = &mut DASHER_INTERFACE {
            match interface.set_node_shape(node_shape) {
                Ok(_) => Ok(()),
                Err(e) => Err(JsValue::from_str(&e.to_string())),
            }
        } else {
            Err(JsValue::from_str("Dasher interface not initialized"))
        }
    }
}

/// Enable or disable X nonlinearity
#[wasm_bindgen]
pub fn set_x_nonlinear(enable: bool) -> Result<(), JsValue> {
    unsafe {
        if let Some(interface) = &mut DASHER_INTERFACE {
            match interface.set_x_nonlinear(enable) {
                Ok(_) => Ok(()),
                Err(e) => Err(JsValue::from_str(&e.to_string())),
            }
        } else {
            Err(JsValue::from_str("Dasher interface not initialized"))
        }
    }
}

/// Enable or disable Y nonlinearity
#[wasm_bindgen]
pub fn set_y_nonlinear(enable: bool) -> Result<(), JsValue> {
    unsafe {
        if let Some(interface) = &mut DASHER_INTERFACE {
            match interface.set_y_nonlinear(enable) {
                Ok(_) => Ok(()),
                Err(e) => Err(JsValue::from_str(&e.to_string())),
            }
        } else {
            Err(JsValue::from_str("Dasher interface not initialized"))
        }
    }
}

/// Enable or disable 3D text
#[wasm_bindgen]
pub fn set_text_3d(enable: bool) -> Result<(), JsValue> {
    unsafe {
        if let Some(interface) = &mut DASHER_INTERFACE {
            match interface.set_text_3d(enable) {
                Ok(_) => Ok(()),
                Err(e) => Err(JsValue::from_str(&e.to_string())),
            }
        } else {
            Err(JsValue::from_str("Dasher interface not initialized"))
        }
    }
}

/// Enable or disable the flowing interface
#[wasm_bindgen]
pub fn set_flowing_interface(enable: bool) -> Result<(), JsValue> {
    unsafe {
        if let Some(interface) = &mut DASHER_INTERFACE {
            match interface.set_flowing_interface(enable) {
                Ok(_) => Ok(()),
                Err(e) => Err(JsValue::from_str(&e.to_string())),
            }
        } else {
            Err(JsValue::from_str("Dasher interface not initialized"))
        }
    }
}

/// Set the flowing interface speed
#[wasm_bindgen]
pub fn set_flowing_speed(speed: f64) -> Result<(), JsValue> {
    unsafe {
        if let Some(interface) = &mut DASHER_INTERFACE {
            match interface.set_flowing_speed(speed) {
                Ok(_) => Ok(()),
                Err(e) => Err(JsValue::from_str(&e.to_string())),
            }
        } else {
            Err(JsValue::from_str("Dasher interface not initialized"))
        }
    }
}

/// Enable or disable PPM (Prediction by Partial Match)
#[wasm_bindgen]
pub fn set_ppm(enable: bool) -> Result<(), JsValue> {
    unsafe {
        if let Some(interface) = &mut DASHER_INTERFACE {
            match interface.set_ppm(enable) {
                Ok(_) => Ok(()),
                Err(e) => Err(JsValue::from_str(&e.to_string())),
            }
        } else {
            Err(JsValue::from_str("Dasher interface not initialized"))
        }
    }
}

/// Enable or disable drawing the crosshair
#[wasm_bindgen]
pub fn set_draw_crosshair(enable: bool) -> Result<(), JsValue> {
    unsafe {
        if let Some(interface) = &mut DASHER_INTERFACE {
            match interface.set_draw_crosshair(enable) {
                Ok(_) => Ok(()),
                Err(e) => Err(JsValue::from_str(&e.to_string())),
            }
        } else {
            Err(JsValue::from_str("Dasher interface not initialized"))
        }
    }
}

/// Enable or disable drawing the cursor
#[wasm_bindgen]
pub fn set_draw_cursor(enable: bool) -> Result<(), JsValue> {
    unsafe {
        if let Some(interface) = &mut DASHER_INTERFACE {
            match interface.set_draw_cursor(enable) {
                Ok(_) => Ok(()),
                Err(e) => Err(JsValue::from_str(&e.to_string())),
            }
        } else {
            Err(JsValue::from_str("Dasher interface not initialized"))
        }
    }
}

/// Enable or disable drawing node outlines
#[wasm_bindgen]
pub fn set_draw_outlines(enable: bool) -> Result<(), JsValue> {
    unsafe {
        if let Some(interface) = &mut DASHER_INTERFACE {
            match interface.set_draw_outlines(enable) {
                Ok(_) => Ok(()),
                Err(e) => Err(JsValue::from_str(&e.to_string())),
            }
        } else {
            Err(JsValue::from_str("Dasher interface not initialized"))
        }
    }
}

/// WebScreen is a DasherScreen implementation that renders to a canvas
#[derive(Clone)]
struct WebScreen {
    canvas: HtmlCanvasElement,
    context: CanvasRenderingContext2d,
    width: i32,
    height: i32,
}

impl DasherScreen for WebScreen {
    fn get_dimensions(&self) -> (i32, i32) {
        (self.width, self.height)
    }

    fn draw_rectangle(&mut self, x: i32, y: i32, width: i32, height: i32, fill_color: (u8, u8, u8, u8), outline_color: (u8, u8, u8, u8), line_width: i32) {
        // Set fill style
        let fill_rgba = format!("rgba({}, {}, {}, {})", fill_color.0, fill_color.1, fill_color.2, fill_color.3 as f32 / 255.0);
        self.context.set_fill_style(&JsValue::from_str(&fill_rgba));

        // Set stroke style
        let stroke_rgba = format!("rgba({}, {}, {}, {})", outline_color.0, outline_color.1, outline_color.2, outline_color.3 as f32 / 255.0);
        self.context.set_stroke_style(&JsValue::from_str(&stroke_rgba));
        self.context.set_line_width(line_width as f64);

        // Draw the rectangle
        self.context.begin_path();
        self.context.rect(x as f64, y as f64, width as f64, height as f64);
        self.context.fill();
        self.context.stroke();
    }

    fn draw_circle(&mut self, cx: i32, cy: i32, radius: i32, fill_color: (u8, u8, u8, u8), outline_color: (u8, u8, u8, u8), line_width: i32) {
        // Set fill style
        let fill_rgba = format!("rgba({}, {}, {}, {})", fill_color.0, fill_color.1, fill_color.2, fill_color.3 as f32 / 255.0);
        self.context.set_fill_style(&JsValue::from_str(&fill_rgba));

        // Set stroke style
        let stroke_rgba = format!("rgba({}, {}, {}, {})", outline_color.0, outline_color.1, outline_color.2, outline_color.3 as f32 / 255.0);
        self.context.set_stroke_style(&JsValue::from_str(&stroke_rgba));
        self.context.set_line_width(line_width as f64);

        // Draw the circle
        self.context.begin_path();
        self.context.arc(cx as f64, cy as f64, radius as f64, 0.0, 2.0 * std::f64::consts::PI).unwrap();
        self.context.fill();
        self.context.stroke();
    }

    fn draw_line(&mut self, x1: i32, y1: i32, x2: i32, y2: i32, color: (u8, u8, u8, u8), line_width: i32) {
        // Set stroke style
        let stroke_rgba = format!("rgba({}, {}, {}, {})", color.0, color.1, color.2, color.3 as f32 / 255.0);
        self.context.set_stroke_style(&JsValue::from_str(&stroke_rgba));
        self.context.set_line_width(line_width as f64);

        // Draw the line
        self.context.begin_path();
        self.context.move_to(x1 as f64, y1 as f64);
        self.context.line_to(x2 as f64, y2 as f64);
        self.context.stroke();
    }

    fn draw_string(&mut self, x: i32, y: i32, text: &str, color: (u8, u8, u8, u8)) {
        // Set fill style
        let fill_rgba = format!("rgba({}, {}, {}, {})", color.0, color.1, color.2, color.3 as f32 / 255.0);
        self.context.set_fill_style(&JsValue::from_str(&fill_rgba));

        // Set font
        self.context.set_font("16px Arial");

        // Draw the text
        self.context.fill_text(text, x as f64, y as f64).unwrap();
    }

    fn make_label(&mut self, text: &str, size: i32) -> Box<dyn std::any::Any> {
        // For web, we'll just return the text and size as a tuple
        Box::new((text.to_string(), size))
    }

    fn destroy_label(&mut self, _label: Box<dyn std::any::Any>) {
        // Nothing to do for web
    }

    fn get_text_size(&mut self, text: &str, size: i32) -> (i32, i32) {
        // Approximate text size based on font size and text length
        let width = text.len() as i32 * size / 2;
        let height = size;
        (width, height)
    }

    fn display(&mut self) {
        // Nothing to do for web, as we're drawing directly to the canvas
    }
}


