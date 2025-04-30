//! # FFI Module
//!
//! This module contains the FFI (Foreign Function Interface) for the Dasher core.
//! It provides a C-compatible API for native integration with other languages.

mod coordinates;
mod config;
pub mod context;

pub use coordinates::*;
pub use config::*;
pub use context::*;

use crate::api::DasherInterface;
use crate::input::{DasherInput, MouseInput, VirtualKey};
use crate::settings::Settings;
use crate::view::{DasherScreen, Color, Label};
use crate::view::square::{DasherViewSquare, SquareViewConfig, NodeShape};
use std::ffi::{c_char, CStr};

// Simple implementation of Label for FFI
struct SimpleLabel {
    text: String,
    wrap_size: u32,
}

impl SimpleLabel {
    fn new(text: &str, wrap_size: u32) -> Self {
        Self {
            text: text.to_string(),
            wrap_size,
        }
    }
}

impl Label for SimpleLabel {
    fn get_text(&self) -> &str {
        &self.text
    }

    fn get_wrap_size(&self) -> u32 {
        self.wrap_size
    }
}

// Simple implementation of DasherScreen for FFI
#[derive(Clone)]
struct SimpleDasherScreen {
    width: i32,
    height: i32,
    // Callback functions for rendering
    draw_rectangle_fn: Option<extern "C" fn(x1: i32, y1: i32, x2: i32, y2: i32,
                                          fill_r: u8, fill_g: u8, fill_b: u8, fill_a: u8,
                                          outline_r: u8, outline_g: u8, outline_b: u8, outline_a: u8,
                                          line_width: i32)>,
    draw_circle_fn: Option<extern "C" fn(cx: i32, cy: i32, r: i32,
                                       fill_r: u8, fill_g: u8, fill_b: u8, fill_a: u8,
                                       line_r: u8, line_g: u8, line_b: u8, line_a: u8,
                                       line_width: i32)>,
    draw_line_fn: Option<extern "C" fn(x1: i32, y1: i32, x2: i32, y2: i32,
                                     r: u8, g: u8, b: u8, a: u8,
                                     line_width: i32)>,
    draw_string_fn: Option<extern "C" fn(text: *const c_char, x: i32, y: i32, size: i32,
                                       r: u8, g: u8, b: u8, a: u8)>,
    make_label_fn: Option<extern "C" fn(text: *const c_char, size: i32) -> *mut std::ffi::c_void>,
    destroy_label_fn: Option<extern "C" fn(label: *mut std::ffi::c_void)>,
    get_text_size_fn: Option<extern "C" fn(label: *mut std::ffi::c_void, size: i32, width: *mut i32, height: *mut i32)>,
}

impl SimpleDasherScreen {
    fn new(width: i32, height: i32) -> Self {
        Self {
            width,
            height,
            draw_rectangle_fn: None,
            draw_circle_fn: None,
            draw_line_fn: None,
            draw_string_fn: None,
            make_label_fn: None,
            destroy_label_fn: None,
            get_text_size_fn: None,
        }
    }

    fn set_draw_rectangle_fn(&mut self, f: extern "C" fn(x1: i32, y1: i32, x2: i32, y2: i32,
                                                       fill_r: u8, fill_g: u8, fill_b: u8, fill_a: u8,
                                                       outline_r: u8, outline_g: u8, outline_b: u8, outline_a: u8,
                                                       line_width: i32)) {
        self.draw_rectangle_fn = Some(f);
    }

    fn set_draw_circle_fn(&mut self, f: extern "C" fn(cx: i32, cy: i32, r: i32,
                                                    fill_r: u8, fill_g: u8, fill_b: u8, fill_a: u8,
                                                    line_r: u8, line_g: u8, line_b: u8, line_a: u8,
                                                    line_width: i32)) {
        self.draw_circle_fn = Some(f);
    }

    fn set_draw_line_fn(&mut self, f: extern "C" fn(x1: i32, y1: i32, x2: i32, y2: i32,
                                                  r: u8, g: u8, b: u8, a: u8,
                                                  line_width: i32)) {
        self.draw_line_fn = Some(f);
    }

    fn set_draw_string_fn(&mut self, f: extern "C" fn(text: *const c_char, x: i32, y: i32, size: i32,
                                                   r: u8, g: u8, b: u8, a: u8)) {
        self.draw_string_fn = Some(f);
    }

    fn set_make_label_fn(&mut self, f: extern "C" fn(text: *const c_char, size: i32) -> *mut std::ffi::c_void) {
        self.make_label_fn = Some(f);
    }

    fn set_destroy_label_fn(&mut self, f: extern "C" fn(label: *mut std::ffi::c_void)) {
        self.destroy_label_fn = Some(f);
    }

    fn set_get_text_size_fn(&mut self, f: extern "C" fn(label: *mut std::ffi::c_void, size: i32, width: *mut i32, height: *mut i32)) {
        self.get_text_size_fn = Some(f);
    }
}

impl DasherScreen for SimpleDasherScreen {
    fn get_width(&self) -> i32 {
        self.width
    }

    fn get_height(&self) -> i32 {
        self.height
    }

    fn make_label(&self, text: &str, wrap_size: u32) -> Box<dyn Label> {
        // Get the global context
        let context = context::get_global_context();

        // Update screen dimensions in the context
        context.set_screen_dimensions(self.width, self.height);

        // Log debug information
        if context.get_debug_mode() {
            context.add_debug(&format!("make_label: text={}, wrap_size={}", text, wrap_size));
        }

        if let Some(f) = self.make_label_fn {
            // Convert the text to a C string
            let c_text = std::ffi::CString::new(text).unwrap_or_default();
            let label_ptr = f(c_text.as_ptr(), wrap_size as i32);

            if !label_ptr.is_null() {
                // Create a new SimpleLabel that wraps the C label
                if context.get_debug_mode() {
                    context.add_debug(&format!("make_label: Created label for '{}'", text));
                }
                return Box::new(SimpleLabel::new(text, wrap_size));
            } else if context.get_debug_mode() {
                context.add_debug("make_label: C function returned null pointer");
            }
        } else if context.get_debug_mode() {
            context.add_debug("make_label: No C function registered");
        }

        // Fallback to a simple implementation
        if context.get_debug_mode() {
            context.add_debug(&format!("make_label: Using fallback for '{}'", text));
        }
        Box::new(SimpleLabel::new(text, wrap_size))
    }

    fn text_size(&self, label: &dyn Label, font_size: u32) -> (i32, i32) {
        // Get the global context
        let context = context::get_global_context();

        // Log debug information
        if context.get_debug_mode() {
            context.add_debug(&format!("text_size: text={}, font_size={}", label.get_text(), font_size));
        }

        if let Some(f) = self.get_text_size_fn {
            // Create a new SimpleLabel that wraps the C label
            let c_text = std::ffi::CString::new(label.get_text()).unwrap_or_default();
            let label_ptr = if let Some(make_label) = self.make_label_fn {
                make_label(c_text.as_ptr(), label.get_wrap_size() as i32)
            } else {
                std::ptr::null_mut()
            };

            if !label_ptr.is_null() {
                let mut width = 0;
                let mut height = 0;
                f(label_ptr, font_size as i32, &mut width, &mut height);

                // Clean up the label
                if let Some(destroy_label) = self.destroy_label_fn {
                    destroy_label(label_ptr);
                }

                if context.get_debug_mode() {
                    context.add_debug(&format!("text_size: width={}, height={}", width, height));
                }

                return (width, height);
            } else if context.get_debug_mode() {
                context.add_debug("text_size: make_label_fn returned null pointer");
            }
        } else if context.get_debug_mode() {
            context.add_debug("text_size: No get_text_size_fn registered");
        }

        // Fallback to a simple implementation
        let text = label.get_text();
        let char_width = (font_size / 2) as i32; // Approximate width of a character
        let width = text.len() as i32 * char_width;
        let height = font_size as i32;

        if context.get_debug_mode() {
            context.add_debug(&format!("text_size (fallback): width={}, height={}", width, height));
        }

        (width, height)
    }

    fn draw_string(&mut self, label: &dyn Label, x: i32, y: i32, font_size: u32, color: Color) {
        // Get the global context
        let context = context::get_global_context();

        // Get the current drawing context
        let drawing_context = context::get_current_drawing_context();

        // Log debug information
        if context.get_debug_mode() {
            context.add_debug(&format!(
                "draw_string: text={}, x={}, y={}, font_size={}, color=({},{},{},{}), node={}",
                label.get_text(), x, y, font_size, color.r, color.g, color.b, color.a,
                drawing_context.node_id
            ));
        }

        if let Some(f) = self.draw_string_fn {
            // Convert the text to a C string
            let c_text = std::ffi::CString::new(label.get_text()).unwrap_or_default();

            // Call the C function
            f(c_text.as_ptr(), x, y, font_size as i32, color.r, color.g, color.b, color.a);
        } else {
            // Fallback to a simple implementation
            if context.get_debug_mode() {
                context.add_debug("draw_string: Using fallback implementation");
            }

            let (width, height) = self.text_size(label, font_size);

            // Draw a rectangle with the text's color
            self.draw_rectangle(
                x, y,
                x + width, y + height,
                Color::new(255, 255, 255, 0), // Transparent fill
                color,
                1
            );
        }
    }

    fn draw_rectangle(&mut self, x1: i32, y1: i32, x2: i32, y2: i32,
                     fill_color: Color, outline_color: Color, line_width: i32) {
        // Get the global context
        let context = context::get_global_context();

        // Get the current drawing context
        let drawing_context = context::get_current_drawing_context();

        // Log debug information
        if context.get_debug_mode() {
            context.add_debug(&format!(
                "draw_rectangle: x1={}, y1={}, x2={}, y2={}, fill=({},{},{},{}), outline=({},{},{},{}), width={}, node={}",
                x1, y1, x2, y2,
                fill_color.r, fill_color.g, fill_color.b, fill_color.a,
                outline_color.r, outline_color.g, outline_color.b, outline_color.a,
                line_width,
                drawing_context.node_id
            ));
        }

        if let Some(f) = self.draw_rectangle_fn {
            f(x1, y1, x2, y2,
              fill_color.r, fill_color.g, fill_color.b, fill_color.a,
              outline_color.r, outline_color.g, outline_color.b, outline_color.a,
              line_width);
        } else if context.get_debug_mode() {
            context.add_debug("draw_rectangle: No C function registered");
        }
    }

    fn draw_circle(&mut self, cx: i32, cy: i32, r: i32,
                  fill_color: Color, line_color: Color, line_width: i32) {
        // Get the global context
        let context = context::get_global_context();

        // Get the current drawing context
        let drawing_context = context::get_current_drawing_context();

        // Log debug information
        if context.get_debug_mode() {
            context.add_debug(&format!(
                "draw_circle: cx={}, cy={}, r={}, fill=({},{},{},{}), line=({},{},{},{}), width={}, node={}",
                cx, cy, r,
                fill_color.r, fill_color.g, fill_color.b, fill_color.a,
                line_color.r, line_color.g, line_color.b, line_color.a,
                line_width,
                drawing_context.node_id
            ));
        }

        if let Some(f) = self.draw_circle_fn {
            f(cx, cy, r,
              fill_color.r, fill_color.g, fill_color.b, fill_color.a,
              line_color.r, line_color.g, line_color.b, line_color.a,
              line_width);
        } else if context.get_debug_mode() {
            context.add_debug("draw_circle: No C function registered");
        }
    }

    fn draw_line(&mut self, x1: i32, y1: i32, x2: i32, y2: i32, color: Color, line_width: i32) {
        // Get the global context
        let context = context::get_global_context();

        // Get the current drawing context
        let drawing_context = context::get_current_drawing_context();

        // Log debug information
        if context.get_debug_mode() {
            context.add_debug(&format!(
                "draw_line: x1={}, y1={}, x2={}, y2={}, color=({},{},{},{}), width={}, node={}",
                x1, y1, x2, y2,
                color.r, color.g, color.b, color.a,
                line_width,
                drawing_context.node_id
            ));
        }

        if let Some(f) = self.draw_line_fn {
            f(x1, y1, x2, y2, color.r, color.g, color.b, color.a, line_width);
        } else if context.get_debug_mode() {
            context.add_debug("draw_line: No C function registered");
        }
    }

    fn display(&mut self) {
        // Get the global context
        let context = context::get_global_context();

        // Log debug information
        if context.get_debug_mode() {
            context.add_debug("display: Frame complete");
        }
    }

    fn is_point_visible(&self, x: i32, y: i32) -> bool {
        // Get the global context
        let context = context::get_global_context();

        // Log debug information
        if context.get_debug_mode() {
            context.add_debug(&format!("is_point_visible: x={}, y={}", x, y));
        }

        true
    }
}

#[repr(C)]
pub struct DasherInterfaceFFI {
    interface: DasherInterface,
}

#[repr(C)]
pub struct DasherSettingsFFI {
    // TODO: Define FFI-compatible settings structure
}

#[repr(C)]
pub enum DasherErrorCode {
    Success = 0,
    InvalidParameter = 1,
    RenderingError = 2,
    InputError = 3,
    SettingsError = 4,
    Other = 5,
}

/// Color representation for FFI
#[repr(C)]
pub struct ColorFFI {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl From<ColorFFI> for Color {
    fn from(color: ColorFFI) -> Self {
        Color::new(color.r, color.g, color.b, color.a)
    }
}

impl From<Color> for ColorFFI {
    fn from(color: Color) -> Self {
        ColorFFI {
            r: color.r,
            g: color.g,
            b: color.b,
            a: color.a,
        }
    }
}

/// Opaque handle to a DasherInput
#[repr(C)]
pub struct DasherInputFFI {
    input: Box<dyn DasherInput>,
}

/// Opaque handle to a DasherScreen
#[repr(C)]
pub struct DasherScreenFFI {
    screen: SimpleDasherScreen,
}

#[no_mangle]
pub extern "C" fn dasher_interface_create(
    settings: *const DasherSettingsFFI
) -> *mut DasherInterfaceFFI {
    println!("FFI: Creating DasherInterface");

    // TODO: Implement proper settings conversion
    let settings = if settings.is_null() {
        println!("FFI: Using default settings");
        Settings::new()
    } else {
        println!("FFI: Converting FFI settings to Rust settings");
        // Convert FFI settings to Rust settings
        Settings::new()
    };

    println!("FFI: Creating DasherInterface with settings");
    let mut interface = DasherInterface::new(settings);

    // Initialize the model
    println!("FFI: Initializing model");
    if let Err(e) = interface.model_mut().initialize() {
        println!("FFI: Failed to initialize model: {:?}", e);
        return std::ptr::null_mut();
    }
    println!("FFI: Model initialized successfully");

    // Try different paths for the English alphabet
    println!("FFI: Loading alphabet");
    let alphabet_paths = [
        "data/alphabets/alphabet.english.xml",
        "DasherUI/Data/alphabet.english.with.limited.punctuation.xml",
        "DasherUI-main/build/DasherUI/Data/alphabet.english.with.limited.punctuation.xml",
        "./DasherUI/Data/alphabet.english.with.limited.punctuation.xml"
    ];

    let mut alphabet_loaded = false;
    for &path in &alphabet_paths {
        let alphabet_path = std::path::Path::new(path);
        println!("FFI: Trying alphabet path: {}", path);
        if alphabet_path.exists() {
            println!("FFI: Alphabet path exists: {}", path);
            match crate::alphabet::load_alphabet(alphabet_path) {
                Ok(alphabet_info) => {
                    println!("FFI: Loaded alphabet from {}", path);
                    let alphabet = crate::alphabet::Alphabet::from_info(alphabet_info);
                    interface.model_mut().set_alphabet(alphabet);
                    alphabet_loaded = true;
                    break;
                }
                Err(e) => {
                    println!("FFI: Failed to load alphabet from {}: {:?}", path, e);
                }
            }
        } else {
            println!("FFI: Alphabet path does not exist: {}", path);
        }
    }

    // If no alphabet was loaded, create a default English alphabet
    if !alphabet_loaded {
        println!("FFI: Using default English alphabet");
        let alphabet = crate::alphabet::Alphabet::english();
        interface.model_mut().set_alphabet(alphabet);
    }

    // Try different paths for the training data
    println!("FFI: Loading training data");
    let training_paths = [
        "data/training/training_english_GB.txt",
        "DasherUI/Data/training_english_GB.txt",
        "DasherUI-main/build/DasherUI/Data/training_english_GB.txt",
        "./DasherUI/Data/training_english_GB.txt"
    ];

    let mut training_loaded = false;
    for &path in &training_paths {
        let training_path = std::path::Path::new(path);
        println!("FFI: Trying training path: {}", path);
        if training_path.exists() {
            println!("FFI: Training path exists: {}", path);
            match std::fs::read_to_string(training_path) {
                Ok(training_text) => {
                    println!("FFI: Loaded training data from {}", path);
                    println!("FFI: Training language model with {} characters", training_text.len());
                    // Train the language model with the text
                    for c in training_text.chars() {
                        interface.model_mut().update_language_model(c);
                    }
                    training_loaded = true;
                    break;
                }
                Err(e) => {
                    println!("FFI: Failed to load training data from {}: {:?}", path, e);
                }
            }
        } else {
            println!("FFI: Training path does not exist: {}", path);
        }
    }

    if !training_loaded {
        println!("FFI: No training data loaded");
    }

    // Create a default view if none exists
    if interface.view().is_none() {
        println!("FFI: Creating default view");
        // We'll create a view when the screen is set
    }

    println!("FFI: DasherInterface created successfully");
    Box::into_raw(Box::new(DasherInterfaceFFI { interface }))
}

/// # Safety
///
/// The `interface` pointer must be a valid pointer to a `DasherInterfaceFFI` object
/// that was created by `dasher_create_interface`. After this function is called,
/// the pointer is no longer valid and should not be used.
#[no_mangle]
pub unsafe extern "C" fn dasher_interface_destroy(interface: *mut DasherInterfaceFFI) {
    if !interface.is_null() {
        let _ = Box::from_raw(interface);
    }
}

/// Process a new frame in the Dasher interface
///
/// # Safety
///
/// The `interface` pointer must be a valid pointer to a `DasherInterfaceFFI` object
/// that was created by `dasher_create_interface`.
#[no_mangle]
pub unsafe extern "C" fn dasher_interface_new_frame(
    interface: *mut DasherInterfaceFFI,
    time_ms: u64
) -> bool {
    if interface.is_null() {
        return false;
    }

    (*interface).interface.new_frame(time_ms)
}

/// Create a mouse input device
#[no_mangle]
pub extern "C" fn dasher_create_mouse_input() -> *mut DasherInputFFI {
    let input = Box::new(MouseInput::new()) as Box<dyn DasherInput>;
    Box::into_raw(Box::new(DasherInputFFI { input }))
}

/// Destroy an input device
///
/// # Safety
///
/// The `input` pointer must be a valid pointer to a `DasherInputFFI` object
/// that was created by `dasher_create_mouse_input`. After this function is called,
/// the pointer is no longer valid and should not be used.
#[no_mangle]
pub unsafe extern "C" fn dasher_destroy_input(input: *mut DasherInputFFI) {
    if !input.is_null() {
        let _ = Box::from_raw(input);
    }
}

/// Set the input device for a DasherInterface
#[no_mangle]
pub unsafe extern "C" fn dasher_interface_set_input(
    interface: *mut DasherInterfaceFFI,
    input: *mut DasherInputFFI
) -> bool {
    if interface.is_null() || input.is_null() {
        return false;
    }

    let input_box = Box::from_raw(input);
    (*interface).interface.set_input(input_box.input.box_clone());
    // Don't drop the input, just forget the box
    std::mem::forget(input_box);
    true
}

/// Set mouse coordinates for a mouse input device
#[no_mangle]
pub unsafe extern "C" fn dasher_set_mouse_coordinates(
    input: *mut DasherInputFFI,
    x: i32,
    y: i32
) -> bool {
    if input.is_null() {
        return false;
    }

    // For now, we'll assume it's a MouseInput since that's all we're using
    // In a more complete implementation, we'd need a way to check the type
    let input_ref = &mut (*input).input;

    // Check if the name contains "Mouse" as a simple heuristic
    if input_ref.get_name().contains("Mouse") {
        // Create a new MouseInput with the updated coordinates
        let mut new_mouse = MouseInput::new();
        new_mouse.set_coordinates(x, y);

        // Replace the input with the new one
        *input_ref = Box::new(new_mouse);
        true
    } else {
        false
    }
}

/// Handle a key down event
#[no_mangle]
pub unsafe extern "C" fn dasher_interface_key_down(
    interface: *mut DasherInterfaceFFI,
    time_ms: u64,
    key: i32
) {
    if interface.is_null() {
        return;
    }

    let virtual_key = match key {
        0 => VirtualKey::PrimaryInput,
        1 => VirtualKey::SecondaryInput,
        2 => VirtualKey::TertiaryInput,
        3 => VirtualKey::StartStopKey,
        4 => VirtualKey::Button1,
        5 => VirtualKey::Button2,
        6 => VirtualKey::Button3,
        7 => VirtualKey::Button4,
        8 => VirtualKey::Button5,
        9 => VirtualKey::Left,
        10 => VirtualKey::Right,
        11 => VirtualKey::Up,
        12 => VirtualKey::Down,
        13 => VirtualKey::Delete,
        14 => VirtualKey::Backspace,
        15 => VirtualKey::Tab,
        16 => VirtualKey::Return,
        17 => VirtualKey::Escape,
        18 => VirtualKey::Space,
        _ => VirtualKey::Other(' '),
    };
    (*interface).interface.key_down(time_ms, virtual_key);
}

/// Handle a key up event
#[no_mangle]
pub unsafe extern "C" fn dasher_interface_key_up(
    interface: *mut DasherInterfaceFFI,
    time_ms: u64,
    key: i32
) {
    if interface.is_null() {
        return;
    }

    let virtual_key = match key {
        0 => VirtualKey::PrimaryInput,
        1 => VirtualKey::SecondaryInput,
        2 => VirtualKey::TertiaryInput,
        3 => VirtualKey::StartStopKey,
        4 => VirtualKey::Button1,
        5 => VirtualKey::Button2,
        6 => VirtualKey::Button3,
        7 => VirtualKey::Button4,
        8 => VirtualKey::Button5,
        9 => VirtualKey::Left,
        10 => VirtualKey::Right,
        11 => VirtualKey::Up,
        12 => VirtualKey::Down,
        13 => VirtualKey::Delete,
        14 => VirtualKey::Backspace,
        15 => VirtualKey::Tab,
        16 => VirtualKey::Return,
        17 => VirtualKey::Escape,
        18 => VirtualKey::Space,
        _ => VirtualKey::Other(' '),
    };
    (*interface).interface.key_up(time_ms, virtual_key);
}

/// Start Dasher
#[no_mangle]
pub unsafe extern "C" fn dasher_interface_start(interface: *mut DasherInterfaceFFI) {
    if interface.is_null() {
        return;
    }

    (*interface).interface.start();
}

/// Stop Dasher
#[no_mangle]
pub unsafe extern "C" fn dasher_interface_stop(interface: *mut DasherInterfaceFFI) {
    if interface.is_null() {
        return;
    }

    (*interface).interface.stop();
}

/// Pause Dasher
#[no_mangle]
pub unsafe extern "C" fn dasher_interface_pause(interface: *mut DasherInterfaceFFI) {
    if interface.is_null() {
        return;
    }

    (*interface).interface.pause();
}

/// Resume Dasher
#[no_mangle]
pub unsafe extern "C" fn dasher_interface_resume(interface: *mut DasherInterfaceFFI) {
    if interface.is_null() {
        return;
    }

    (*interface).interface.resume();
}

/// Check if Dasher is running
#[no_mangle]
pub unsafe extern "C" fn dasher_interface_is_running(interface: *mut DasherInterfaceFFI) -> bool {
    if interface.is_null() {
        return false;
    }

    (*interface).interface.is_running()
}

/// Check if Dasher is paused
#[no_mangle]
pub unsafe extern "C" fn dasher_interface_is_paused(interface: *mut DasherInterfaceFFI) -> bool {
    if interface.is_null() {
        return false;
    }

    (*interface).interface.is_paused()
}

/// Get the current offset in the text buffer
#[no_mangle]
pub unsafe extern "C" fn dasher_interface_get_offset(interface: *mut DasherInterfaceFFI) -> i32 {
    if interface.is_null() {
        return 0;
    }

    (*interface).interface.get_offset()
}

/// Edit the output text
#[no_mangle]
pub unsafe extern "C" fn dasher_interface_edit_output(
    interface: *mut DasherInterfaceFFI,
    text: *const c_char
) {
    if interface.is_null() || text.is_null() {
        return;
    }

    let c_str = CStr::from_ptr(text);
    if let Ok(text) = c_str.to_str() {
        (*interface).interface.edit_output(text);
    }
}

/// Create a new screen for rendering
#[no_mangle]
pub extern "C" fn dasher_create_screen(
    width: i32,
    height: i32,
) -> *mut DasherScreenFFI {
    let screen = SimpleDasherScreen::new(width, height);
    Box::into_raw(Box::new(DasherScreenFFI { screen }))
}

/// Destroy a screen
#[no_mangle]
pub unsafe extern "C" fn dasher_destroy_screen(screen: *mut DasherScreenFFI) {
    if !screen.is_null() {
        let _ = Box::from_raw(screen);
    }
}

/// Set the draw rectangle callback for a screen
#[no_mangle]
pub unsafe extern "C" fn dasher_screen_set_draw_rectangle_callback(
    screen: *mut DasherScreenFFI,
    callback: extern "C" fn(x1: i32, y1: i32, x2: i32, y2: i32,
                          fill_r: u8, fill_g: u8, fill_b: u8, fill_a: u8,
                          outline_r: u8, outline_g: u8, outline_b: u8, outline_a: u8,
                          line_width: i32),
) {
    if screen.is_null() {
        return;
    }

    (*screen).screen.set_draw_rectangle_fn(callback);
}

/// Set the draw circle callback for a screen
#[no_mangle]
pub unsafe extern "C" fn dasher_screen_set_draw_circle_callback(
    screen: *mut DasherScreenFFI,
    callback: extern "C" fn(cx: i32, cy: i32, r: i32,
                          fill_r: u8, fill_g: u8, fill_b: u8, fill_a: u8,
                          line_r: u8, line_g: u8, line_b: u8, line_a: u8,
                          line_width: i32),
) {
    if screen.is_null() {
        return;
    }

    (*screen).screen.set_draw_circle_fn(callback);
}

/// Set the draw line callback for a screen
#[no_mangle]
pub unsafe extern "C" fn dasher_screen_set_draw_line_callback(
    screen: *mut DasherScreenFFI,
    callback: extern "C" fn(x1: i32, y1: i32, x2: i32, y2: i32,
                          r: u8, g: u8, b: u8, a: u8,
                          line_width: i32),
) {
    if screen.is_null() {
        return;
    }

    (*screen).screen.set_draw_line_fn(callback);
}

/// Set the draw string callback for a screen
#[no_mangle]
pub unsafe extern "C" fn dasher_screen_set_draw_string_callback(
    screen: *mut DasherScreenFFI,
    callback: extern "C" fn(text: *const c_char, x: i32, y: i32, size: i32,
                          r: u8, g: u8, b: u8, a: u8),
) {
    if screen.is_null() {
        return;
    }

    (*screen).screen.set_draw_string_fn(callback);
}

/// Set the make label callback for a screen
#[no_mangle]
pub unsafe extern "C" fn dasher_screen_set_make_label_callback(
    screen: *mut DasherScreenFFI,
    callback: extern "C" fn(text: *const c_char, size: i32) -> *mut std::ffi::c_void,
) {
    if screen.is_null() {
        return;
    }

    (*screen).screen.set_make_label_fn(callback);
}

/// Set the destroy label callback for a screen
#[no_mangle]
pub unsafe extern "C" fn dasher_screen_set_destroy_label_callback(
    screen: *mut DasherScreenFFI,
    callback: extern "C" fn(label: *mut std::ffi::c_void),
) {
    if screen.is_null() {
        return;
    }

    (*screen).screen.set_destroy_label_fn(callback);
}

/// Set the get text size callback for a screen
#[no_mangle]
pub unsafe extern "C" fn dasher_screen_set_get_text_size_callback(
    screen: *mut DasherScreenFFI,
    callback: extern "C" fn(label: *mut std::ffi::c_void, size: i32, width: *mut i32, height: *mut i32),
) {
    if screen.is_null() {
        return;
    }

    (*screen).screen.set_get_text_size_fn(callback);
}

/// Set the screen for a DasherInterface
#[no_mangle]
pub unsafe extern "C" fn dasher_interface_set_screen(
    interface: *mut DasherInterfaceFFI,
    screen: *mut DasherScreenFFI,
) -> bool {
    println!("FFI: Setting screen for interface");

    if interface.is_null() {
        println!("FFI: Interface pointer is null");
        return false;
    }

    if screen.is_null() {
        println!("FFI: Screen pointer is null");
        return false;
    }

    let interface = &mut *interface;
    let screen_ref = &mut *screen;

    println!("FFI: Screen dimensions: {}x{}", screen_ref.screen.get_width(), screen_ref.screen.get_height());

    // Clone the screen
    let screen_clone = screen_ref.screen.clone();

    // Create a square view with the screen
    println!("FFI: Creating square view with screen");
    let mut view = Box::new(DasherViewSquare::new(Box::new(screen_clone)));

    // Configure the view with default settings for flowing interface
    println!("FFI: Configuring square view");

    // Enable flowing interface
    view.set_flowing_interface(true);
    println!("FFI: Enabled flowing interface");

    // Set flowing speed
    view.set_flowing_speed(2.0);
    println!("FFI: Set flowing speed to 2.0");

    // Enable X nonlinearity
    view.set_x_nonlinear(true);
    println!("FFI: Enabled X nonlinearity");

    // Enable Y nonlinearity
    view.set_y_nonlinear(true);
    println!("FFI: Enabled Y nonlinearity");

    // Set node shape to Rectangle
    view.set_node_shape(NodeShape::Rectangle);
    println!("FFI: Set node shape to Rectangle");

    // Enable crosshair, cursor, and outlines
    view.config_mut().draw_crosshair = true;
    view.config_mut().draw_cursor = true;
    view.config_mut().draw_outlines = true;
    println!("FFI: Enabled crosshair, cursor, and outlines");

    // Set the view
    println!("FFI: Setting view for interface");
    let result = interface.interface.set_view(view);

    if result.is_ok() {
        println!("FFI: View set successfully");
        true
    } else {
        println!("FFI: Failed to set view: {:?}", result);
        false
    }
}

/// Node shape types for FFI
#[repr(C)]
pub enum NodeShapeFFI {
    /// Rectangle shape
    Rectangle = 0,
    /// Triangle shape
    Triangle = 1,
    /// Truncated triangle shape
    TruncatedTriangle = 2,
    /// Circle shape
    Circle = 3,
    /// Quadric shape (curved)
    Quadric = 4,
}

impl From<NodeShapeFFI> for crate::view::NodeShape {
    fn from(shape: NodeShapeFFI) -> Self {
        match shape {
            NodeShapeFFI::Rectangle => crate::view::NodeShape::Rectangle,
            NodeShapeFFI::Triangle => crate::view::NodeShape::Triangle,
            NodeShapeFFI::TruncatedTriangle => crate::view::NodeShape::TruncatedTriangle,
            NodeShapeFFI::Circle => crate::view::NodeShape::Circle,
            NodeShapeFFI::Quadric => crate::view::NodeShape::Quadric,
        }
    }
}

/// Set the node shape for the Square View
#[no_mangle]
pub unsafe extern "C" fn dasher_interface_set_node_shape(
    interface: *mut DasherInterfaceFFI,
    shape: NodeShapeFFI,
) -> bool {
    if interface.is_null() {
        return false;
    }

    let interface = &mut *interface;
    let result = interface.interface.set_node_shape(shape.into());
    result.is_ok()
}

/// Enable or disable X nonlinearity
#[no_mangle]
pub unsafe extern "C" fn dasher_interface_set_x_nonlinear(
    interface: *mut DasherInterfaceFFI,
    enable: bool,
) -> bool {
    if interface.is_null() {
        return false;
    }

    let interface = &mut *interface;
    let result = interface.interface.set_x_nonlinear(enable);
    result.is_ok()
}

/// Enable or disable Y nonlinearity
#[no_mangle]
pub unsafe extern "C" fn dasher_interface_set_y_nonlinear(
    interface: *mut DasherInterfaceFFI,
    enable: bool,
) -> bool {
    if interface.is_null() {
        return false;
    }

    let interface = &mut *interface;
    let result = interface.interface.set_y_nonlinear(enable);
    result.is_ok()
}

/// Enable or disable 3D text
#[no_mangle]
pub unsafe extern "C" fn dasher_interface_set_text_3d(
    interface: *mut DasherInterfaceFFI,
    enable: bool,
) -> bool {
    if interface.is_null() {
        return false;
    }

    let interface = &mut *interface;
    let result = interface.interface.set_text_3d(enable);
    result.is_ok()
}

/// Enable or disable the flowing interface
#[no_mangle]
pub unsafe extern "C" fn dasher_interface_set_flowing_interface(
    interface: *mut DasherInterfaceFFI,
    enable: bool,
) -> bool {
    if interface.is_null() {
        return false;
    }

    let interface = &mut *interface;
    let result = interface.interface.set_flowing_interface(enable);
    result.is_ok()
}

/// Set the flowing interface speed
#[no_mangle]
pub unsafe extern "C" fn dasher_interface_set_flowing_speed(
    interface: *mut DasherInterfaceFFI,
    speed: f64,
) -> bool {
    if interface.is_null() {
        return false;
    }

    let interface = &mut *interface;
    let result = interface.interface.set_flowing_speed(speed);
    result.is_ok()
}

/// Enable or disable PPM (Prediction by Partial Match)
#[no_mangle]
pub unsafe extern "C" fn dasher_interface_set_ppm(
    interface: *mut DasherInterfaceFFI,
    enable: bool,
) -> bool {
    if interface.is_null() {
        return false;
    }

    let interface = &mut *interface;
    let result = interface.interface.set_ppm(enable);
    result.is_ok()
}

/// Enable or disable drawing the crosshair
#[no_mangle]
pub unsafe extern "C" fn dasher_interface_set_draw_crosshair(
    interface: *mut DasherInterfaceFFI,
    enable: bool,
) -> bool {
    if interface.is_null() {
        return false;
    }

    let interface = &mut *interface;
    let result = interface.interface.set_draw_crosshair(enable);
    result.is_ok()
}

/// Enable or disable drawing the cursor
#[no_mangle]
pub unsafe extern "C" fn dasher_interface_set_draw_cursor(
    interface: *mut DasherInterfaceFFI,
    enable: bool,
) -> bool {
    if interface.is_null() {
        return false;
    }

    let interface = &mut *interface;
    let result = interface.interface.set_draw_cursor(enable);
    result.is_ok()
}

/// Enable or disable drawing node outlines
#[no_mangle]
pub unsafe extern "C" fn dasher_interface_set_draw_outlines(
    interface: *mut DasherInterfaceFFI,
    enable: bool,
) -> bool {
    if interface.is_null() {
        return false;
    }

    let interface = &mut *interface;
    let result = interface.interface.set_draw_outlines(enable);
    result.is_ok()
}

/// Get the output text
#[no_mangle]
pub unsafe extern "C" fn dasher_interface_get_output(
    interface: *mut DasherInterfaceFFI,
    buffer: *mut c_char,
    buffer_size: usize
) -> usize {
    if interface.is_null() || buffer.is_null() {
        return 0;
    }

    let output = (*interface).interface.get_output_text();
    let output_len = output.len();

    if output_len == 0 {
        return 0;
    }

    // Copy the output to the buffer
    let copy_len = std::cmp::min(output_len, buffer_size - 1);
    let output_bytes = output.as_bytes();

    std::ptr::copy_nonoverlapping(
        output_bytes.as_ptr(),
        buffer as *mut u8,
        copy_len
    );

    // Null-terminate the string
    *buffer.add(copy_len) = 0;

    copy_len
}

/// Transform Dasher coordinates to screen coordinates
///
/// # Safety
///
/// This function is unsafe because it dereferences raw pointers.
#[no_mangle]
pub unsafe extern "C" fn dasher_transform_coordinates(
    dasher_x: i64,
    dasher_y: i64,
    screen_width: i32,
    screen_height: i32,
    orientation: i32,
    screen_x: *mut i32,
    screen_y: *mut i32
) -> bool {
    if screen_x.is_null() || screen_y.is_null() {
        return false;
    }

    let (x, y) = coordinates::dasher_to_screen(
        dasher_x,
        dasher_y,
        screen_width,
        screen_height,
        orientation
    );

    *screen_x = x;
    *screen_y = y;

    true
}

/// Transform screen coordinates to Dasher coordinates
///
/// # Safety
///
/// This function is unsafe because it dereferences raw pointers.
#[no_mangle]
pub unsafe extern "C" fn dasher_screen_to_dasher(
    screen_x: i32,
    screen_y: i32,
    screen_width: i32,
    screen_height: i32,
    orientation: i32,
    dasher_x: *mut i64,
    dasher_y: *mut i64
) -> bool {
    if dasher_x.is_null() || dasher_y.is_null() {
        return false;
    }

    let (x, y) = coordinates::screen_to_dasher(
        screen_x,
        screen_y,
        screen_width,
        screen_height,
        orientation
    );

    *dasher_x = x;
    *dasher_y = y;

    true
}

/// Transform a rectangle from Dasher coordinates to screen coordinates
///
/// # Safety
///
/// This function is unsafe because it dereferences raw pointers.
#[no_mangle]
pub unsafe extern "C" fn dasher_transform_rectangle(
    dasher_x1: i64,
    dasher_y1: i64,
    dasher_x2: i64,
    dasher_y2: i64,
    screen_width: i32,
    screen_height: i32,
    orientation: i32,
    screen_x1: *mut i32,
    screen_y1: *mut i32,
    screen_x2: *mut i32,
    screen_y2: *mut i32
) -> bool {
    if screen_x1.is_null() || screen_y1.is_null() || screen_x2.is_null() || screen_y2.is_null() {
        return false;
    }

    let (x1, y1, x2, y2) = coordinates::transform_rectangle(
        dasher_x1,
        dasher_y1,
        dasher_x2,
        dasher_y2,
        screen_width,
        screen_height,
        orientation
    );

    *screen_x1 = x1;
    *screen_y1 = y1;
    *screen_x2 = x2;
    *screen_y2 = y2;

    true
}

/// Create a default square view configuration
#[no_mangle]
pub extern "C" fn dasher_create_square_view_config() -> *mut SquareViewConfigFFI {
    let config = SquareViewConfigFFI::default();
    Box::into_raw(Box::new(config))
}

/// Destroy a square view configuration
///
/// # Safety
///
/// This function is unsafe because it deallocates memory from a raw pointer.
/// The pointer must have been created by `dasher_create_square_view_config`.
#[no_mangle]
pub unsafe extern "C" fn dasher_destroy_square_view_config(config: *mut SquareViewConfigFFI) {
    if !config.is_null() {
        let _ = Box::from_raw(config);
    }
}

/// Set the square view configuration for a Dasher interface
///
/// # Safety
///
/// This function is unsafe because it dereferences raw pointers.
/// Both `interface` and `config` must be valid pointers to their respective types.
#[no_mangle]
pub unsafe extern "C" fn dasher_interface_set_square_view_config(
    interface: *mut DasherInterfaceFFI,
    config: *const SquareViewConfigFFI
) -> bool {
    if interface.is_null() || config.is_null() {
        return false;
    }

    let interface = &mut *interface;
    let config = &*config;

    // Convert FFI config to Rust config
    let rust_config: SquareViewConfig = config::ffi_to_rust_config(config);

    // Get the square view from the interface
    if let Some(square_view) = config::get_square_view(&mut interface.interface) {
        // Update the configuration
        *square_view.config_mut() = rust_config;
        true
    } else {
        false
    }
}

/// Get the square view configuration from a Dasher interface
///
/// # Safety
///
/// This function is unsafe because it dereferences raw pointers.
/// Both `interface` and `config` must be valid pointers to their respective types.
#[no_mangle]
pub unsafe extern "C" fn dasher_interface_get_square_view_config(
    interface: *mut DasherInterfaceFFI,
    config: *mut SquareViewConfigFFI
) -> bool {
    if interface.is_null() || config.is_null() {
        return false;
    }

    let interface = &mut *interface;

    // Get the square view from the interface
    if let Some(square_view) = config::get_square_view(&mut interface.interface) {
        // Get the configuration
        let rust_config = square_view.config();

        // Convert Rust config to FFI config
        *config = rust_config.clone().into();
        true
    } else {
        false
    }
}

// Functions moved to avoid duplication

/// Enable or disable debug mode
#[no_mangle]
pub extern "C" fn dasher_set_debug_mode(enable: bool) {
    let context = context::get_global_context();
    context.set_debug_mode(enable);
}

/// Get debug mode
#[no_mangle]
pub extern "C" fn dasher_get_debug_mode() -> bool {
    let context = context::get_global_context();
    context.get_debug_mode()
}

/// Get the number of error messages
#[no_mangle]
pub extern "C" fn dasher_get_error_count() -> i32 {
    let context = context::get_global_context();
    let errors = context.get_errors();
    errors.len() as i32
}

/// Get an error message
///
/// # Safety
///
/// This function is unsafe because it dereferences raw pointers.
/// The `buffer` pointer must be valid and point to a buffer of at least `buffer_size` bytes.
#[no_mangle]
pub unsafe extern "C" fn dasher_get_error_message(
    index: i32,
    buffer: *mut c_char,
    buffer_size: usize
) -> bool {
    if buffer.is_null() {
        return false;
    }

    let context = context::get_global_context();
    let errors = context.get_errors();

    if index < 0 || index >= errors.len() as i32 {
        return false;
    }

    let message = &errors[index as usize];
    let copy_len = std::cmp::min(message.len(), buffer_size - 1);

    std::ptr::copy_nonoverlapping(
        message.as_ptr(),
        buffer as *mut u8,
        copy_len
    );

    // Null-terminate the string
    *buffer.add(copy_len) = 0;

    true
}

/// Get the number of debug messages
#[no_mangle]
pub extern "C" fn dasher_get_debug_message_count() -> i32 {
    let context = context::get_global_context();
    let messages = context.get_debug_messages();
    messages.len() as i32
}

/// Get a debug message
///
/// # Safety
///
/// This function is unsafe because it dereferences raw pointers.
/// The `buffer` pointer must be valid and point to a buffer of at least `buffer_size` bytes.
#[no_mangle]
pub unsafe extern "C" fn dasher_get_debug_message(
    index: i32,
    buffer: *mut c_char,
    buffer_size: usize
) -> bool {
    if buffer.is_null() {
        return false;
    }

    let context = context::get_global_context();
    let messages = context.get_debug_messages();

    if index < 0 || index >= messages.len() as i32 {
        return false;
    }

    let message = &messages[index as usize];
    let copy_len = std::cmp::min(message.len(), buffer_size - 1);

    std::ptr::copy_nonoverlapping(
        message.as_ptr(),
        buffer as *mut u8,
        copy_len
    );

    // Null-terminate the string
    *buffer.add(copy_len) = 0;

    true
}

/// Add a debug message
///
/// # Safety
///
/// This function is unsafe because it dereferences raw pointers.
/// The `message` pointer must be valid and point to a null-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn dasher_add_debug_message(
    message: *const c_char
) -> bool {
    if message.is_null() {
        return false;
    }

    let c_str = CStr::from_ptr(message);
    if let Ok(message_str) = c_str.to_str() {
        let context = context::get_global_context();
        context.add_debug(message_str);
        true
    } else {
        false
    }
}

// Functions moved to avoid duplication
