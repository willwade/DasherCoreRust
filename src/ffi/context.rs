//! # Context Module for FFI
//!
//! This module provides context information for the FFI layer.

use std::sync::atomic::{AtomicBool, AtomicI32, Ordering};
use std::sync::Mutex;
use std::collections::HashMap;
use std::sync::Arc;
use std::cell::RefCell;
use std::rc::Rc;
use crate::model::node::DasherNode;

/// Global context for FFI operations
pub struct FFIContext {
    /// Debug mode flag
    pub debug_mode: AtomicBool,

    /// Current screen width
    pub screen_width: AtomicI32,

    /// Current screen height
    pub screen_height: AtomicI32,

    /// Error messages
    pub error_messages: Mutex<Vec<String>>,

    /// Debug messages
    pub debug_messages: Mutex<Vec<String>>,

    /// Node information for debugging
    pub node_info: Mutex<HashMap<String, String>>,
}

impl FFIContext {
    /// Create a new FFI context
    pub fn new() -> Self {
        Self {
            debug_mode: AtomicBool::new(false),
            screen_width: AtomicI32::new(0),
            screen_height: AtomicI32::new(0),
            error_messages: Mutex::new(Vec::new()),
            debug_messages: Mutex::new(Vec::new()),
            node_info: Mutex::new(HashMap::new()),
        }
    }

    /// Set debug mode
    pub fn set_debug_mode(&self, debug: bool) {
        self.debug_mode.store(debug, Ordering::SeqCst);
    }

    /// Get debug mode
    pub fn get_debug_mode(&self) -> bool {
        self.debug_mode.load(Ordering::SeqCst)
    }

    /// Set screen dimensions
    pub fn set_screen_dimensions(&self, width: i32, height: i32) {
        self.screen_width.store(width, Ordering::SeqCst);
        self.screen_height.store(height, Ordering::SeqCst);
    }

    /// Get screen dimensions
    pub fn get_screen_dimensions(&self) -> (i32, i32) {
        (
            self.screen_width.load(Ordering::SeqCst),
            self.screen_height.load(Ordering::SeqCst),
        )
    }

    /// Add an error message
    pub fn add_error(&self, message: &str) {
        if let Ok(mut messages) = self.error_messages.lock() {
            messages.push(message.to_string());
        }
    }

    /// Add a debug message
    pub fn add_debug(&self, message: &str) {
        if self.get_debug_mode() {
            if let Ok(mut messages) = self.debug_messages.lock() {
                messages.push(message.to_string());
            }
        }
    }

    /// Get all error messages and clear the list
    pub fn get_errors(&self) -> Vec<String> {
        if let Ok(mut messages) = self.error_messages.lock() {
            let result = messages.clone();
            messages.clear();
            result
        } else {
            Vec::new()
        }
    }

    /// Get all debug messages and clear the list
    pub fn get_debug_messages(&self) -> Vec<String> {
        if let Ok(mut messages) = self.debug_messages.lock() {
            let result = messages.clone();
            messages.clear();
            result
        } else {
            Vec::new()
        }
    }

    /// Add node information
    pub fn add_node_info(&self, node_id: &str, info: &str) {
        if self.get_debug_mode() {
            if let Ok(mut node_info) = self.node_info.lock() {
                node_info.insert(node_id.to_string(), info.to_string());
            }
        }
    }

    /// Get node information
    pub fn get_node_info(&self, node_id: &str) -> Option<String> {
        if let Ok(node_info) = self.node_info.lock() {
            node_info.get(node_id).cloned()
        } else {
            None
        }
    }

    /// Clear all node information
    pub fn clear_node_info(&self) {
        if let Ok(mut node_info) = self.node_info.lock() {
            node_info.clear();
        }
    }
}

/// Global FFI context
static mut GLOBAL_CONTEXT: Option<Arc<FFIContext>> = None;

/// Initialize the global FFI context
pub fn init_global_context() -> Arc<FFIContext> {
    let context = Arc::new(FFIContext::new());
    unsafe {
        GLOBAL_CONTEXT = Some(context.clone());
    }
    context
}

/// Get the global FFI context
pub fn get_global_context() -> Arc<FFIContext> {
    unsafe {
        if let Some(context) = &GLOBAL_CONTEXT {
            context.clone()
        } else {
            init_global_context()
        }
    }
}

/// Enhanced callback context for drawing operations
#[derive(Clone, Debug)]
pub struct DrawingContext {
    /// Node ID for the current node being drawn
    pub node_id: String,

    /// Node depth in the tree
    pub depth: i32,

    /// Node probability
    pub probability: f64,

    /// Node symbol
    pub symbol: Option<char>,

    /// Node text
    pub text: String,

    /// Node is a leaf
    pub is_leaf: bool,

    /// Node is visible
    pub is_visible: bool,

    /// Node is selected
    pub is_selected: bool,
}

impl DrawingContext {
    /// Create a new drawing context
    pub fn new() -> Self {
        Self {
            node_id: String::new(),
            depth: 0,
            probability: 0.0,
            symbol: None,
            text: String::new(),
            is_leaf: false,
            is_visible: false,
            is_selected: false,
        }
    }

    /// Create a drawing context from a node
    pub fn from_node(node: &Rc<RefCell<crate::model::node::DasherNode>>) -> Self {
        let node_ref = node.borrow();
        Self {
            node_id: format!("{:p}", node),
            depth: node_ref.get_depth(),
            probability: node_ref.get_probability(),
            symbol: node_ref.symbol(),
            text: node_ref.label().cloned().unwrap_or_default(),
            is_leaf: node_ref.children().is_empty(),
            is_visible: true,
            is_selected: false,
        }
    }

    /// Convert to a string representation
    pub fn to_string(&self) -> String {
        format!(
            "Node: {} (depth={}, p={:.4}, symbol={:?}, text={}, leaf={}, visible={}, selected={})",
            self.node_id,
            self.depth,
            self.probability,
            self.symbol,
            self.text,
            self.is_leaf,
            self.is_visible,
            self.is_selected
        )
    }
}

/// Thread-local current drawing context
thread_local! {
    static CURRENT_DRAWING_CONTEXT: RefCell<DrawingContext> = RefCell::new(DrawingContext::new());
}

/// Set the current drawing context
pub fn set_current_drawing_context(context: DrawingContext) {
    CURRENT_DRAWING_CONTEXT.with(|c| {
        *c.borrow_mut() = context;
    });
}

/// Get the current drawing context
pub fn get_current_drawing_context() -> DrawingContext {
    CURRENT_DRAWING_CONTEXT.with(|c| {
        c.borrow().clone()
    })
}

/// Clear the current drawing context
pub fn clear_current_drawing_context() {
    CURRENT_DRAWING_CONTEXT.with(|c| {
        *c.borrow_mut() = DrawingContext::new();
    });
}

/// Add the current drawing context to the global context
pub fn log_current_drawing_context() {
    let context = get_current_drawing_context();
    let global = get_global_context();

    if global.get_debug_mode() {
        global.add_node_info(&context.node_id, &context.to_string());
    }
}
