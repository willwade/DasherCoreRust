//! # Configuration Module for FFI
//!
//! This module provides functions for configuring the Dasher interface
//! through the FFI layer.

use crate::view::square::{SquareViewConfig, NodeShape};
use crate::view::DasherViewSquare;
use crate::api::DasherInterface;

/// Node shape types for FFI
#[repr(C)]
#[derive(Clone, Copy)]
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

impl From<NodeShapeFFI> for NodeShape {
    fn from(shape: NodeShapeFFI) -> Self {
        match shape {
            NodeShapeFFI::Rectangle => NodeShape::Rectangle,
            NodeShapeFFI::Triangle => NodeShape::Triangle,
            NodeShapeFFI::TruncatedTriangle => NodeShape::TruncatedTriangle,
            NodeShapeFFI::Circle => NodeShape::Circle,
            NodeShapeFFI::Quadric => NodeShape::Quadric,
        }
    }
}

impl From<NodeShape> for NodeShapeFFI {
    fn from(shape: NodeShape) -> Self {
        match shape {
            NodeShape::Rectangle => NodeShapeFFI::Rectangle,
            NodeShape::Triangle => NodeShapeFFI::Triangle,
            NodeShape::TruncatedTriangle => NodeShapeFFI::TruncatedTriangle,
            NodeShape::Circle => NodeShapeFFI::Circle,
            NodeShape::Quadric => NodeShapeFFI::Quadric,
        }
    }
}

/// Configuration options for the Dasher interface
#[repr(C)]
pub struct SquareViewConfigFFI {
    /// Node shape type
    pub node_shape: NodeShapeFFI,

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

    /// Margin width in abstract screen coordinates
    pub margin_width: i64,

    /// Whether to draw the crosshair
    pub draw_crosshair: bool,

    /// Whether to draw the cursor
    pub draw_cursor: bool,

    /// Whether to draw node outlines
    pub draw_outlines: bool,

    /// Whether to use the flowing interface (right to left movement)
    pub flowing_interface: bool,

    /// Speed of the flowing interface
    pub flowing_speed: f64,

    /// Whether to use PPM (Prediction by Partial Match) for node sizing
    pub use_ppm: bool,
}

impl From<SquareViewConfigFFI> for SquareViewConfig {
    fn from(config: SquareViewConfigFFI) -> Self {
        Self {
            node_shape: config.node_shape.into(),
            x_nonlinear: config.x_nonlinear,
            x_nonlinear_factor: config.x_nonlinear_factor,
            y_nonlinear: config.y_nonlinear,
            y1: config.y1,
            y2: config.y2,
            y3: config.y3,
            text_3d: config.text_3d,
            text_3d_depth: config.text_3d_depth,
            base_font_size: config.base_font_size,
            font_size_scaling: config.font_size_scaling,
            margin_width: config.margin_width,
            draw_crosshair: config.draw_crosshair,
            draw_cursor: config.draw_cursor,
            draw_outlines: config.draw_outlines,
            flowing_interface: config.flowing_interface,
            flowing_speed: config.flowing_speed,
            use_ppm: config.use_ppm,
        }
    }
}

impl From<SquareViewConfig> for SquareViewConfigFFI {
    fn from(config: SquareViewConfig) -> Self {
        Self {
            node_shape: config.node_shape.into(),
            x_nonlinear: config.x_nonlinear,
            x_nonlinear_factor: config.x_nonlinear_factor,
            y_nonlinear: config.y_nonlinear,
            y1: config.y1,
            y2: config.y2,
            y3: config.y3,
            text_3d: config.text_3d,
            text_3d_depth: config.text_3d_depth,
            base_font_size: config.base_font_size,
            font_size_scaling: config.font_size_scaling,
            margin_width: config.margin_width,
            draw_crosshair: config.draw_crosshair,
            draw_cursor: config.draw_cursor,
            draw_outlines: config.draw_outlines,
            flowing_interface: config.flowing_interface,
            flowing_speed: config.flowing_speed,
            use_ppm: config.use_ppm,
        }
    }
}

impl Default for SquareViewConfigFFI {
    fn default() -> Self {
        let default_config = SquareViewConfig::default();
        default_config.into()
    }
}

/// Get the square view from a DasherInterface
///
/// # Safety
///
/// This function is unsafe because it dereferences a raw pointer.
pub unsafe fn get_square_view(interface: &mut DasherInterface) -> Option<&mut DasherViewSquare> {
    if let Some(view) = interface.view_mut() {
        let any_view = view.as_any_mut();
        any_view.downcast_mut::<DasherViewSquare>()
    } else {
        None
    }
}

/// Convert FFI config to Rust config
///
/// # Safety
///
/// This function is unsafe because it dereferences a raw pointer.
pub unsafe fn ffi_to_rust_config(config: &SquareViewConfigFFI) -> SquareViewConfig {
    SquareViewConfig {
        node_shape: config.node_shape.into(),
        x_nonlinear: config.x_nonlinear,
        x_nonlinear_factor: config.x_nonlinear_factor,
        y_nonlinear: config.y_nonlinear,
        y1: config.y1,
        y2: config.y2,
        y3: config.y3,
        text_3d: config.text_3d,
        text_3d_depth: config.text_3d_depth,
        base_font_size: config.base_font_size,
        font_size_scaling: config.font_size_scaling,
        margin_width: config.margin_width,
        draw_crosshair: config.draw_crosshair,
        draw_cursor: config.draw_cursor,
        draw_outlines: config.draw_outlines,
        flowing_interface: config.flowing_interface,
        flowing_speed: config.flowing_speed,
        use_ppm: config.use_ppm,
    }
}
