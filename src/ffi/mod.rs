//! # FFI Module
//!
//! This module contains the FFI (Foreign Function Interface) for the Dasher core.
//! It provides a C-compatible API for native integration with other languages.

use crate::api::DasherInterface;
use crate::input::{DasherInput, MouseInput, VirtualKey};
use crate::settings::Settings;
use crate::view::{DasherScreen, Color, Label};
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
        if let Some(f) = self.make_label_fn {
            // Convert the text to a C string
            let c_text = std::ffi::CString::new(text).unwrap_or_default();
            let label_ptr = f(c_text.as_ptr(), wrap_size as i32);

            if !label_ptr.is_null() {
                // Create a new SimpleLabel that wraps the C label
                return Box::new(SimpleLabel::new(text, wrap_size));
            }
        }

        // Fallback to a simple implementation
        Box::new(SimpleLabel::new(text, wrap_size))
    }

    fn text_size(&self, label: &dyn Label, font_size: u32) -> (i32, i32) {
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

                return (width, height);
            }
        }

        // Fallback to a simple implementation
        let text = label.get_text();
        let char_width = (font_size / 2) as i32; // Approximate width of a character
        let width = text.len() as i32 * char_width;
        let height = font_size as i32;

        (width, height)
    }

    fn draw_string(&mut self, label: &dyn Label, x: i32, y: i32, font_size: u32, color: Color) {
        if let Some(f) = self.draw_string_fn {
            // Convert the text to a C string
            let c_text = std::ffi::CString::new(label.get_text()).unwrap_or_default();

            // Call the C function
            f(c_text.as_ptr(), x, y, font_size as i32, color.r, color.g, color.b, color.a);
        } else {
            // Fallback to a simple implementation
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
        if let Some(f) = self.draw_rectangle_fn {
            f(x1, y1, x2, y2,
              fill_color.r, fill_color.g, fill_color.b, fill_color.a,
              outline_color.r, outline_color.g, outline_color.b, outline_color.a,
              line_width);
        }
    }

    fn draw_circle(&mut self, cx: i32, cy: i32, r: i32,
                  fill_color: Color, line_color: Color, line_width: i32) {
        if let Some(f) = self.draw_circle_fn {
            f(cx, cy, r,
              fill_color.r, fill_color.g, fill_color.b, fill_color.a,
              line_color.r, line_color.g, line_color.b, line_color.a,
              line_width);
        }
    }

    fn draw_line(&mut self, x1: i32, y1: i32, x2: i32, y2: i32, color: Color, line_width: i32) {
        if let Some(f) = self.draw_line_fn {
            f(x1, y1, x2, y2, color.r, color.g, color.b, color.a, line_width);
        }
    }

    fn display(&mut self) {
        // Nothing to do - rendering is handled by the ImGUI app
    }

    fn is_point_visible(&self, _x: i32, _y: i32) -> bool {
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
    // TODO: Implement proper settings conversion
    let settings = if settings.is_null() {
        Settings::new()
    } else {
        // Convert FFI settings to Rust settings
        Settings::new()
    };

    let interface = DasherInterface::new(settings);

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
    if interface.is_null() || screen.is_null() {
        return false;
    }

    let interface = &mut *interface;
    let screen_ref = &mut *screen;
    let screen_clone = screen_ref.screen.clone();
    let result = interface.interface.change_screen(Box::new(screen_clone));
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
