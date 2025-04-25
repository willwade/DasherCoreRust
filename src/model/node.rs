//! # Node Module
//!
//! This module contains the implementation of DasherNode, which represents
//! a node in the Dasher tree.

use std::cell::RefCell;
use std::rc::{Rc, Weak};

/// Node flags representing the state of the node
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub struct NodeFlags(u32);

impl NodeFlags {
    /// Default flags for a new node
    pub const DEFAULT: Self = Self(0);

    /// Node has been seen (passed under the crosshair)
    pub const SEEN: u32 = 0x0001;

    /// Node has been committed (can't be undone)
    pub const COMMITTED: u32 = 0x0002;

    /// Node has all its children
    pub const ALL_CHILDREN: u32 = 0x0004;

    /// Node represents a word boundary (space, punctuation, etc.)
    pub const WORD_BOUNDARY: u32 = 0x0008;

    /// Node represents a predicted word
    pub const PREDICTED_WORD: u32 = 0x0010;

    /// Node is on the game path
    pub const GAME: u32 = 0x0020;

    /// Node has been converted
    pub const CONVERTED: u32 = 0x0040;
    pub const CONVERTED: u32 = 0x0010;

    /// Node is a control node
    pub const CONTROL: u32 = 0x0020;

    /// Node is a super node (fills the screen)
    pub const SUPER: u32 = 0x0040;

    /// Check if a flag is set
    pub fn is_set(&self, flag: u32) -> bool {
        (self.0 & flag) != 0
    }

    /// Set a flag
    pub fn set(&mut self, flag: u32, value: bool) {
        if value {
            self.0 |= flag;
        } else {
            self.0 &= !flag;
        }
    }
}

/// A node in the Dasher tree
pub struct DasherNode {
    /// Lower bound probability relative to parent
    lower_bound: u32,

    /// Upper bound probability relative to parent
    upper_bound: u32,

    /// Parent node
    parent: Option<Weak<RefCell<DasherNode>>>,

    /// Children nodes
    children: Vec<Rc<RefCell<DasherNode>>>,

    /// Binary flags representing the state of the node
    flags: NodeFlags,

    /// Offset into text buffer
    offset: i32,

    /// Label for the node
    label: Option<String>,

    /// Only child that was rendered (filled the screen)
    only_child_rendered: Option<Weak<RefCell<DasherNode>>>,

    /// The character represented by this node
    symbol: Option<char>,

    /// Foreground color for this node (RGB)
    foreground_color: (u8, u8, u8),

    /// Background color for this node (RGB)
    background_color: (u8, u8, u8),
}

impl DasherNode {
    /// Check if this node is a word boundary
    pub fn is_word_boundary(&self) -> bool {
        self.get_flag(NodeFlags::WORD_BOUNDARY)
    }

    /// Set this node as a word boundary
    pub fn set_word_boundary(&mut self, is_boundary: bool) {
        self.set_flag(NodeFlags::WORD_BOUNDARY, is_boundary);
    }

    /// Check if this node represents a predicted word
    pub fn is_predicted_word(&self) -> bool {
        self.get_flag(NodeFlags::PREDICTED_WORD)
    }

    /// Set this node as a predicted word
    pub fn set_predicted_word(&mut self, is_predicted: bool) {
        self.set_flag(NodeFlags::PREDICTED_WORD, is_predicted);
    }
    /// Normalization constant for probability calculations
    pub const NORMALIZATION: u32 = 1 << 16;

    /// Create a new Dasher node
    pub fn new(offset: i32, label: Option<String>) -> Self {
        Self {
            lower_bound: 0,
            upper_bound: Self::NORMALIZATION,
            parent: None,
            children: Vec::new(),
            flags: NodeFlags::DEFAULT,
            offset,
            label,
            only_child_rendered: None,
            symbol: None,
            foreground_color: (0, 0, 0),
            background_color: (255, 255, 255),
        }
    }

    /// Set the bounds of this node
    pub fn set_bounds(&mut self, lower_bound: u32, upper_bound: u32) {
        self.lower_bound = lower_bound;
        self.upper_bound = upper_bound;
    }

    /// Set the symbol for this node
    pub fn set_symbol(&mut self, symbol: char) {
        self.symbol = Some(symbol);
    }

    /// Get the symbol for this node
    pub fn symbol(&self) -> Option<char> {
        self.symbol
    }

    /// Set the colors for this node
    pub fn set_colors(&mut self, foreground: (u8, u8, u8), background: (u8, u8, u8)) {
        self.foreground_color = foreground;
        self.background_color = background;
    }

    /// Get the foreground color
    pub fn foreground_color(&self) -> (u8, u8, u8) {
        self.foreground_color
    }

    /// Get the background color
    pub fn background_color(&self) -> (u8, u8, u8) {
        self.background_color
    }

    /// Set the parent of this node
    pub fn set_parent(&mut self, parent: Weak<RefCell<DasherNode>>) {
        self.parent = Some(parent);
    }

    /// Add a child to this node
    pub fn add_child(&mut self, child: Rc<RefCell<DasherNode>>) {
        self.children.push(child);
    }

    /// Get the lower bound probability
    pub fn lower_bound(&self) -> u32 {
        self.lower_bound
    }

    /// Get the upper bound probability
    pub fn upper_bound(&self) -> u32 {
        self.upper_bound
    }

    /// Get the probability range of this node
    pub fn range(&self) -> u32 {
        self.upper_bound - self.lower_bound
    }

    /// Get the parent node
    pub fn parent(&self) -> Option<&Weak<RefCell<DasherNode>>> {
        self.parent.as_ref()
    }

    /// Get the children nodes
    pub fn children(&self) -> &Vec<Rc<RefCell<DasherNode>>> {
        &self.children
    }

    /// Get the number of children
    pub fn child_count(&self) -> usize {
        self.children.len()
    }

    /// Check if a flag is set
    pub fn get_flag(&self, flag: u32) -> bool {
        self.flags.is_set(flag)
    }

    /// Set a flag
    pub fn set_flag(&mut self, flag: u32, value: bool) {
        self.flags.set(flag, value);
    }

    /// Get the offset
    pub fn offset(&self) -> i32 {
        self.offset
    }

    /// Get the label
    pub fn label(&self) -> Option<&String> {
        self.label.as_ref()
    }

    /// Reparent this node to a new parent
    pub fn reparent(&mut self, parent: Weak<RefCell<DasherNode>>, lower_bound: u32, upper_bound: u32) {
        self.parent = Some(parent.clone());

        // Add this node to the parent's children
        if let Some(parent) = parent.upgrade() {
            parent.borrow_mut().children.push(Rc::new(RefCell::new(self.clone())));
        }

        self.lower_bound = lower_bound;
        self.upper_bound = upper_bound;
    }

    /// Delete all children of this node
    pub fn delete_children(&mut self) {
        self.children.clear();
        self.set_flag(NodeFlags::ALL_CHILDREN, false);
        self.only_child_rendered = None;
    }

    /// Delete nephews of the specified child
    pub fn delete_nephews(&mut self, child: &Rc<RefCell<DasherNode>>) {
        for node in &self.children {
            if !Rc::ptr_eq(node, child) {
                node.borrow_mut().delete_children();
            }
        }
    }

    /// Orphan a child of this node
    pub fn orphan_child(&mut self, child: &Rc<RefCell<DasherNode>>) {
        // Delete all other children
        for node in &self.children {
            if !Rc::ptr_eq(node, child) {
                node.borrow_mut().delete_children();
            }
        }

        // Clear the children list
        self.children.clear();

        // Set the child's parent to None
        child.borrow_mut().parent = None;

        // Reset the ALL_CHILDREN flag
        self.set_flag(NodeFlags::ALL_CHILDREN, false);
    }

    /// Find the most probable child
    pub fn most_probable_child(&self) -> u32 {
        let mut max = 0;

        for child in &self.children {
            let range = child.borrow().range();
            if range > max {
                max = range;
            }
        }

        max
    }

    /// Perform an action when this node is entered
    pub fn do_action(&mut self) {
        // If this node has a symbol, it should be added to the output
        if let Some(symbol) = self.symbol {
            // The actual appending to output text is handled by the model
            // This is just a placeholder for now
        }
    }

    /// Undo the action when this node is exited
    pub fn undo_action(&mut self) {
        // If this node has a symbol, it should be removed from the output
        if self.symbol.is_some() {
            // The actual removal from output text is handled by the model
            // This is just a placeholder for now
        }
    }

    /// Clone this node
    pub fn clone(&self) -> Self {
        Self {
            lower_bound: self.lower_bound,
            upper_bound: self.upper_bound,
            parent: self.parent.clone(),
            children: Vec::new(), // Don't clone children
            flags: self.flags,
            offset: self.offset,
            label: self.label.clone(),
            only_child_rendered: None,
            symbol: self.symbol,
            foreground_color: self.foreground_color,
            background_color: self.background_color,
        }
    }
}
