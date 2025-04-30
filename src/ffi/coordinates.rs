//! # Coordinate Transformation Module
//!
//! This module provides functions for transforming coordinates between
//! Dasher space and screen space in the FFI layer.

use crate::model::DasherModel;

/// Constants for coordinate transformation
const DASHER_X_MAX: i64 = DasherModel::MAX_X;
const DASHER_Y_MAX: i64 = DasherModel::MAX_Y;
const DASHER_Y_MIN: i64 = 0;
const DASHER_ORIGIN_X: i64 = DasherModel::ORIGIN_X;
const DASHER_ORIGIN_Y: i64 = DasherModel::ORIGIN_Y;

/// Transform Dasher coordinates to screen coordinates
///
/// This function maps Dasher coordinates to screen coordinates based on the
/// screen dimensions and orientation.
///
/// # Arguments
///
/// * `dasher_x` - X coordinate in Dasher space
/// * `dasher_y` - Y coordinate in Dasher space
/// * `screen_width` - Width of the screen in pixels
/// * `screen_height` - Height of the screen in pixels
/// * `orientation` - Orientation of the view (0 = LeftToRight, 1 = RightToLeft, 2 = TopToBottom, 3 = BottomToTop)
///
/// # Returns
///
/// A tuple of (screen_x, screen_y) coordinates
pub fn dasher_to_screen(
    dasher_x: i64,
    dasher_y: i64,
    screen_width: i32,
    screen_height: i32,
    orientation: i32,
) -> (i32, i32) {
    // Apply orientation-specific transformations
    match orientation {
        0 => { // LeftToRight
            // In LTR orientation:
            // - x=0 in Dasher is the right edge of the screen
            // - x increases to the left in Dasher, but to the right on screen
            // - y=0 in Dasher is the vertical center of the screen
            // - y increases upward in Dasher, but downward on screen
            
            // Scale factors
            let x_scale = screen_width as f64 / DASHER_X_MAX as f64;
            let y_scale = screen_height as f64 / DASHER_Y_MAX as f64;
            
            // Transform coordinates
            let screen_x = screen_width - (dasher_x as f64 * x_scale) as i32;
            let screen_y = (screen_height / 2) + ((dasher_y - DASHER_ORIGIN_Y) as f64 * y_scale) as i32;
            
            (screen_x, screen_y)
        },
        1 => { // RightToLeft
            // In RTL orientation:
            // - x=0 in Dasher is the left edge of the screen
            // - x increases to the right in Dasher, and to the left on screen
            // - y=0 in Dasher is the vertical center of the screen
            // - y increases upward in Dasher, but downward on screen
            
            // Scale factors
            let x_scale = screen_width as f64 / DASHER_X_MAX as f64;
            let y_scale = screen_height as f64 / DASHER_Y_MAX as f64;
            
            // Transform coordinates
            let screen_x = (dasher_x as f64 * x_scale) as i32;
            let screen_y = (screen_height / 2) + ((dasher_y - DASHER_ORIGIN_Y) as f64 * y_scale) as i32;
            
            (screen_x, screen_y)
        },
        2 => { // TopToBottom
            // In TTB orientation:
            // - y=0 in Dasher is the bottom edge of the screen
            // - y increases upward in Dasher, but downward on screen
            // - x=0 in Dasher is the horizontal center of the screen
            // - x increases to the right in Dasher, and to the right on screen
            
            // Scale factors
            let x_scale = screen_width as f64 / DASHER_Y_MAX as f64;
            let y_scale = screen_height as f64 / DASHER_X_MAX as f64;
            
            // Transform coordinates
            let screen_x = (screen_width / 2) + ((dasher_y - DASHER_ORIGIN_Y) as f64 * x_scale) as i32;
            let screen_y = screen_height - (dasher_x as f64 * y_scale) as i32;
            
            (screen_x, screen_y)
        },
        3 => { // BottomToTop
            // In BTT orientation:
            // - y=0 in Dasher is the top edge of the screen
            // - y increases downward in Dasher, and upward on screen
            // - x=0 in Dasher is the horizontal center of the screen
            // - x increases to the right in Dasher, and to the right on screen
            
            // Scale factors
            let x_scale = screen_width as f64 / DASHER_Y_MAX as f64;
            let y_scale = screen_height as f64 / DASHER_X_MAX as f64;
            
            // Transform coordinates
            let screen_x = (screen_width / 2) + ((dasher_y - DASHER_ORIGIN_Y) as f64 * x_scale) as i32;
            let screen_y = (dasher_x as f64 * y_scale) as i32;
            
            (screen_x, screen_y)
        },
        _ => {
            // Default to LeftToRight
            dasher_to_screen(dasher_x, dasher_y, screen_width, screen_height, 0)
        }
    }
}

/// Transform screen coordinates to Dasher coordinates
///
/// This function maps screen coordinates to Dasher coordinates based on the
/// screen dimensions and orientation.
///
/// # Arguments
///
/// * `screen_x` - X coordinate on the screen
/// * `screen_y` - Y coordinate on the screen
/// * `screen_width` - Width of the screen in pixels
/// * `screen_height` - Height of the screen in pixels
/// * `orientation` - Orientation of the view (0 = LeftToRight, 1 = RightToLeft, 2 = TopToBottom, 3 = BottomToTop)
///
/// # Returns
///
/// A tuple of (dasher_x, dasher_y) coordinates
pub fn screen_to_dasher(
    screen_x: i32,
    screen_y: i32,
    screen_width: i32,
    screen_height: i32,
    orientation: i32,
) -> (i64, i64) {
    // Apply orientation-specific transformations
    match orientation {
        0 => { // LeftToRight
            // Scale factors
            let x_scale = DASHER_X_MAX as f64 / screen_width as f64;
            let y_scale = DASHER_Y_MAX as f64 / screen_height as f64;
            
            // Transform coordinates
            let dasher_x = ((screen_width - screen_x) as f64 * x_scale) as i64;
            let dasher_y = DASHER_ORIGIN_Y + (((screen_y - (screen_height / 2)) as f64) * y_scale) as i64;
            
            (dasher_x, dasher_y)
        },
        1 => { // RightToLeft
            // Scale factors
            let x_scale = DASHER_X_MAX as f64 / screen_width as f64;
            let y_scale = DASHER_Y_MAX as f64 / screen_height as f64;
            
            // Transform coordinates
            let dasher_x = (screen_x as f64 * x_scale) as i64;
            let dasher_y = DASHER_ORIGIN_Y + (((screen_y - (screen_height / 2)) as f64) * y_scale) as i64;
            
            (dasher_x, dasher_y)
        },
        2 => { // TopToBottom
            // Scale factors
            let x_scale = DASHER_Y_MAX as f64 / screen_width as f64;
            let y_scale = DASHER_X_MAX as f64 / screen_height as f64;
            
            // Transform coordinates
            let dasher_x = ((screen_height - screen_y) as f64 * y_scale) as i64;
            let dasher_y = DASHER_ORIGIN_Y + (((screen_x - (screen_width / 2)) as f64) * x_scale) as i64;
            
            (dasher_x, dasher_y)
        },
        3 => { // BottomToTop
            // Scale factors
            let x_scale = DASHER_Y_MAX as f64 / screen_width as f64;
            let y_scale = DASHER_X_MAX as f64 / screen_height as f64;
            
            // Transform coordinates
            let dasher_x = (screen_y as f64 * y_scale) as i64;
            let dasher_y = DASHER_ORIGIN_Y + (((screen_x - (screen_width / 2)) as f64) * x_scale) as i64;
            
            (dasher_x, dasher_y)
        },
        _ => {
            // Default to LeftToRight
            screen_to_dasher(screen_x, screen_y, screen_width, screen_height, 0)
        }
    }
}

/// Apply coordinate transformation to rectangle coordinates
///
/// This function transforms a rectangle from Dasher space to screen space.
///
/// # Arguments
///
/// * `x1` - X coordinate of the first corner in Dasher space
/// * `y1` - Y coordinate of the first corner in Dasher space
/// * `x2` - X coordinate of the second corner in Dasher space
/// * `y2` - Y coordinate of the second corner in Dasher space
/// * `screen_width` - Width of the screen in pixels
/// * `screen_height` - Height of the screen in pixels
/// * `orientation` - Orientation of the view
///
/// # Returns
///
/// A tuple of (screen_x1, screen_y1, screen_x2, screen_y2) coordinates
pub fn transform_rectangle(
    x1: i64,
    y1: i64,
    x2: i64,
    y2: i64,
    screen_width: i32,
    screen_height: i32,
    orientation: i32,
) -> (i32, i32, i32, i32) {
    let (sx1, sy1) = dasher_to_screen(x1, y1, screen_width, screen_height, orientation);
    let (sx2, sy2) = dasher_to_screen(x2, y2, screen_width, screen_height, orientation);
    
    // Ensure x2 > x1 and y2 > y1 for screen coordinates
    let (screen_x1, screen_x2) = if sx1 <= sx2 { (sx1, sx2) } else { (sx2, sx1) };
    let (screen_y1, screen_y2) = if sy1 <= sy2 { (sy1, sy2) } else { (sy2, sy1) };
    
    (screen_x1, screen_y1, screen_x2, screen_y2)
}
