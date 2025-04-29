//! WASM API for DasherCore: Simplified version
//! Exposes functions for frontend to interact with the model

use wasm_bindgen::prelude::*;
use wasm_bindgen::JsValue;
use web_sys::console;

use crate::view::NodeShape;
use crate::settings::Settings;
use crate::api::DasherInterface;

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

    // For now, just log that we received the canvas ID
    // In a real implementation, we would create a WebScreen and set it in the interface
    console::log_1(&JsValue::from_str(&format!("[WASM] Canvas ID: {}", canvas_id)));

    Ok(())
}

/// Process a new frame
#[wasm_bindgen]
pub fn new_frame(timestamp: f64) -> Result<(), JsValue> {
    // For now, just log that we received a frame
    // In a real implementation, we would call the interface's new_frame method
    console::log_1(&JsValue::from_str(&format!("[WASM] New frame: {}", timestamp)));

    Ok(())
}

/// Start the interface
#[wasm_bindgen]
pub fn start() -> Result<(), JsValue> {
    console::log_1(&JsValue::from_str("[WASM] Starting Dasher"));

    // For now, just log that we're starting
    // In a real implementation, we would call the interface's start method

    Ok(())
}

/// Stop the interface
#[wasm_bindgen]
pub fn stop() -> Result<(), JsValue> {
    console::log_1(&JsValue::from_str("[WASM] Stopping Dasher"));

    // For now, just log that we're stopping
    // In a real implementation, we would call the interface's stop method

    Ok(())
}

/// Pause the interface
#[wasm_bindgen]
pub fn pause() -> Result<(), JsValue> {
    console::log_1(&JsValue::from_str("[WASM] Pausing Dasher"));

    // For now, just log that we're pausing
    // In a real implementation, we would call the interface's pause method

    Ok(())
}

/// Resume the interface
#[wasm_bindgen]
pub fn resume() -> Result<(), JsValue> {
    console::log_1(&JsValue::from_str("[WASM] Resuming Dasher"));

    // For now, just log that we're resuming
    // In a real implementation, we would call the interface's resume method

    Ok(())
}

/// Reset the interface
#[wasm_bindgen]
pub fn reset() -> Result<(), JsValue> {
    console::log_1(&JsValue::from_str("[WASM] Resetting Dasher"));

    // For now, just log that we're resetting
    // In a real implementation, we would call the interface's reset method

    Ok(())
}

/// Handle backspace
#[wasm_bindgen]
pub fn backspace() -> Result<(), JsValue> {
    console::log_1(&JsValue::from_str("[WASM] Backspace"));

    // For now, just log that we're handling backspace
    // In a real implementation, we would call the interface's backspace method

    Ok(())
}

/// Get the output text
#[wasm_bindgen]
pub fn get_output_text() -> String {
    // For now, just return a placeholder
    // In a real implementation, we would call the interface's get_output_text method
    "Dasher Output".to_string()
}

/// Set the node shape
#[wasm_bindgen]
pub fn set_node_shape(shape: &str) -> Result<(), JsValue> {
    console::log_1(&JsValue::from_str(&format!("[WASM] Setting node shape: {}", shape)));

    // For now, just log that we're setting the node shape
    // In a real implementation, we would call the interface's set_node_shape method

    Ok(())
}

/// Enable or disable X nonlinearity
#[wasm_bindgen]
pub fn set_x_nonlinear(enable: bool) -> Result<(), JsValue> {
    console::log_1(&JsValue::from_str(&format!("[WASM] Setting X nonlinearity: {}", enable)));

    // For now, just log that we're setting X nonlinearity
    // In a real implementation, we would call the interface's set_x_nonlinear method

    Ok(())
}

/// Enable or disable Y nonlinearity
#[wasm_bindgen]
pub fn set_y_nonlinear(enable: bool) -> Result<(), JsValue> {
    console::log_1(&JsValue::from_str(&format!("[WASM] Setting Y nonlinearity: {}", enable)));

    // For now, just log that we're setting Y nonlinearity
    // In a real implementation, we would call the interface's set_y_nonlinear method

    Ok(())
}

/// Enable or disable 3D text
#[wasm_bindgen]
pub fn set_text_3d(enable: bool) -> Result<(), JsValue> {
    console::log_1(&JsValue::from_str(&format!("[WASM] Setting 3D text: {}", enable)));

    // For now, just log that we're setting 3D text
    // In a real implementation, we would call the interface's set_text_3d method

    Ok(())
}

/// Enable or disable the flowing interface
#[wasm_bindgen]
pub fn set_flowing_interface(enable: bool) -> Result<(), JsValue> {
    console::log_1(&JsValue::from_str(&format!("[WASM] Setting flowing interface: {}", enable)));

    // For now, just log that we're setting the flowing interface
    // In a real implementation, we would call the interface's set_flowing_interface method

    Ok(())
}

/// Set the flowing interface speed
#[wasm_bindgen]
pub fn set_flowing_speed(speed: f64) -> Result<(), JsValue> {
    console::log_1(&JsValue::from_str(&format!("[WASM] Setting flowing speed: {}", speed)));

    // For now, just log that we're setting the flowing speed
    // In a real implementation, we would call the interface's set_flowing_speed method

    Ok(())
}

/// Enable or disable PPM (Prediction by Partial Match)
#[wasm_bindgen]
pub fn set_ppm(enable: bool) -> Result<(), JsValue> {
    console::log_1(&JsValue::from_str(&format!("[WASM] Setting PPM: {}", enable)));

    // For now, just log that we're setting PPM
    // In a real implementation, we would call the interface's set_ppm method

    Ok(())
}

/// Enable or disable drawing the crosshair
#[wasm_bindgen]
pub fn set_draw_crosshair(enable: bool) -> Result<(), JsValue> {
    console::log_1(&JsValue::from_str(&format!("[WASM] Setting draw crosshair: {}", enable)));

    // For now, just log that we're setting draw crosshair
    // In a real implementation, we would call the interface's set_draw_crosshair method

    Ok(())
}

/// Enable or disable drawing the cursor
#[wasm_bindgen]
pub fn set_draw_cursor(enable: bool) -> Result<(), JsValue> {
    console::log_1(&JsValue::from_str(&format!("[WASM] Setting draw cursor: {}", enable)));

    // For now, just log that we're setting draw cursor
    // In a real implementation, we would call the interface's set_draw_cursor method

    Ok(())
}

/// Enable or disable drawing node outlines
#[wasm_bindgen]
pub fn set_draw_outlines(enable: bool) -> Result<(), JsValue> {
    console::log_1(&JsValue::from_str(&format!("[WASM] Setting draw outlines: {}", enable)));

    // For now, just log that we're setting draw outlines
    // In a real implementation, we would call the interface's set_draw_outlines method

    Ok(())
}
